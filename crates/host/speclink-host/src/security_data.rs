//! Explicit data-management operations. Offline review does not import this module.
use anyhow::{ensure, Result};
use serde::Serialize;
use speclink_core::security::{
    data::{InstallReceipt, IntegrityKind, StandardRepository, ValidatedStandard},
    standards::{source_catalog, standard_source, StandardId, StandardSource, MAX_DOWNLOAD_BYTES},
    JobProgress,
};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{IpAddr, SocketAddr, ToSocketAddrs},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct DataJob {
    pub id: String,
    cancelled: Arc<AtomicBool>,
    elapsed: Arc<dyn Fn() -> Duration + Send + Sync>,
    max_download_bytes: usize,
}
impl DataJob {
    pub fn cancellation_token(&self) -> Arc<AtomicBool> {
        self.cancelled.clone()
    }
    pub fn new(cancelled: Arc<AtomicBool>) -> Self {
        let start = Instant::now();
        Self {
            id: ulid::Ulid::new().to_string(),
            cancelled,
            elapsed: Arc::new(move || start.elapsed()),
            max_download_bytes: MAX_DOWNLOAD_BYTES,
        }
    }
    pub fn check(&self) -> Result<()> {
        ensure!(
            !self.cancelled.load(Ordering::Relaxed),
            "cancelled: data operation cancelled"
        );
        ensure!(
            (self.elapsed)() < Duration::from_secs(120),
            "budget_exceeded: data operation exceeded 120 seconds"
        );
        Ok(())
    }
    fn progress(&self, phase: &str, bytes: usize, files: usize) -> JobProgress {
        JobProgress {
            job_id: self.id.clone(),
            project_key: "application-data".into(),
            phase: phase.into(),
            completed_bytes: bytes as u64,
            total_bytes: None,
            completed_files: files,
            message: phase.into(),
        }
    }
}

pub struct DataResponse {
    pub status: u16,
    pub location: Option<String>,
    pub content_length: Option<u64>,
    pub body: Box<dyn Read + Send>,
}
pub trait DataTransport {
    fn fetch(&self, url: &str, job: &DataJob) -> Result<DataResponse>;
    fn head(&self, _url: &str, _job: &DataJob) -> Result<DataResponse> {
        anyhow::bail!("download_failed: HEAD unavailable")
    }
}

pub struct OfficialTransport {
    agent: ureq::Agent,
    job_id: String,
}
impl OfficialTransport {
    pub fn new(job: &DataJob) -> Self {
        let resolver_job = job.clone();
        let tls = CancelTls {
            config: tls_config(),
            job: job.clone(),
        };
        let agent = ureq::AgentBuilder::new()
            .redirects(0)
            .try_proxy_from_env(false)
            .timeout_connect(Duration::from_secs(1))
            .timeout_read(Duration::from_secs(1))
            .timeout_write(Duration::from_secs(1))
            .timeout(Duration::from_secs(120))
            .resolver(move |netloc: &str| resolve_public(netloc, &resolver_job))
            .tls_connector(Arc::new(tls))
            .build();
        Self {
            agent,
            job_id: job.id.clone(),
        }
    }
}
impl DataTransport for OfficialTransport {
    fn fetch(&self, url: &str, job: &DataJob) -> Result<DataResponse> {
        self.request("GET", url, job)
    }
    fn head(&self, url: &str, job: &DataJob) -> Result<DataResponse> {
        self.request("HEAD", url, job)
    }
}
impl OfficialTransport {
    fn request(&self, method: &str, url: &str, job: &DataJob) -> Result<DataResponse> {
        job.check()?;
        ensure!(
            self.job_id == job.id,
            "invalid_job: transport belongs to another data job"
        );
        ensure!(
            https_host(url).is_some_and(|h| official_host(&h)),
            "untrusted_source"
        );
        let response = match self
            .agent
            .request(method, url)
            .set("User-Agent", "SpecLink-security-data")
            .set("Accept", "application/json")
            .call()
        {
            Ok(response) | Err(ureq::Error::Status(_, response)) => response,
            Err(_) => {
                job.check()?;
                anyhow::bail!("download_failed: network request failed");
            }
        };
        job.check()?;
        Ok(DataResponse {
            status: response.status(),
            location: response.header("Location").map(str::to_owned),
            content_length: response
                .header("Content-Length")
                .and_then(|v| v.parse().ok()),
            body: response.into_reader(),
        })
    }
}

fn tls_config() -> Arc<ureq::rustls::ClientConfig> {
    static CONFIG: std::sync::OnceLock<Arc<ureq::rustls::ClientConfig>> =
        std::sync::OnceLock::new();
    CONFIG
        .get_or_init(|| {
            let roots = ureq::rustls::RootCertStore {
                roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
            };
            Arc::new(
                ureq::rustls::ClientConfig::builder_with_provider(
                    ureq::rustls::crypto::ring::default_provider().into(),
                )
                .with_protocol_versions(&[
                    &ureq::rustls::version::TLS12,
                    &ureq::rustls::version::TLS13,
                ])
                .expect("built-in TLS protocol versions")
                .with_root_certificates(roots)
                .with_no_client_auth(),
            )
        })
        .clone()
}

