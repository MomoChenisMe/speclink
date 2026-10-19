//! Offline package codec and atomic publication; selection/commit policy lives in Host.
use anyhow::{ensure, Context, Result};
use speclink_core::security::{
    data::*,
    standards::{StandardId, MAX_DOWNLOAD_BYTES, MAX_PACKAGE_BYTES},
};
use std::{
    collections::BTreeSet,
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom, Write},
    path::Path,
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};
const MAX_COLLECTION_BYTES: u64 = 512 * 1024 * 1024;
const MAX_COLLECTION_RAW: usize = 1024 * 1024 * 1024;
const MAX_COLLECTION_PACKAGES: usize = 32;
fn options(method: CompressionMethod) -> SimpleFileOptions {
    SimpleFileOptions::default()
        .compression_method(method)
        .unix_permissions(0o644)
}
fn check(cancelled: &dyn Fn() -> bool) -> Result<()> {
    ensure!(!cancelled(), "cancelled");
    Ok(())
}

pub fn encode_single(source: ExportSource, cancelled: &dyn Fn() -> bool) -> Result<Vec<u8>> {
    check(cancelled)?;
    let m = source.metadata;
    ensure!(
        m.files.len() == source.files.len(),
        "integrity_error: export entries"
    );
    let manifest = PackageManifest {
        format_version: 1,
        standard_id: m.standard_id,
        version: m.version,
        source_revision: m.source_revision,
        source_url: m.source_url,
        source_format: m.source_format,
        license_name: m.license_name,
        license_url: m.license_url,
        attribution: m.attribution,
        entries: source
            .files
            .iter()
            .map(|(name, bytes)| PackageEntry {
                name: name.clone(),
                sha256: sha256(bytes),
            })
            .collect(),
    };
    manifest.source()?;
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file("manifest.json", options(CompressionMethod::Deflated))?;
    zip.write_all(&serde_json::to_vec(&manifest)?)?;
    let mut total = 0usize;
    for (name, bytes) in source.files {
        check(cancelled)?;
        total += bytes.len();
        ensure!(total <= MAX_PACKAGE_BYTES, "size_limit: export source");
        ensure!(
            m.files.get(&name) == Some(&sha256(&bytes)),
            "integrity_error: export source"
        );
        zip.start_file(name, options(CompressionMethod::Deflated))?;
        for chunk in bytes.chunks(64 * 1024) {
            check(cancelled)?;
            zip.write_all(chunk)?;
        }
    }
    let bytes = zip.finish()?.into_inner();
    ensure!(
        bytes.len() <= MAX_DOWNLOAD_BYTES,
        "size_limit: compressed package"
    );
    // The generated package must pass the exact same source/license/parser boundary as imports.
    crate::security::read_offline_package(&bytes, cancelled)?;
    Ok(bytes)
}