struct CancelTls {
    config: Arc<ureq::rustls::ClientConfig>,
    job: DataJob,
}
impl ureq::TlsConnector for CancelTls {
    fn connect(
        &self,
        name: &str,
        io: Box<dyn ureq::ReadWrite>,
    ) -> std::result::Result<Box<dyn ureq::ReadWrite>, ureq::Error> {
        ureq::TlsConnector::connect(
            &self.config,
            name,
            Box::new(CancelIo {
                io,
                job: self.job.clone(),
            }),
        )
    }
}
struct CancelIo {
    io: Box<dyn ureq::ReadWrite>,
    job: DataJob,
}
impl std::fmt::Debug for CancelIo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecurityDataStream")
    }
}
impl CancelIo {
    fn check(&self) -> std::io::Result<()> {
        self.job.check().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                "security data operation stopped",
            )
        })
    }
}
impl Read for CancelIo {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        self.check()?;
        if let Some(socket) = self.io.socket() {
            socket.set_read_timeout(Some(Duration::from_secs(1)))?;
        }
        let result = self.io.read(bytes);
        self.check()?;
        result
    }
}
impl Write for CancelIo {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.check()?;
        if let Some(socket) = self.io.socket() {
            socket.set_write_timeout(Some(Duration::from_secs(1)))?;
        }
        let result = self.io.write(bytes);
        self.check()?;
        result
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.check()?;
        self.io.flush()
    }
}
impl ureq::ReadWrite for CancelIo {
    fn socket(&self) -> Option<&std::net::TcpStream> {
        self.io.socket()
    }
}

fn resolve_public(netloc: &str, job: &DataJob) -> std::io::Result<Vec<SocketAddr>> {
    // getaddrinfo cannot be interrupted portably. Bound outstanding OS lookups;
    // abandoned results never start an HTTP request or mutate installed data.
    static LOOKUPS: AtomicUsize = AtomicUsize::new(0);
    struct LookupPermit;
    impl Drop for LookupPermit {
        fn drop(&mut self) {
            LOOKUPS.fetch_sub(1, Ordering::Relaxed);
        }
    }
    let (host, port) = netloc
        .rsplit_once(':')
        .ok_or_else(|| std::io::Error::other("untrusted source"))?;
    if !official_host(host) || port != "443" {
        return Err(std::io::Error::other("untrusted source"));
    }
    let mut count = LOOKUPS.load(Ordering::Relaxed);
    loop {
        if count >= 4 {
            return Err(std::io::Error::other("DNS worker capacity reached"));
        }
        match LOOKUPS.compare_exchange_weak(count, count + 1, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => break,
            Err(current) => count = current,
        }
    }
    let permit = LookupPermit;
    let address = netloc.to_owned();
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("security-data-dns".into())
        .spawn(move || {
            let _permit = permit;
            let result = address
                .to_socket_addrs()
                .map(|addresses| addresses.collect::<Vec<_>>());
            let _ = send.send(result);
        })?;
    let started = Instant::now();
    loop {
        if job.check().is_err() || started.elapsed() >= Duration::from_secs(1) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "DNS lookup stopped",
            ));
        }
        match receive.recv_timeout(Duration::from_millis(25)) {
            Ok(result) => {
                let addresses = result?;
                if addresses.is_empty()
                    || addresses
                        .iter()
                        .any(|a| a.port() != 443 || !public_address(a.ip()))
                {
                    return Err(std::io::Error::other("untrusted DNS address"));
                }
                return Ok(addresses);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => return Err(std::io::Error::other("DNS lookup unavailable")),
        }
    }
}

fn public_address(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(ip) => {
            let [a, b, _, _] = ip.octets();
            !ip.is_private()
                && !ip.is_loopback()
                && !ip.is_link_local()
                && !ip.is_documentation()
                && !ip.is_broadcast()
                && !ip.is_unspecified()
                && a != 0
                && a < 224
                && !(a == 100 && (64..=127).contains(&b))
                && !(a == 198 && (b == 18 || b == 19))
                && !(a == 192 && b == 0)
        }
        IpAddr::V6(ip) => {
            if let Some(v4) = ip.to_ipv4_mapped() {
                return public_address(IpAddr::V4(v4));
            }
            let octets = ip.octets();
            octets[0] & 0xe0 == 0x20
                && octets[..4] != [0x20, 0x01, 0x0d, 0xb8]
                && !(octets[0] == 0x3f && octets[1] == 0xff && octets[2] & 0xf0 == 0)
                && !(octets[0] == 0x20 && octets[1] == 0x02)
                && !(octets[..4] == [0x20, 0x01, 0, 0])
        }
    }
}

fn official_host(host: &str) -> bool {
    matches!(
        host,
        "github.com"
            | "raw.githubusercontent.com"
            | "api.github.com"
            | "cwe.mitre.org"
            | "release-assets.githubusercontent.com"
            | "objects.githubusercontent.com"
    )
}
fn https_host(value: &str) -> Option<String> {
    let request = ureq::get(value).request_url().ok()?;
    let url = request.as_url();
    (url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.port_or_known_default() == Some(443)
        && url.fragment().is_none())
    .then(|| url.host_str().map(str::to_owned))
    .flatten()
}
fn approved_source_url(url: &str) -> bool {
    https_host(url).is_some_and(|h| official_host(&h))
        && source_catalog().is_ok_and(|sources| {
            sources
                .iter()
                .any(|s| refresh_url(s) == url || s.files.iter().any(|f| f.url == url))
        })
}
fn approved_redirect(original: &str, next: &str) -> bool {
    if !approved_source_url(original) {
        return false;
    }
    let Some(host) = https_host(next) else {
        return false;
    };
    if original.starts_with("https://github.com/OWASP/ASVS/releases/download/") {
        matches!(
            host.as_str(),
            "release-assets.githubusercontent.com" | "objects.githubusercontent.com"
        )
    } else {
        next == original
    }
}

fn download_file(
    url: &str,
    transport: &dyn DataTransport,
    job: &DataJob,
    downloaded: &mut usize,
    progress: &dyn Fn(JobProgress),
) -> Result<Vec<u8>> {
    job.check()?;
    ensure!(
        approved_source_url(url),
        "untrusted_source: URL is not in the approved source catalog"
    );
    let file_limit = if url.ends_with(".xml.zip") {
        MAX_DOWNLOAD_BYTES
    } else {
        2 * 1024 * 1024
    };
    let mut current = url.to_owned();
    for redirects in 0..=3 {
        job.check()?;
        let response = transport.fetch(&current, job);
        job.check()?;
        let mut response =
            response.map_err(|_| anyhow::anyhow!("download_failed: network request failed"))?;
        if matches!(response.status, 301 | 302 | 303 | 307 | 308) {
            ensure!(redirects < 3, "untrusted_source: redirect limit");
            let next = response
                .location
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("untrusted_source: missing redirect destination"))?;
            let parsed = ureq::get(&current)
                .request_url()
                .map_err(|_| anyhow::anyhow!("untrusted_source"))?;
            let next = parsed
                .as_url()
                .join(next)
                .map_err(|_| anyhow::anyhow!("untrusted_source"))?
                .to_string();
            ensure!(
                approved_redirect(url, &next),
                "untrusted_source: redirect destination refused"
            );
            current = next;
            continue;
        }
        ensure!(
            response.status == 200,
            "download_failed: official source returned HTTP {}",
            response.status
        );
        if let Some(size) = response.content_length {
            ensure!(
                size <= file_limit.min(job.max_download_bytes.saturating_sub(*downloaded)) as u64,
                "size_limit: response length"
            );
        }
        let mut bytes = Vec::new();
        let mut chunk = [0; 64 * 1024];
        loop {
            job.check()?;
            let count = response.body.read(&mut chunk);
            job.check()?;
            let count =
                count.map_err(|_| anyhow::anyhow!("download_failed: response interrupted"))?;
            if count == 0 {
                break;
            }
            ensure!(
                count <= job.max_download_bytes.saturating_sub(*downloaded),
                "size_limit: actual downloaded bytes"
            );
            ensure!(
                count <= file_limit.saturating_sub(bytes.len()),
                "size_limit: individual source file"
            );
            *downloaded += count;
            bytes.extend_from_slice(&chunk[..count]);
            progress(job.progress("downloading", *downloaded, 0));
        }
        return Ok(bytes);
    }
    anyhow::bail!("untrusted_source: redirect limit")
}

pub type SourceDecoder = dyn Fn(
    StandardSource,
    BTreeMap<String, Vec<u8>>,
    IntegrityKind,
    &dyn Fn() -> bool,
) -> Result<ValidatedStandard>;

// Refresh checks only the shipped, parser-supported matrix. Remote responses
// never replace source URLs, digests, license metadata or executable rules.
fn refresh_url(source: &StandardSource) -> String {
    match source.standard_id {
        StandardId::OwaspTop10 => format!(
            "https://api.github.com/repos/OWASP/Top10/contents/{}/docs/en?ref={}",
            source.version, source.source_revision
        ),
        StandardId::OwaspApiTop10 => format!(
            "https://api.github.com/repos/OWASP/API-Security/contents/editions/{}/en?ref={}",
            source.version, source.source_revision
        ),
        StandardId::OwaspAsvs => format!(
            "https://api.github.com/repos/OWASP/ASVS/releases/tags/v{}_release",
            source.version
        ),
        StandardId::Cwe => source.source_url.clone(),
    }
}