pub fn write_export(
    path: &Path,
    selected: &[(StandardId, String)],
    all: bool,
    skipped: Vec<SkippedPackage>,
    load: &dyn Fn(StandardId, &str) -> Result<ExportSource>,
    cancelled: &dyn Fn() -> bool,
) -> Result<ExportReceipt> {
    check(cancelled)?;
    ensure!(!selected.is_empty(), "missing_data");
    ensure!(
        selected.len() <= MAX_COLLECTION_PACKAGES && (all || selected.len() == 1),
        "size_limit: package count"
    );
    ensure!(!path.try_exists()?, "file_exists");
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temp =
        tempfile::NamedTempFile::new_in(parent).context("storage_error: export temporary file")?;
    let mut packages = Vec::new();
    if all {
        let mut zip = ZipWriter::new(temp.as_file_mut());
        let mut entries = Vec::new();
        let mut total = 0u64;
        let mut total_raw = 0usize;
        for (id, version) in selected {
            check(cancelled)?;
            let source = load(*id, version)?;
            total_raw = expanded_total(total_raw, source.files.values().map(Vec::len).sum())?;
            let bytes = encode_single(source, cancelled)?;
            total += bytes.len() as u64;
            ensure!(
                total <= MAX_COLLECTION_BYTES,
                "size_limit: collection packages"
            );
            let digest = sha256(&bytes);
            let name = format!("packages/{}/{version}.zip", id.as_str());
            zip.start_file(&name, options(CompressionMethod::Stored))?;
            for chunk in bytes.chunks(64 * 1024) {
                check(cancelled)?;
                zip.write_all(chunk)?;
            }
            entries.push(CollectionEntry {
                standard_id: *id,
                version: version.clone(),
                name,
                sha256: digest.clone(),
            });
            packages.push(ExportedPackage {
                standard_id: *id,
                version: version.clone(),
                sha256: digest,
            });
        }
        let manifest = serde_json::to_vec(&CollectionManifest {
            format_version: 2,
            packages: entries,
        })?;
        ensure!(
            manifest.len() <= 64 * 1024,
            "size_limit: collection manifest"
        );
        zip.start_file("manifest.json", options(CompressionMethod::Stored))?;
        zip.write_all(&manifest)?;
        zip.finish()?;
    } else {
        let (id, version) = &selected[0];
        let bytes = encode_single(load(*id, version)?, cancelled)?;
        packages.push(ExportedPackage {
            standard_id: *id,
            version: version.clone(),
            sha256: sha256(&bytes),
        });
        for chunk in bytes.chunks(64 * 1024) {
            check(cancelled)?;
            temp.write_all(chunk)?;
        }
    }
    let bytes = temp.as_file().metadata()?.len();
    ensure!(
        bytes <= MAX_COLLECTION_BYTES,
        "size_limit: collection output"
    );
    temp.as_file().sync_all()?;
    check(cancelled)?;
    temp.persist_noclobber(path).map_err(|e| {
        if e.error.kind() == std::io::ErrorKind::AlreadyExists {
            anyhow::anyhow!("file_exists")
        } else {
            anyhow::anyhow!("storage_error: atomic export publication")
        }
    })?;
    Ok(ExportReceipt {
        path: path.to_string_lossy().into_owned(),
        format_version: if all { 2 } else { 1 },
        bytes,
        packages,
        skipped,
    })
}