pub fn refresh_data(
    repository: &dyn StandardRepository,
    transport: &dyn DataTransport,
    job: &DataJob,
    progress: &dyn Fn(JobProgress),
) -> Result<StandardDataList> {
    let mut inventory = list_data(repository)?;
    let mut downloaded = 0;
    for (index, source) in source_catalog()?.iter().enumerate() {
        job.check()?;
        let url = refresh_url(source);
        let available = if source.standard_id == StandardId::Cwe {
            let response = transport.head(&url, job);
            job.check()?;
            let response = response.map_err(|_| {
                anyhow::anyhow!("download_failed: source availability request failed")
            })?;
            ensure!(
                response.location.is_none(),
                "untrusted_source: availability redirect refused"
            );
            response.status == 200
                && response
                    .content_length
                    .is_some_and(|size| size > 0 && size <= MAX_DOWNLOAD_BYTES as u64)
        } else {
            let bytes = download_file(&url, transport, job, &mut downloaded, &|p| progress(p))?;
            ensure!(
                bytes.len() <= 2 * 1024 * 1024,
                "size_limit: version catalog response"
            );
            let value: serde_json::Value = serde_json::from_slice(&bytes)
                .map_err(|_| anyhow::anyhow!("invalid_catalog: expected official JSON"))?;
            if source.standard_id == StandardId::OwaspAsvs {
                value["tag_name"].as_str() == Some(format!("v{}_release", source.version).as_str())
                    && value["assets"].as_array().is_some_and(|assets| {
                        source
                            .files
                            .iter()
                            .filter(|f| f.name.ends_with(".json"))
                            .all(|file| {
                                assets.iter().any(|a| {
                                    a["name"].as_str() == Some(file.name.trim_start_matches("raw/"))
                                        && a["browser_download_url"].as_str()
                                            == Some(file.url.as_str())
                                })
                            })
                    })
            } else {
                value.as_array().is_some_and(|entries| {
                    source
                        .files
                        .iter()
                        .filter(|f| f.name != "LICENSE")
                        .all(|file| {
                            entries.iter().any(|e| {
                                e["type"] == "file"
                                    && e["name"].as_str()
                                        == Some(file.name.trim_start_matches("raw/"))
                                    && e["download_url"].as_str() == Some(file.url.as_str())
                            })
                        })
                })
            }
        };
        if !available {
            if let Some(row) = inventory
                .standards
                .iter_mut()
                .find(|s| s.standard_id == source.standard_id)
                .and_then(|s| s.versions.iter_mut().find(|v| v.version == source.version))
            {
                // Installed offline data stays usable when its upstream changes.
                if row.status != "installed" {
                    row.status = "unavailable".into();
                }
                row.error = Some(
                    "source_unavailable: official metadata does not match the supported source"
                        .into(),
                );
            }
        }
        progress(job.progress("refreshing", downloaded, index + 1));
    }
    job.check()?;
    Ok(inventory)
}

pub fn install_data(
    repository: &dyn StandardRepository,
    id: StandardId,
    version: &str,
    reinstall: bool,
    transport: &dyn DataTransport,
    decoder: &SourceDecoder,
    job: &DataJob,
    progress: &dyn Fn(JobProgress),
) -> Result<InstallReceipt> {
    install_source(
        repository,
        &standard_source(id, version)?,
        reinstall,
        transport,
        decoder,
        job,
        progress,
    )
}
fn install_source(
    repository: &dyn StandardRepository,
    source: &StandardSource,
    reinstall: bool,
    transport: &dyn DataTransport,
    decoder: &SourceDecoder,
    job: &DataJob,
    progress: &dyn Fn(JobProgress),
) -> Result<InstallReceipt> {
    job.check()?;
    let transaction = repository.begin_install(source.standard_id, &source.version)?;
    if !reinstall {
        ensure!(
            repository
                .read(source.standard_id, &source.version)?
                .is_none(),
            "already_installed: use reinstall to replace this version"
        );
    }
    let mut files = BTreeMap::new();
    let mut downloaded = 0;
    for (index, file) in source.files.iter().enumerate() {
        job.check()?;
        let bytes = download_file(&file.url, transport, job, &mut downloaded, &|mut p| {
            p.completed_files = index;
            progress(p);
        })?;
        files.insert(file.name.clone(), bytes);
        progress(job.progress("downloading", downloaded, index + 1));
    }
    job.check()?;
    progress(job.progress("validating", downloaded, files.len()));
    let kind = if source.files.iter().all(|f| f.sha256.is_some()) {
        IntegrityKind::PinnedDigest
    } else {
        IntegrityKind::SourceTls
    };
    let validated = decoder(source.clone(), files, kind, &|| job.check().is_err());
    job.check()?;
    let validated = validated?;
    progress(job.progress("installing", downloaded, source.files.len()));
    let result = transaction.commit_cancellable(validated, &|| job.check().is_err());
    // A successful receipt means the atomic commit happened, even if cancel arrives later.
    if result.is_err() {
        job.check()?;
    }
    result
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardDataList {
    pub standards: Vec<StandardDataEntry>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardDataEntry {
    pub standard_id: StandardId,
    pub versions: Vec<StandardVersionEntry>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardVersionEntry {
    pub version: String,
    pub status: String,
    pub source_url: String,
    pub license_name: String,
    pub license_url: String,
    pub attribution: String,
    pub source_format: String,
    pub error: Option<String>,
}

pub fn list_data(repository: &dyn StandardRepository) -> Result<StandardDataList> {
    let catalog = source_catalog()?;
    let mut rows = std::collections::BTreeMap::<StandardId, Vec<StandardVersionEntry>>::new();
    for source in &catalog {
        let (status, error) = match repository.read(source.standard_id, &source.version) {
            Ok(Some(_)) => ("installed", None),
            Ok(None) => ("missing", None),
            Err(error) => ("invalid", Some(error.to_string())),
        };
        rows.entry(source.standard_id)
            .or_default()
            .push(StandardVersionEntry {
                version: source.version.clone(),
                status: status.into(),
                source_url: source.source_url.clone(),
                license_name: source.license_name.clone(),
                license_url: source.license_url.clone(),
                attribution: source.attribution.clone(),
                source_format: source.source_format.clone(),
                error,
            });
    }
    for (id, version) in repository.versions()? {
        if catalog
            .iter()
            .any(|s| s.standard_id == id && s.version == version)
        {
            continue;
        }
        if let Some(installed) = repository.read(id, &version)? {
            let m = installed.metadata;
            rows.entry(id).or_default().push(StandardVersionEntry {
                version,
                status: "installed".into(),
                source_url: m.source_url,
                license_name: m.license_name,
                license_url: m.license_url,
                attribution: m.attribution,
                source_format: m.source_format,
                error: None,
            });
        }
    }
    Ok(StandardDataList {
        standards: rows
            .into_iter()
            .map(|(standard_id, versions)| StandardDataEntry {
                standard_id,
                versions,
            })
            .collect(),
    })
}

/// The path is explicitly selected by the user; decoding and bounded I/O belong
/// to the repository adapter. This operation has no network capability.
pub fn import_data(
    repository: &dyn StandardRepository,
    path: &std::path::Path,
    cancelled: &dyn Fn() -> bool,
) -> Result<speclink_core::security::data::InstallReceipt> {
    repository.import_package(path, cancelled)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveReceipt {
    pub standard_id: StandardId,
    pub version: String,
    pub removed: bool,
    pub reference_scope: String,
}
pub fn remove_data(
    repository: &dyn StandardRepository,
    standard_id: StandardId,
    version: &str,
) -> Result<RemoveReceipt> {
    repository.remove(standard_id, version)?;
    Ok(RemoveReceipt { standard_id, version:version.into(), removed:true, reference_scope:"locally known policies, prepared snapshots and active readers; unopened remote projects are not enumerated".into() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_core::security::{
        data::{IntegrityKind, StandardRepository, ValidatedStandard},
        standards::{standard_source, StandardId},
    };

    struct HttpFixture {
        base: String,
        task: Option<std::thread::JoinHandle<()>>,
    }
    impl HttpFixture {
        fn new(response: &'static [u8], delay: std::time::Duration) -> Self {
            use std::io::{Read, Write};
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let base = format!("http://{}", listener.local_addr().unwrap());
            let task = std::thread::spawn(move || {
                let started = std::time::Instant::now();
                let (mut stream, _) = loop {
                    match listener.accept() {
                        Ok(connection) => break connection,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            if started.elapsed() > std::time::Duration::from_secs(2) {
                                return;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(5));
                        }
                        Err(error) => panic!("fixture accept: {error}"),
                    }
                };
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                    .unwrap();
                let mut buffer = [0; 4096];
                let _ = stream.read(&mut buffer);
                std::thread::sleep(delay);
                let _ = stream.write_all(response);
            });
            Self {
                base,
                task: Some(task),
            }
        }
    }
    impl Drop for HttpFixture {
        fn drop(&mut self) {
            if let Some(task) = self.task.take() {
                task.join().unwrap();
            }
        }
    }
    // Test-only transport maps an approved public source to loopback. The production
    // source decision is tested independently and never gains a localhost exception.
    struct FixtureTransport(String);
    impl DataTransport for FixtureTransport {
        fn fetch(&self, _url: &str, _job: &DataJob) -> Result<DataResponse> {
            let response = ureq::AgentBuilder::new()
                .redirects(0)
                .timeout_connect(std::time::Duration::from_secs(1))
                .timeout_read(std::time::Duration::from_secs(1))
                .build()
                .get(&self.0)
                .call()
                .map_err(|_| anyhow::anyhow!("download_failed: fixture transport"))?;
            Ok(DataResponse {
                status: response.status(),
                location: response.header("Location").map(str::to_owned),
                content_length: response
                    .header("Content-Length")
                    .and_then(|n| n.parse().ok()),
                body: response.into_reader(),
            })
        }
    }

    fn download_job() -> DataJob {
        DataJob::new(std::sync::Arc::new(std::sync::atomic::AtomicBool::new(
            false,
        )))
    }

    #[test]
    fn security_download_refresh_uses_structured_pinned_queries_and_preserves_inventory_on_failure()
    {
        struct CatalogTransport {
            fail: bool,
        }
        impl DataTransport for CatalogTransport {
            fn fetch(&self, url: &str, _: &DataJob) -> Result<DataResponse> {
                if self.fail {
                    anyhow::bail!("private-response-sentinel");
                }
                assert!(url.starts_with("https://api.github.com/repos/OWASP/"));
                assert!(!url.contains("latest"));
                let sources = source_catalog()?;
                let source = sources.iter().find(|s| refresh_url(s) == url).unwrap();
                let value = if source.standard_id == StandardId::OwaspAsvs {
                    serde_json::json!({"tag_name": format!("v{}_release",source.version), "assets": source.files.iter().filter(|f|f.name.ends_with(".json")).map(|f|serde_json::json!({"name":f.name.trim_start_matches("raw/"),"browser_download_url":f.url})).collect::<Vec<_>>()})
                } else {
                    serde_json::Value::Array(source.files.iter().filter(|f| f.name != "LICENSE").map(|f|serde_json::json!({"type":"file","name":f.name.trim_start_matches("raw/"),"download_url":f.url})).collect())
                };
                Ok(DataResponse {
                    status: 200,
                    location: None,
                    content_length: None,
                    body: Box::new(std::io::Cursor::new(serde_json::to_vec(&value)?)),
                })
            }
            fn head(&self, url: &str, _: &DataJob) -> Result<DataResponse> {
                assert_eq!(url, "https://cwe.mitre.org/data/xml/cwec_v4.20.xml.zip");
                Ok(DataResponse {
                    status: 200,
                    location: None,
                    content_length: Some(2_021_351),
                    body: Box::new(std::io::empty()),
                })
            }
        }
        let directory = tempfile::tempdir().unwrap();
        let repository = speclink_fs::security::FsSecurityData::new(directory.path()).unwrap();
        let before = serde_json::to_value(list_data(&repository).unwrap()).unwrap();
        let events = std::cell::RefCell::new(Vec::new());
        let updated = refresh_data(
            &repository,
            &CatalogTransport { fail: false },
            &download_job(),
            &|p| events.borrow_mut().push(p),
        )
        .unwrap();
        assert_eq!(serde_json::to_value(updated).unwrap(), before);
        assert_eq!(events.borrow().last().unwrap().completed_files, 7);
        let error = refresh_data(
            &repository,
            &CatalogTransport { fail: true },
            &download_job(),
            &|_| {},
        )
        .err()
        .unwrap();
        assert!(!error.to_string().contains("sentinel"));
        assert_eq!(
            serde_json::to_value(list_data(&repository).unwrap()).unwrap(),
            before
        );
        assert!(!directory.path().join("security").exists());
    }

    #[test]
    fn security_download_official_url_and_redirect_decisions_are_separate_from_transport() {
        for source in source_catalog().unwrap() {
            for file in source.files {
                assert!(approved_source_url(&file.url), "{}", file.url);
            }
        }
        for url in [
            "http://github.com/OWASP/ASVS/releases/download/x",
            "https://127.0.0.1/x",
            "https://github.com.evil.invalid/OWASP/ASVS",
            "https://user:secret@github.com/OWASP/ASVS",
            "https://cwe.mitre.org:444/data/xml/cwec_v4.20.xml.zip",
            "https://raw.githubusercontent.com/evil/project/main/run.sh",
        ] {
            assert!(!approved_source_url(url));
        }
        let asset = standard_source(StandardId::OwaspAsvs, "5.0.0")
            .unwrap()
            .files[0]
            .url
            .clone();
        assert!(approved_redirect(&asset, "https://release-assets.githubusercontent.com/github-production-release-asset/file?signature=redacted"));
        assert!(!approved_redirect(&asset, "https://evil.invalid/file"));
        assert!(!approved_redirect(
            &asset,
            "http://release-assets.githubusercontent.com/file"
        ));
        for address in [
            "127.0.0.1",
            "10.0.0.1",
            "172.16.0.1",
            "192.168.1.1",
            "169.254.1.1",
            "100.64.0.1",
            "::1",
            "fc00::1",
            "fe80::1",
            "::ffff:127.0.0.1",
        ] {
            assert!(!public_address(address.parse().unwrap()), "{address}");
        }
        assert!(public_address("8.8.8.8".parse().unwrap()));
        assert!(public_address("2606:4700:4700::1111".parse().unwrap()));
    }

    #[test]
    fn security_download_bounds_actual_chunked_bytes_and_refuses_untrusted_redirects() {
        use std::time::Duration;
        let fixture = HttpFixture::new(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n5\r\n12345\r\n4\r\n6789\r\n0\r\n\r\n", Duration::ZERO);
        let transport = FixtureTransport(fixture.base.clone());
        let source = standard_source(StandardId::Cwe, "4.20").unwrap().files[0]
            .url
            .clone();
        let mut job = download_job();
        job.max_download_bytes = 8;
        assert!(download_file(&source, &transport, &job, &mut 0, &|_| {})
            .err()
            .unwrap()
            .to_string()
            .contains("size_limit"));
        let fixture = HttpFixture::new(b"HTTP/1.1 302 Found\r\nLocation: https://evil.invalid/file\r\nContent-Length: 0\r\nConnection: close\r\n\r\n", Duration::ZERO);
        let transport = FixtureTransport(fixture.base.clone());
        assert!(
            download_file(&source, &transport, &download_job(), &mut 0, &|_| {})
                .err()
                .unwrap()
                .to_string()
                .contains("untrusted_source")
        );
    }

    #[test]
    fn security_download_virtual_deadline_and_cancel_during_blocked_read() {
        use std::{
            sync::{
                atomic::{AtomicBool, Ordering},
                Arc,
            },
            time::{Duration, Instant},
        };
        let mut job = download_job();
        job.elapsed = Arc::new(|| Duration::from_secs(120));
        assert!(job
            .check()
            .unwrap_err()
            .to_string()
            .contains("budget_exceeded"));
        let fixture = HttpFixture::new(
            b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\ndata",
            Duration::from_millis(1300),
        );
        let cancel = Arc::new(AtomicBool::new(false));
        let job = DataJob::new(cancel.clone());
        let control = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(30));
            cancel.store(true, Ordering::Relaxed);
        });
        let source = standard_source(StandardId::Cwe, "4.20").unwrap().files[0]
            .url
            .clone();
        let started = Instant::now();
        let error = download_file(
            &source,
            &FixtureTransport(fixture.base.clone()),
            &job,
            &mut 0,
            &|_| {},
        )
        .err()
        .unwrap();
        assert!(error.to_string().contains("cancelled"));
        assert!(started.elapsed() < Duration::from_secs(2));
        control.join().unwrap();
    }

    #[test]
    fn security_download_failure_keeps_installed_generation_and_redacts_transport_error() {
        struct Failure;
        impl DataTransport for Failure {
            fn fetch(&self, _: &str, _: &DataJob) -> Result<DataResponse> {
                anyhow::bail!("do-not-propagate-response-body-or-signed-url");
            }
        }
        let directory = tempfile::tempdir().unwrap();
        let repository = speclink_fs::security::FsSecurityData::new(directory.path()).unwrap();
        let mut source = standard_source(StandardId::OwaspAsvs, "5.0.0").unwrap();
        for file in &mut source.files {
            file.sha256 = None;
        }
        let name = source
            .files
            .iter()
            .find(|f| f.name.ends_with(".json"))
            .unwrap()
            .name
            .clone();
        let files = std::collections::BTreeMap::from([
            (name, br#"{"Version":"5.0.0","Requirements":[{"Shortcode":"V1.1.1","Description":"Synthetic old data","L":"1"}]}"#.to_vec()),
            ("LICENSE".into(), b"Synthetic fixture license".to_vec()),
        ]);
        let first = repository
            .begin_install(source.standard_id, &source.version)
            .unwrap()
            .commit(
                ValidatedStandard::new(source.clone(), files, IntegrityKind::Imported, &|| false)
                    .unwrap(),
            )
            .unwrap();
        let error = install_source(
            &repository,
            &source,
            true,
            &Failure,
            &speclink_fs::security::decode_sources,
            &download_job(),
            &|_| {},
        )
        .err()
        .unwrap();
        assert!(error.to_string().contains("download_failed"));
        assert!(!error.to_string().contains("do-not-propagate"));
        assert_eq!(
            repository
                .read(source.standard_id, &source.version)
                .unwrap()
                .unwrap()
                .metadata
                .generation,
            first.generation
        );
    }

    #[test]
    fn security_download_success_reports_progress_and_preserves_source_tls_trust() {
        struct MemoryTransport(BTreeMap<String, Vec<u8>>);
        impl DataTransport for MemoryTransport {
            fn fetch(&self, url: &str, _: &DataJob) -> Result<DataResponse> {
                let bytes = self.0.get(url).unwrap().clone();
                Ok(DataResponse {
                    status: 200,
                    location: None,
                    content_length: Some(bytes.len() as u64),
                    body: Box::new(std::io::Cursor::new(bytes)),
                })
            }
        }
        let directory = tempfile::tempdir().unwrap();
        let repository = speclink_fs::security::FsSecurityData::new(directory.path()).unwrap();
        let mut source = standard_source(StandardId::OwaspAsvs, "5.0.0").unwrap();
        source.files.iter_mut().for_each(|f| f.sha256 = None);
        let transport = MemoryTransport(source.files.iter().map(|f|(f.url.clone(), if f.name == "LICENSE" {b"Synthetic license".to_vec()} else {br#"{"Version":"5.0.0","Requirements":[{"Shortcode":"V1.1.1","Description":"Synthetic fixture","L":"1"}]}"#.to_vec()})).collect());
        let job = download_job();
        let events = std::cell::RefCell::new(Vec::new());
        let receipt = install_source(
            &repository,
            &source,
            false,
            &transport,
            &speclink_fs::security::decode_sources,
            &job,
            &|p| events.borrow_mut().push(p),
        )
        .unwrap();
        assert_eq!(receipt.integrity_kind, IntegrityKind::SourceTls);
        assert_eq!(receipt.installed_status, "installed");
        assert!(events
            .borrow()
            .iter()
            .all(|p| p.job_id == job.id && p.project_key == "application-data"));
        assert_eq!(events.borrow().last().unwrap().phase, "installing");
        assert_eq!(events.borrow().last().unwrap().completed_files, 2);
        let cancelled = download_job();
        cancelled.cancelled.store(true, Ordering::Relaxed);
        assert!(install_source(
            &repository,
            &source,
            true,
            &transport,
            &speclink_fs::security::decode_sources,
            &cancelled,
            &|_| {}
        )
        .err()
        .unwrap()
        .to_string()
        .contains("cancelled"));
        assert_eq!(
            repository
                .read(source.standard_id, &source.version)
                .unwrap()
                .unwrap()
                .metadata
                .generation,
            receipt.generation
        );
    }

    #[test]
    fn security_data_inventory_offline_keeps_source_license_and_installed_state() {
        let dir = tempfile::tempdir().unwrap();
        let repository = speclink_fs::security::FsSecurityData::new(dir.path()).unwrap();
        let initial = list_data(&repository).unwrap();
        assert_eq!(initial.standards.len(), 4);
        assert!(initial
            .standards
            .iter()
            .flat_map(|s| &s.versions)
            .all(|v| v.status == "missing"
                && !v.license_name.is_empty()
                && v.source_url.starts_with("https://")));
        let mut source = standard_source(StandardId::OwaspAsvs, "5.0.0").unwrap();
        for file in &mut source.files {
            file.sha256 = None;
        }
        let name = source
            .files
            .iter()
            .find(|f| f.name.ends_with(".json"))
            .unwrap()
            .name
            .clone();
        let files = std::collections::BTreeMap::from([
            (name, br#"{"Version":"5.0.0","Requirements":[{"Shortcode":"V1.1.1","Description":"Synthetic test","L":"1"}]}"#.to_vec()),
            ("LICENSE".into(), b"Synthetic licensed fixture".to_vec()),
        ]);
        repository
            .begin_install(StandardId::OwaspAsvs, "5.0.0")
            .unwrap()
            .commit(
                ValidatedStandard::new(source, files, IntegrityKind::Imported, &|| false).unwrap(),
            )
            .unwrap();
        let installed = list_data(&repository).unwrap();
        let row = installed
            .standards
            .iter()
            .find(|s| s.standard_id == StandardId::OwaspAsvs)
            .unwrap()
            .versions
            .iter()
            .find(|v| v.version == "5.0.0")
            .unwrap();
        assert_eq!(row.status, "installed");
        assert_eq!(row.license_name, "CC-BY-SA-4.0");
        assert!(!row.attribution.is_empty());
        let legacy = installed
            .standards
            .iter()
            .find(|s| s.standard_id == StandardId::OwaspAsvs)
            .unwrap()
            .versions
            .iter()
            .find(|v| v.version == "4.0.3")
            .unwrap();
        assert_eq!(legacy.license_name, "CC-BY-SA-3.0");
        let json = serde_json::to_value(installed).unwrap();
        assert!(json["standards"][0]["standardId"].is_string());
        assert!(json["standards"][0]["versions"][0]["licenseUrl"].is_string());
    }

    #[test]
    fn security_remove_known_policy_and_version_lock_protect_data() {
        use speclink_core::security::policy::SecurityPolicy;
        let directory = tempfile::tempdir().unwrap();
        let repository = speclink_fs::security::FsSecurityData::new(directory.path()).unwrap();
        let policy: SecurityPolicy = serde_json::from_value(serde_json::json!({"enabled":true,"standards":{"owasp_top10":{"enabled":true,"version":"2025"}}})).unwrap();
        repository
            .remember_policy("synthetic-project", &policy)
            .unwrap();
        assert!(remove_data(&repository, StandardId::OwaspTop10, "2025")
            .unwrap_err()
            .to_string()
            .contains("in_use"));
        repository
            .remember_policy("synthetic-project", &SecurityPolicy::default())
            .unwrap();
        let transaction = repository
            .begin_install(StandardId::OwaspTop10, "2025")
            .unwrap();
        assert!(remove_data(&repository, StandardId::OwaspTop10, "2025")
            .unwrap_err()
            .to_string()
            .contains("busy"));
        drop(transaction);
        let result = remove_data(&repository, StandardId::OwaspTop10, "2025").unwrap();
        assert!(result.removed);
        assert!(repository
            .read(StandardId::OwaspTop10, "2025")
            .unwrap()
            .is_none());
        let value = serde_json::to_value(result).unwrap();
        assert_eq!(value["standardId"], "owasp_top10");
        assert_eq!(value["version"], "2025");
        assert!(value["referenceScope"].as_str().unwrap().contains("known"));
    }
}