struct StagedCollection {
    manifest: CollectionManifest,
    staging: tempfile::TempDir,
}
impl PreparedCollection for StagedCollection {
    fn manifest(&self) -> &CollectionManifest {
        &self.manifest
    }
    fn load(&self, index: usize, cancelled: &dyn Fn() -> bool) -> Result<ValidatedStandard> {
        let bytes = read_bounded(
            File::open(self.staging.path().join(index.to_string()))?,
            MAX_DOWNLOAD_BYTES,
            cancelled,
        )?;
        ensure!(
            sha256(&bytes).eq_ignore_ascii_case(&self.manifest.packages[index].sha256),
            "integrity_error: staging package"
        );
        crate::security::read_offline_package(&bytes, cancelled)
    }
}
fn read_bounded(
    mut reader: impl Read,
    max: usize,
    cancelled: &dyn Fn() -> bool,
) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut chunk = [0; 64 * 1024];
    loop {
        check(cancelled)?;
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        ensure!(
            n <= max.saturating_sub(bytes.len()),
            "size_limit: actual package bytes"
        );
        bytes.extend_from_slice(&chunk[..n]);
    }
    Ok(bytes)
}
// Bound the central directory before ZipArchive allocates its index. No ZIP64/multi-disk.
fn preflight(file: &mut File) -> Result<usize> {
    let length = file.metadata()?.len();
    ensure!(
        (22..=MAX_COLLECTION_BYTES).contains(&length),
        "size_limit: collection ZIP"
    );
    let mut header = [0; 4];
    file.read_exact(&mut header)?;
    ensure!(&header == b"PK\x03\x04", "invalid_package: ZIP header");
    let tail_len = length.min(65_557) as usize;
    file.seek(SeekFrom::End(-(tail_len as i64)))?;
    let mut tail = vec![0; tail_len];
    file.read_exact(&mut tail)?;
    for offset in (0..=tail_len - 22).rev() {
        if tail[offset..offset + 4] != *b"PK\x05\x06" {
            continue;
        }
        let u16_at =
            |n| u16::from_le_bytes(tail[offset + n..offset + n + 2].try_into().unwrap()) as usize;
        let u32_at =
            |n| u32::from_le_bytes(tail[offset + n..offset + n + 4].try_into().unwrap()) as u64;
        if offset + 22 + u16_at(20) != tail_len {
            continue;
        }
        ensure!(
            u16_at(4) == 0 && u16_at(6) == 0 && u16_at(8) == u16_at(10),
            "invalid_package: multi-disk ZIP"
        );
        let count = u16_at(10);
        ensure!((1..=128).contains(&count), "size_limit: ZIP entry count");
        ensure!(
            offset < 20 || tail[offset - 20..offset - 16] != *b"PK\x06\x07",
            "unsupported_format: ZIP64"
        );
        let eocd = length - tail_len as u64 + offset as u64;
        ensure!(
            u32_at(16) <= eocd && u32_at(12) == eocd - u32_at(16),
            "invalid_package: central directory bounds"
        );
        file.seek(SeekFrom::Start(0))?;
        return Ok(count);
    }
    anyhow::bail!("invalid_package: missing ZIP end record")
}
/// Returns None for v1, leaving its original import path/output unchanged.
/// All v2 packages are parsed and validated before this returns any commit candidates.
pub fn prepare_collection(
    path: &Path,
    cancelled: &dyn Fn() -> bool,
) -> Result<Option<Box<dyn PreparedCollection>>> {
    check(cancelled)?;
    let mut file = File::open(path)?;
    let entries = preflight(&mut file)?;
    let mut headers = file.try_clone()?;
    let config = zip::read::Config {
        archive_offset: zip::read::ArchiveOffset::Known(0),
    };
    let mut zip =
        ZipArchive::with_config(config, file).context("invalid_package: ZIP directory")?;
    ensure!(
        zip.len() == entries && !zip.has_overlapping_files()?,
        "invalid_package: duplicate or overlapping ZIP entries"
    );
    let manifest_bytes = read_bounded(zip.by_name("manifest.json")?, 64 * 1024, cancelled)?;
    let format: serde_json::Value =
        serde_json::from_slice(&manifest_bytes).context("invalid_package: manifest JSON")?;
    if format.get("formatVersion") == Some(&serde_json::json!(1)) {
        return Ok(None);
    }
    let manifest: CollectionManifest =
        serde_json::from_slice(&manifest_bytes).context("invalid_package: collection manifest")?;
    let names = collection_names(&manifest)?;
    ensure!(
        zip.len() == manifest.packages.len() + 1 && zip.len() <= 33,
        "invalid_package: unlisted collection entries"
    );
    let mut seen = BTreeSet::new();
    let mut total = 0u64;
    for i in 0..zip.len() {
        check(cancelled)?;
        let entry = zip.by_index(i)?;
        let name = std::str::from_utf8(entry.name_raw()).context("invalid_package: ZIP name")?;
        ensure!(
            names.contains(name)
                && seen.insert(name.to_ascii_lowercase())
                && !entry.is_dir()
                && !entry.encrypted()
                && !entry.is_symlink(),
            "invalid_package: collection ZIP entry"
        );
        ensure!(
            matches!(
                entry.compression(),
                CompressionMethod::Stored | CompressionMethod::Deflated
            ),
            "unsupported_format: ZIP codec"
        );
        ensure!(
            entry
                .unix_mode()
                .is_none_or(|m| m & 0o111 == 0 && matches!(m & 0o170000, 0 | 0o100000)),
            "invalid_package: special ZIP entry"
        );
        ensure!(
            entry.size()
                <= if name == "manifest.json" {
                    64 * 1024
                } else {
                    MAX_DOWNLOAD_BYTES as u64
                },
            "size_limit: inner package"
        );
        total = total
            .checked_add(entry.size())
            .context("size_limit: collection bytes")?;
        ensure!(
            total <= MAX_COLLECTION_BYTES,
            "size_limit: collection bytes"
        );
        ensure!(
            entry.extra_data().is_none_or(|data| !has_zip64(data)),
            "unsupported_format: ZIP64 entry"
        );
        let offset = entry.header_start();
        drop(entry);
        check_local_header(&mut headers, offset)?;
    }
    let staging = tempfile::Builder::new()
        .prefix("speclink-security-import-")
        .tempdir()?;
    let mut total_raw = 0usize;
    let mut actual = 0u64;
    for (index, package) in manifest.packages.iter().enumerate() {
        let bytes = read_bounded(zip.by_name(&package.name)?, MAX_DOWNLOAD_BYTES, cancelled)?;
        actual += bytes.len() as u64;
        ensure!(
            actual <= MAX_COLLECTION_BYTES,
            "size_limit: collection bytes"
        );
        ensure!(
            sha256(&bytes).eq_ignore_ascii_case(&package.sha256),
            "integrity_error: inner package fingerprint"
        );
        let decoded = crate::security::read_offline_package(&bytes, cancelled)?;
        ensure!(
            decoded.source().standard_id == package.standard_id
                && decoded.source().version == package.version,
            "invalid_package: inner identity"
        );
        total_raw = expanded_total(total_raw, decoded.files().values().map(Vec::len).sum())?;
        check(cancelled)?;
        let mut output = File::create(staging.path().join(index.to_string()))?;
        output.write_all(&bytes)?;
        output.sync_all()?;
    }
    Ok(Some(Box::new(StagedCollection { manifest, staging })))
}
fn collection_names(manifest: &CollectionManifest) -> Result<BTreeSet<String>> {
    ensure!(
        manifest.format_version == 2
            && !manifest.packages.is_empty()
            && manifest.packages.len() <= MAX_COLLECTION_PACKAGES,
        "invalid_package: collection count/version"
    );
    let mut names = BTreeSet::from(["manifest.json".to_owned()]);
    let mut identities = BTreeSet::new();
    for package in &manifest.packages {
        ensure!(
            package.standard_id.supports_version(&package.version)
                && package.name
                    == format!(
                        "packages/{}/{}.zip",
                        package.standard_id.as_str(),
                        package.version
                    ),
            "invalid_package: collection path"
        );
        ensure!(
            names.insert(package.name.clone())
                && identities.insert((package.standard_id, package.version.clone())),
            "invalid_package: duplicate collection package"
        );
        ensure!(
            package.sha256.len() == 64 && package.sha256.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid_package: collection fingerprint"
        );
    }
    Ok(names)
}
fn expanded_total(total: usize, additional: usize) -> Result<usize> {
    let next = total
        .checked_add(additional)
        .context("size_limit: collection expanded bytes")?;
    ensure!(
        next <= MAX_COLLECTION_RAW,
        "size_limit: collection expanded bytes"
    );
    Ok(next)
}
pub(crate) fn check_local_header(reader: &mut (impl Read + Seek), offset: u64) -> Result<()> {
    reader
        .seek(SeekFrom::Start(offset))
        .context("invalid_package: ZIP local header")?;
    let mut header = [0; 30];
    reader
        .read_exact(&mut header)
        .context("invalid_package: ZIP local header")?;
    ensure!(
        &header[..4] == b"PK\x03\x04",
        "invalid_package: ZIP local header"
    );
    let value = |offset| u16::from_le_bytes([header[offset], header[offset + 1]]);
    ensure!(value(4) < 45, "unsupported_format: ZIP64 local header");
    ensure!(
        value(6) & (1 | 0x40 | 0x2000) == 0,
        "invalid_package: encrypted local ZIP entry"
    );
    reader.seek(SeekFrom::Current(value(26) as i64))?;
    let mut extra = vec![0; value(28) as usize];
    reader
        .read_exact(&mut extra)
        .context("invalid_package: ZIP local extras")?;
    ensure!(!has_zip64(&extra), "unsupported_format: ZIP64 local entry");
    Ok(())
}
pub(crate) fn has_zip64(mut data: &[u8]) -> bool {
    while data.len() >= 4 {
        let tag = u16::from_le_bytes([data[0], data[1]]);
        let len = u16::from_le_bytes([data[2], data[3]]) as usize;
        if tag == 1 {
            return true;
        }
        if len > data.len() - 4 {
            return true;
        }
        data = &data[4 + len..];
    }
    !data.is_empty()
}
#[cfg(test)]
mod tests {
    use super::*;
    use speclink_core::security::standards::standard_source;
    use std::{
        collections::BTreeMap,
        io::{Cursor, Write},
    };
    pub fn source() -> ExportSource {
        let files=BTreeMap::from([
          ("raw/cwec_v4.20.xml".into(),br#"<Weakness_Catalog xmlns="http://cwe.mitre.org/cwe-7" Version="4.20"><Weaknesses><Weakness ID="79" Name="Synthetic"><Description>Synthetic fixture</Description></Weakness></Weaknesses></Weakness_Catalog>"#.to_vec()),
          ("LICENSE".into(),b"MITRE CWE Terms of Use synthetic fixture".to_vec())]);
        let data = ValidatedStandard::new(
            standard_source(StandardId::Cwe, "4.20").unwrap(),
            files,
            IntegrityKind::Imported,
            &|| false,
        )
        .unwrap();
        ExportSource {
            metadata: data.metadata("generation".into(), "now".into()).unwrap(),
            files: data.files().clone(),
        }
    }
    fn collection(manifest: serde_json::Value, entries: Vec<(&str, Vec<u8>)>) -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default().unix_permissions(0o644);
        zip.start_file("manifest.json", options).unwrap();
        zip.write_all(&serde_json::to_vec(&manifest).unwrap())
            .unwrap();
        for (name, bytes) in entries {
            zip.start_file(name, options).unwrap();
            zip.write_all(&bytes).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }
    #[test]
    fn local_zip64_headers_are_rejected_even_without_a_zip64_footer() {
        let bytes = encode_single(source(), &|| false).unwrap();
        let mut original = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .large_file(true)
            .unix_permissions(0o644);
        for index in 0..original.len() {
            let mut file = original.by_index(index).unwrap();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            zip.start_file(file.name(), options).unwrap();
            zip.write_all(&bytes).unwrap();
        }
        let zip64 = zip.finish().unwrap().into_inner();
        assert!(crate::security::read_offline_package(&zip64, &|| false).is_err());
    }
    #[test]
    fn collection_count_file_and_expansion_limits_are_inclusive() {
        let mut manifest = CollectionManifest {
            format_version: 2,
            packages: (1..=32)
                .map(|i| CollectionEntry {
                    standard_id: StandardId::Cwe,
                    version: format!("4.{i}"),
                    name: format!("packages/cwe/4.{i}.zip"),
                    sha256: "0".repeat(64),
                })
                .collect(),
        };
        assert!(collection_names(&manifest).is_ok());
        manifest.packages.push(manifest.packages[0].clone());
        assert!(collection_names(&manifest).is_err());
        assert_eq!(
            expanded_total(MAX_COLLECTION_RAW - 1, 1).unwrap(),
            MAX_COLLECTION_RAW
        );
        assert!(expanded_total(MAX_COLLECTION_RAW, 1).is_err());
        assert!(expanded_total(usize::MAX, 1).is_err());
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("large.zip");
        let file = File::create(&path).unwrap();
        file.set_len(MAX_COLLECTION_BYTES + 1).unwrap();
        assert!(prepare_collection(&path, &|| false)
            .err()
            .unwrap()
            .to_string()
            .contains("size_limit"));
    }
    #[test]
    fn v1_cwe_round_trip_preserves_source_license_and_imported_trust() {
        let original = source();
        let metadata = original.metadata.clone();
        let raw = original.files.clone();
        let bytes = encode_single(original, &|| false).unwrap();
        let decoded = crate::security::read_offline_package(&bytes, &|| false).unwrap();
        assert_eq!(decoded.files(), &raw);
        assert_eq!(decoded.source().source_revision, metadata.source_revision);
        assert_eq!(
            decoded
                .metadata("new".into(), "now".into())
                .unwrap()
                .integrity_kind,
            IntegrityKind::Imported
        );
    }
    #[test]
    fn collection_round_trip_and_atomic_destination_cancel() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("all.zip");
        let selected = vec![(StandardId::Cwe, "4.20".into())];
        let receipt = write_export(
            &path,
            &selected,
            true,
            vec![],
            &|_, _| Ok(source()),
            &|| false,
        )
        .unwrap();
        assert_eq!(receipt.format_version, 2);
        assert_eq!(receipt.packages.len(), 1);
        let prepared = prepare_collection(&path, &|| false).unwrap().unwrap();
        assert_eq!(
            prepared.load(0, &|| false).unwrap().source().version,
            "4.20"
        );
        let original = std::fs::read(&path).unwrap();
        assert!(write_export(
            &path,
            &selected,
            false,
            vec![],
            &|_, _| Ok(source()),
            &|| false
        )
        .unwrap_err()
        .to_string()
        .contains("file_exists"));
        assert_eq!(std::fs::read(&path).unwrap(), original);
        let cancelled = dir.path().join("cancel.zip");
        assert!(write_export(
            &cancelled,
            &selected,
            true,
            vec![],
            &|_, _| Ok(source()),
            &|| true
        )
        .is_err());
        assert!(!cancelled.exists());
    }
    #[test]
    fn collection_rejects_duplicate_slip_unlisted_sha_license_and_nested() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.zip");
        let inner = encode_single(source(), &|| false).unwrap();
        let name = "packages/cwe/4.20.zip";
        let entry = serde_json::json!({"standardId":"cwe","version":"4.20","name":name,"sha256":sha256(&inner)});
        let good = serde_json::json!({"formatVersion":2,"packages":[entry.clone()]});
        let mut cases = Vec::new();
        let mut bad = good.clone();
        bad["packages"] = serde_json::json!([entry.clone(), entry.clone()]);
        cases.push((bad, vec![(name, inner.clone())]));
        let mut bad = good.clone();
        bad["packages"][0]["name"] = "../outside.zip".into();
        cases.push((bad, vec![("../outside.zip", inner.clone())]));
        let mut bad = good.clone();
        bad["packages"][0]["sha256"] = "0".repeat(64).into();
        cases.push((bad, vec![(name, inner.clone())]));
        cases.push((good.clone(), vec![(name, inner.clone()), ("extra", vec![])]));
        let nested = collection(good.clone(), vec![(name, inner.clone())]);
        let mut bad = good.clone();
        bad["packages"][0]["sha256"] = sha256(&nested).into();
        cases.push((bad, vec![(name, nested)]));
        let mut bad = good.clone();
        bad["packages"] = serde_json::json!(vec![entry; 33]);
        cases.push((bad, vec![(name, inner)]));
        for (manifest, entries) in cases {
            std::fs::write(&path, collection(manifest, entries)).unwrap();
            assert!(prepare_collection(&path, &|| false).is_err());
        }
        let mut no_license = source();
        no_license.files.remove("LICENSE");
        assert!(encode_single(no_license, &|| false).is_err());
    }
}
