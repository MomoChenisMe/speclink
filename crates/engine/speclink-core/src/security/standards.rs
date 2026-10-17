//! Parsers for approved data formats. Input is data only; references are never fetched.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

use anyhow::{bail, ensure, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

pub const PARSER_VERSION: &str = "1";
pub const MAX_STANDARD_ITEMS: usize = 100_000;
pub const MAX_DOWNLOAD_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_DATA_FILE_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_XML_BYTES: usize = 128 * 1024 * 1024;
pub const MAX_PACKAGE_BYTES: usize = 256 * 1024 * 1024;
pub const MAX_PACKAGE_ENTRIES: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StandardId {
    OwaspTop10,
    OwaspApiTop10,
    OwaspAsvs,
    Cwe,
}

impl StandardId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OwaspTop10 => "owasp_top10",
            Self::OwaspApiTop10 => "owasp_api_top10",
            Self::OwaspAsvs => "owasp_asvs",
            Self::Cwe => "cwe",
        }
    }
    pub fn supports_version(self, version: &str) -> bool {
        match self {
            Self::OwaspTop10 => matches!(version, "2021" | "2025"),
            Self::OwaspApiTop10 => matches!(version, "2019" | "2023"),
            Self::OwaspAsvs => matches!(version, "4.0.3" | "5.0.0"),
            Self::Cwe => super::policy::valid_version(version),
        }
    }
}

impl std::str::FromStr for StandardId {
    type Err = anyhow::Error;
    fn from_str(value: &str) -> Result<Self> {
        match value {
            "owasp_top10" => Ok(Self::OwaspTop10),
            "owasp_api_top10" => Ok(Self::OwaspApiTop10),
            "owasp_asvs" => Ok(Self::OwaspAsvs),
            "cwe" => Ok(Self::Cwe),
            _ => bail!(
                "unsupported_standard: expected owasp_top10, owasp_api_top10, owasp_asvs or cwe"
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceFile {
    pub name: String,
    pub url: String,
    pub sha256: Option<String>,
    pub item_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StandardSource {
    pub standard_id: StandardId,
    pub version: String,
    pub source_url: String,
    pub source_revision: String,
    pub source_format: String,
    pub license_name: String,
    pub license_url: String,
    pub attribution: String,
    pub files: Vec<SourceFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CweMapping {
    pub cwe_id: String,
    pub source_url: String,
    pub source_revision: String,
    pub section: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub levels: Vec<u8>,
    pub cwe_mappings: Vec<CweMapping>,
    pub relationships: Vec<CweRelationship>,
    /// Installing a requirement never attests that the project satisfies it.
    pub verification_status: super::ReviewStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CweRelationship {
    pub cwe_id: String,
    pub nature: String,
    pub view_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardSnapshot {
    pub standard_id: StandardId,
    pub version: String,
    pub items: Vec<StandardItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlAssessment {
    pub control_id: String,
    pub status: super::ReviewStatus,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardAssessment {
    pub standard_id: StandardId,
    pub version: Option<String>,
    pub level: Option<super::policy::AsvsLevel>,
    pub status: super::ReviewStatus,
    pub reason: Option<String>,
    pub controls: Vec<ControlAssessment>,
    pub metadata: Option<super::data::StandardMetadata>,
}
pub fn associate_standards(
    policy: &super::policy::SecurityPolicy,
    available: &[super::data::InstalledStandard],
    results: &mut [super::rules::RuleResult],
    stop: &dyn Fn() -> Option<&'static str>,
) -> Vec<StandardAssessment> {
    use super::{rules::StandardReference, ReviewStatus};
    for finding in results.iter_mut().flat_map(|r| &mut r.findings) {
        finding.references.clear();
    }
    let selected = policy.standards.selected();
    let mut assessments = Vec::new();
    for standard_id in [
        StandardId::OwaspTop10,
        StandardId::OwaspApiTop10,
        StandardId::OwaspAsvs,
        StandardId::Cwe,
    ] {
        let choice = selected
            .iter()
            .find(|(id, _, _)| *id == standard_id.as_str());
        let mut assessment = StandardAssessment {
            standard_id,
            version: choice.map(|(_, v, _)| v.to_string()),
            level: choice.and_then(|(_, _, l)| *l),
            status: ReviewStatus::Disabled,
            reason: None,
            controls: Vec::new(),
            metadata: None,
        };
        if !policy.enabled || choice.is_none() {
            assessments.push(assessment);
            continue;
        }
        let (_, version, level) = choice.unwrap();
        let data = available.iter().find(|d| {
            d.snapshot.standard_id == standard_id
                && d.metadata.standard_id == standard_id
                && d.snapshot.version == *version
                && d.metadata.version == *version
        });
        let Some(data) = data else {
            assessment.status = ReviewStatus::NotVerified;
            assessment.reason = Some("missing_data".into());
            assessments.push(assessment);
            continue;
        };
        assessment.metadata = Some(data.metadata.clone());
        assessment.status = ReviewStatus::NotVerified;
        if let Some(reason) = stop() {
            assessment.reason = Some(reason.into());
            assessments.push(assessment);
            continue;
        }
        assessment.reason = Some(
            if standard_id == StandardId::Cwe {
                "classification_only"
            } else {
                "preliminary_rules_do_not_verify_entire_standard"
            }
            .into(),
        );
        let mut stopped = None;
        'items: for item in &data.snapshot.items {
            if let Some(reason) = stop() {
                stopped = Some(reason);
                break;
            }
            if standard_id == StandardId::OwaspAsvs
                && !item
                    .levels
                    .contains(&level.unwrap_or(super::policy::AsvsLevel::L1).number())
            {
                continue;
            }
            let mut found = false;
            for finding in results.iter_mut().flat_map(|r| &mut r.findings) {
                if let Some(reason) = stop() {
                    stopped = Some(reason);
                    break 'items;
                }
                if standard_id == StandardId::Cwe {
                    if finding.cwe_ids.contains(&item.id) {
                        finding.references.push(StandardReference {
                            standard_id,
                            version: version.to_string(),
                            control_id: item.id.clone(),
                            source_url: data.metadata.source_url.clone(),
                            source_revision: data.metadata.source_revision.clone(),
                            section: "Weaknesses/Weakness".into(),
                        });
                        found = true;
                    }
                } else if matches!(
                    standard_id,
                    StandardId::OwaspTop10 | StandardId::OwaspApiTop10
                ) {
                    for mapping in &item.cwe_mappings {
                        if let Some(reason) = stop() {
                            stopped = Some(reason);
                            break 'items;
                        }
                        if !finding.cwe_ids.contains(&mapping.cwe_id) {
                            continue;
                        }
                        let reference = StandardReference {
                            standard_id,
                            version: version.to_string(),
                            control_id: item.id.clone(),
                            source_url: mapping.source_url.clone(),
                            source_revision: mapping.source_revision.clone(),
                            section: mapping.section.clone(),
                        };
                        if !finding.references.iter().any(|r| {
                            r.standard_id == standard_id
                                && r.version == *version
                                && r.control_id == item.id
                        }) {
                            finding.references.push(reference);
                        }
                        found = true;
                    }
                }
            }
            if standard_id == StandardId::Cwe && !found {
                continue;
            }
            assessment.controls.push(ControlAssessment {
                control_id: item.id.clone(),
                status: if found {
                    ReviewStatus::Finding
                } else {
                    ReviewStatus::NotVerified
                },
                reason: if found {
                    "heuristic_finding_associated_by_official_cwe_witness"
                } else {
                    "no_supported_direct_control_verification"
                }
                .into(),
            });
        }
        if let Some(reason) = stopped {
            assessment.reason = Some(reason.into());
        } else if assessment
            .controls
            .iter()
            .any(|c| c.status == ReviewStatus::Finding)
        {
            assessment.status = ReviewStatus::Finding;
        }
        assessments.push(assessment);
    }
    assessments
}

impl StandardSnapshot {
    pub fn items_for_level(
        &self,
        level: super::policy::AsvsLevel,
    ) -> impl Iterator<Item = &StandardItem> {
        self.items
            .iter()
            .filter(move |item| item.levels.contains(&level.number()))
    }
    pub fn item_key(&self, item: &StandardItem) -> String {
        format!("{}/{}/{}", self.standard_id.as_str(), self.version, item.id)
    }
}

pub fn source_catalog() -> Result<Vec<StandardSource>> {
    let sources: Vec<StandardSource> =
        serde_json::from_str(include_str!("../../assets/security/sources.json"))?;
    let mut keys = BTreeSet::new();
    for source in &sources {
        validate_source(source)?;
        ensure!(
            keys.insert((source.standard_id, &source.version)),
            "invalid_catalog: duplicate standard version"
        );
    }
    Ok(sources)
}

pub fn standard_source(id: StandardId, version: &str) -> Result<StandardSource> {
    source_catalog()?
        .into_iter()
        .find(|s| s.standard_id == id && s.version == version)
        .ok_or_else(|| {
            anyhow::anyhow!("unsupported_version: standard version is not in the approved catalog")
        })
}

pub fn validate_source(source: &StandardSource) -> Result<()> {
    ensure!(
        source.standard_id.supports_version(&source.version),
        "unsupported_version"
    );
    ensure!(
        !source.source_revision.is_empty() && source.source_url.starts_with("https://"),
        "invalid_source"
    );
    ensure!(
        !source.license_name.is_empty()
            && source.license_url.starts_with("https://")
            && !source.attribution.is_empty(),
        "license_missing"
    );
    ensure!(
        !source.files.is_empty() && source.files.len() <= MAX_PACKAGE_ENTRIES,
        "invalid_package: entry count"
    );
    let mut names = BTreeSet::new();
    for file in &source.files {
        ensure!(
            valid_data_name(&file.name) && names.insert(file.name.to_ascii_lowercase()),
            "invalid_package: unsafe or duplicate name"
        );
        ensure!(file.url.starts_with("https://"), "invalid_source");
        if let Some(hash) = &file.sha256 {
            ensure!(
                hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()),
                "invalid_package: fingerprint"
            );
        }
    }
    ensure!(names.contains("license"), "license_missing");
    Ok(())
}

pub fn valid_data_name(name: &str) -> bool {
    if name == "LICENSE" {
        return true;
    }
    name.strip_prefix("raw/").is_some_and(|base| {
        !base.is_empty()
            && base.len() <= 240
            && !base.contains(['/', '\\', ':', '<', '>', '"', '|', '?', '*'])
            && !base.ends_with(['.', ' '])
            && !matches!(
                base.split('.')
                    .next()
                    .unwrap_or("")
                    .to_ascii_uppercase()
                    .as_str(),
                "CON"
                    | "PRN"
                    | "AUX"
                    | "NUL"
                    | "COM1"
                    | "COM2"
                    | "COM3"
                    | "COM4"
                    | "COM5"
                    | "COM6"
                    | "COM7"
                    | "COM8"
                    | "COM9"
                    | "LPT1"
                    | "LPT2"
                    | "LPT3"
                    | "LPT4"
                    | "LPT5"
                    | "LPT6"
                    | "LPT7"
                    | "LPT8"
                    | "LPT9"
            )
            && base != "."
            && base != ".."
            && !base.chars().any(char::is_control)
            && (base.ends_with(".md")
                || base.ends_with(".json")
                || base.ends_with(".xml")
                || base.ends_with(".xml.zip"))
    })
}

pub fn parse_standard(
    source: &StandardSource,
    files: &BTreeMap<String, Vec<u8>>,
    cancelled: &dyn Fn() -> bool,
) -> Result<StandardSnapshot> {
    validate_source(source)?;
    ensure!(!cancelled(), "cancelled");
    ensure!(
        files.len() <= MAX_PACKAGE_ENTRIES,
        "size_limit: entry count"
    );
    let mut total = 0usize;
    for (name, bytes) in files {
        ensure!(
            valid_data_name(name),
            "invalid_package: unexpected file name"
        );
        ensure!(
            bytes.len()
                <= if name.ends_with(".xml") {
                    MAX_XML_BYTES
                } else {
                    MAX_DATA_FILE_BYTES
                },
            "size_limit: data file"
        );
        total = total
            .checked_add(bytes.len())
            .ok_or_else(|| anyhow::anyhow!("size_limit"))?;
        ensure!(total <= MAX_PACKAGE_BYTES, "size_limit: package");
    }
    let license = files
        .get("LICENSE")
        .ok_or_else(|| anyhow::anyhow!("license_missing"))?;
    ensure!(!license.is_empty(), "license_missing");
    let items = match source.source_format.as_str() {
        "markdown"
            if matches!(
                source.standard_id,
                StandardId::OwaspTop10 | StandardId::OwaspApiTop10
            ) =>
        {
            parse_markdown(source, files, cancelled)?
        }
        "asvs-json" if source.standard_id == StandardId::OwaspAsvs => {
            parse_asvs(source, files, cancelled)?
        }
        "cwe-xml-zip" if source.standard_id == StandardId::Cwe => {
            parse_cwe(source, files, cancelled)?
        }
        _ => bail!("unsupported_format"),
    };
    let mut ids = BTreeSet::new();
    ensure!(
        !items.is_empty() && items.len() <= MAX_STANDARD_ITEMS,
        "invalid_package: item count"
    );
    for item in &items {
        ensure!(ids.insert(&item.id), "invalid_package: duplicate item ID");
    }
    Ok(StandardSnapshot {
        standard_id: source.standard_id,
        version: source.version.clone(),
        items,
    })
}

fn parse_cwe(
    source: &StandardSource,
    files: &BTreeMap<String, Vec<u8>>,
    cancelled: &dyn Fn() -> bool,
) -> Result<Vec<StandardItem>> {
    use quick_xml::{events::Event, reader::Reader, XmlVersion};
    let key = format!("raw/cwec_v{}.xml", source.version);
    let bytes = files
        .get(&key)
        .ok_or_else(|| anyhow::anyhow!("invalid_package: missing CWE XML"))?;
    std::str::from_utf8(bytes).map_err(|_| anyhow::anyhow!("invalid_format: CWE must be UTF-8"))?;
    let mut reader = Reader::from_reader(bytes.as_slice());
    reader.config_mut().enable_all_checks(true);
    reader.config_mut().expand_empty_elements = true;
    let mut stack: Vec<Vec<u8>> = Vec::new();
    let mut root_seen = false;
    let mut weaknesses_seen = false;
    let mut items = Vec::new();
    let mut current: Option<StandardItem> = None;
    let mut description_depth = None;
    let mut description_seen = false;
    let mut declaration_seen = false;
    loop {
        ensure!(!cancelled(), "cancelled");
        match reader
            .read_event()
            .map_err(|_| anyhow::anyhow!("invalid_format: malformed CWE XML"))?
        {
            Event::DocType(_) => {
                bail!("invalid_package: XML DTD and external entities are forbidden")
            }
            Event::PI(_) => bail!("invalid_package: XML processing instructions are forbidden"),
            Event::Decl(decl) => {
                ensure!(
                    !root_seen && !declaration_seen,
                    "invalid_format: XML declaration location"
                );
                declaration_seen = true;
                if let Some(encoding) = decl.encoding() {
                    let encoding =
                        encoding.map_err(|_| anyhow::anyhow!("invalid_format: XML encoding"))?;
                    ensure!(
                        encoding.eq_ignore_ascii_case(b"utf-8")
                            || encoding.eq_ignore_ascii_case(b"us-ascii"),
                        "invalid_format: unsupported XML encoding"
                    );
                }
            }
            Event::Start(start) => {
                stack.push(start.name().as_ref().to_vec());
                ensure!(stack.len() <= 64, "size_limit: XML depth");
                let mut attrs = BTreeMap::new();
                for attribute in start.attributes() {
                    ensure!(attrs.len() < 64, "size_limit: XML attributes");
                    let attribute =
                        attribute.map_err(|_| anyhow::anyhow!("invalid_format: XML attribute"))?;
                    let name = std::str::from_utf8(attribute.key.as_ref())
                        .map_err(|_| anyhow::anyhow!("invalid_format: XML attribute name"))?;
                    let value = attribute
                        .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
                        .map_err(|_| anyhow::anyhow!("invalid_package: XML attribute entity"))?;
                    attrs.insert(name.to_owned(), value.into_owned());
                }
                let name = start.name();
                let name = name.as_ref();
                if stack.len() == 1 {
                    ensure!(
                        !root_seen && name == b"Weakness_Catalog",
                        "invalid_format: CWE root"
                    );
                    root_seen = true;
                    ensure!(
                        attrs.get("Version") == Some(&source.version),
                        "version_mismatch: CWE Version"
                    );
                    ensure!(
                        attrs.get("xmlns").map(String::as_str)
                            == Some("http://cwe.mitre.org/cwe-7"),
                        "unsupported_format: CWE namespace"
                    );
                } else if name == b"Weaknesses" {
                    ensure!(
                        stack.len() == 2 && !weaknesses_seen,
                        "invalid_format: Weaknesses container"
                    );
                    weaknesses_seen = true;
                } else if name == b"Weakness" {
                    ensure!(
                        stack.len() == 3 && stack[1] == b"Weaknesses" && current.is_none(),
                        "invalid_format: Weakness location"
                    );
                    ensure!(items.len() < MAX_STANDARD_ITEMS, "size_limit: CWE items");
                    let id = attrs
                        .get("ID")
                        .ok_or_else(|| anyhow::anyhow!("invalid_format: CWE ID"))?;
                    ensure!(valid_cwe_number(id), "invalid_format: CWE ID");
                    let title = attrs
                        .get("Name")
                        .ok_or_else(|| anyhow::anyhow!("invalid_format: CWE Name"))?;
                    ensure!(
                        !title.trim().is_empty() && title.len() <= 4096,
                        "invalid_format: CWE Name"
                    );
                    current = Some(StandardItem {
                        id: format!("CWE-{id}"),
                        title: title.clone(),
                        description: String::new(),
                        levels: Vec::new(),
                        cwe_mappings: Vec::new(),
                        relationships: Vec::new(),
                        verification_status: super::ReviewStatus::NotVerified,
                    });
                    description_seen = false;
                } else if current.is_some() && name == b"Description" && stack.len() == 4 {
                    ensure!(
                        !description_seen,
                        "invalid_format: duplicate CWE Description"
                    );
                    description_depth = Some(stack.len());
                    description_seen = true;
                } else if name == b"Related_Weakness"
                    && stack.len() == 5
                    && stack[3] == b"Related_Weaknesses"
                {
                    let item = current
                        .as_mut()
                        .ok_or_else(|| anyhow::anyhow!("invalid_format: CWE relationship"))?;
                    let target = attrs
                        .get("CWE_ID")
                        .ok_or_else(|| anyhow::anyhow!("invalid_format: CWE relationship ID"))?;
                    let nature = attrs.get("Nature").ok_or_else(|| {
                        anyhow::anyhow!("invalid_format: CWE relationship nature")
                    })?;
                    ensure!(
                        valid_cwe_number(target) && !nature.is_empty() && nature.len() <= 64,
                        "invalid_format: CWE relationship"
                    );
                    ensure!(
                        item.relationships.len() < 1000,
                        "size_limit: CWE relationships"
                    );
                    item.relationships.push(CweRelationship {
                        cwe_id: format!("CWE-{target}"),
                        nature: nature.clone(),
                        view_id: attrs.get("View_ID").cloned(),
                    });
                }
            }
            Event::End(end) => {
                if description_depth == Some(stack.len()) {
                    description_depth = None;
                }
                if end.name().as_ref() == b"Weakness" && stack.len() == 3 {
                    let item = current
                        .take()
                        .ok_or_else(|| anyhow::anyhow!("invalid_format: CWE Weakness"))?;
                    ensure!(
                        description_seen && !item.description.trim().is_empty(),
                        "invalid_format: CWE Description missing"
                    );
                    items.push(item);
                }
                ensure!(stack.pop().is_some(), "invalid_format: unbalanced CWE XML");
            }
            Event::Text(text) => {
                let text = text
                    .xml_content(XmlVersion::Implicit1_0)
                    .map_err(|_| anyhow::anyhow!("invalid_format: XML text"))?;
                if stack.is_empty() {
                    ensure!(
                        text.trim().is_empty(),
                        "invalid_format: text outside XML root"
                    );
                }
                if description_depth.is_some() {
                    append_cwe_description(&mut current, &text)?;
                }
            }
            Event::CData(text) => {
                ensure!(!stack.is_empty(), "invalid_format: CDATA outside XML root");
                let text = text
                    .decode()
                    .map_err(|_| anyhow::anyhow!("invalid_format: XML CDATA"))?;
                if description_depth.is_some() {
                    append_cwe_description(&mut current, &text)?;
                }
            }
            Event::GeneralRef(entity) => {
                ensure!(!stack.is_empty(), "invalid_format: XML entity outside root");
                let name = entity
                    .decode()
                    .map_err(|_| anyhow::anyhow!("invalid_format: XML entity"))?;
                let text = match name.as_ref() {
                    "lt" => "<".into(),
                    "gt" => ">".into(),
                    "amp" => "&".into(),
                    "apos" => "'".into(),
                    "quot" => "\"".into(),
                    _ => entity
                        .resolve_char_ref()
                        .map_err(|_| anyhow::anyhow!("invalid_package: XML entity"))?
                        .ok_or_else(|| anyhow::anyhow!("invalid_package: unknown XML entity"))?
                        .to_string(),
                };
                if description_depth.is_some() {
                    append_cwe_description(&mut current, &text)?;
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    ensure!(
        root_seen && weaknesses_seen && stack.is_empty() && current.is_none(),
        "invalid_format: incomplete CWE XML"
    );
    Ok(items)
}

fn valid_cwe_number(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 6
        && value.as_bytes()[0] != b'0'
        && value.bytes().all(|b| b.is_ascii_digit())
}

fn append_cwe_description(item: &mut Option<StandardItem>, text: &str) -> Result<()> {
    if let Some(item) = item {
        ensure!(
            item.description.len() + text.len() <= 64 * 1024,
            "size_limit: CWE description"
        );
        item.description.push_str(text);
    }
    Ok(())
}

fn parse_asvs(
    source: &StandardSource,
    files: &BTreeMap<String, Vec<u8>>,
    cancelled: &dyn Fn() -> bool,
) -> Result<Vec<StandardItem>> {
    let file = source
        .files
        .iter()
        .find(|f| f.name.ends_with(".json"))
        .ok_or_else(|| anyhow::anyhow!("invalid_package: missing ASVS file"))?;
    let bytes = files
        .get(&file.name)
        .ok_or_else(|| anyhow::anyhow!("invalid_package: missing ASVS file"))?;
    let root: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| anyhow::anyhow!("invalid_format: ASVS JSON"))?;
    ensure!(
        root.get("Version").and_then(|v| v.as_str()) == Some(source.version.as_str()),
        "version_mismatch: ASVS Version"
    );
    let requirements = root
        .get("Requirements")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("invalid_format: ASVS Requirements"))?;
    let mut items = Vec::new();
    walk_asvs(requirements, source, 0, &mut items, cancelled)?;
    Ok(items)
}

fn walk_asvs(
    nodes: &[serde_json::Value],
    source: &StandardSource,
    depth: usize,
    items: &mut Vec<StandardItem>,
    cancelled: &dyn Fn() -> bool,
) -> Result<()> {
    static ID: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^V[1-9]\d{0,2}\.[1-9]\d{0,2}\.[1-9]\d{0,2}$").unwrap());
    ensure!(depth <= 64, "size_limit: ASVS depth");
    // Official 4.0.3 retains empty architecture sections (V1.3 / V1.13).
    // They contain no requirements; the complete parsed list must still be nonempty.
    for node in nodes {
        ensure!(!cancelled(), "cancelled");
        if let Some(children) = node.get("Items") {
            let children = children
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("invalid_format: ASVS Items"))?;
            walk_asvs(children, source, depth + 1, items, cancelled)?;
            continue;
        }
        let id = node
            .get("Shortcode")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("invalid_format: ASVS Shortcode"))?;
        let description = node
            .get("Description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("invalid_format: ASVS Description"))?;
        ensure!(
            ID.is_match(id) && !description.trim().is_empty() && description.len() <= 32 * 1024,
            "invalid_format: ASVS requirement"
        );
        let levels = if source.version == "4.0.3" {
            let mut levels = Vec::new();
            for n in 1..=3 {
                let required = node
                    .get(format!("L{n}"))
                    .and_then(|v| v.get("Required"))
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| {
                        anyhow::anyhow!("invalid_format: ASVS level Required must be boolean")
                    })?;
                if required {
                    levels.push(n);
                }
            }
            levels
        } else {
            let min = match node.get("L").and_then(|v| v.as_str()) {
                Some("1") => 1,
                Some("2") => 2,
                Some("3") => 3,
                _ => bail!("invalid_format: ASVS minimum L must be 1, 2 or 3"),
            };
            (min..=3).collect()
        };
        ensure!(items.len() < MAX_STANDARD_ITEMS, "size_limit: ASVS items");
        items.push(StandardItem {
            id: id.into(),
            title: id.into(),
            description: description.into(),
            levels,
            cwe_mappings: Vec::new(),
            relationships: Vec::new(),
            verification_status: super::ReviewStatus::NotVerified,
        });
    }
    Ok(())
}

fn parse_markdown(
    source: &StandardSource,
    files: &BTreeMap<String, Vec<u8>>,
    cancelled: &dyn Fn() -> bool,
) -> Result<Vec<StandardItem>> {
    static HEADING: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^#\s+((?:A\d{2}|API\d{1,2}):(\d{4}))\b").unwrap());
    static CWE_LINK: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"\[CWE-(\d{1,6})[^\]]*\]\(https://cwe\.mitre\.org/data/definitions/(\d{1,6})\.html\)",
        )
        .unwrap()
    });
    let mut items = Vec::new();
    let mut expected_ids = BTreeSet::new();
    for file in source.files.iter().filter(|f| f.item_id.is_some()) {
        ensure!(!cancelled(), "cancelled");
        let expected = file.item_id.as_deref().unwrap();
        ensure!(
            expected_ids.insert(expected),
            "invalid_package: duplicate category ID"
        );
        let bytes = files
            .get(&file.name)
            .ok_or_else(|| anyhow::anyhow!("invalid_package: missing category"))?;
        let text = std::str::from_utf8(bytes)
            .map_err(|_| anyhow::anyhow!("invalid_format: expected UTF-8 Markdown"))?;
        let heading = text
            .lines()
            .find(|l| l.starts_with("# "))
            .ok_or_else(|| anyhow::anyhow!("invalid_format: missing category heading"))?;
        let capture = HEADING
            .captures(heading)
            .ok_or_else(|| anyhow::anyhow!("invalid_format: category heading"))?;
        ensure!(
            &capture[2] == source.version,
            "version_mismatch: category year"
        );
        ensure!(
            &capture[1] == expected,
            "invalid_package: category ID mismatch"
        );
        let mut seen = BTreeSet::new();
        let mut mappings = Vec::new();
        for section in text.split("\n## ") {
            ensure!(!cancelled(), "cancelled");
            let label = section.lines().next().unwrap_or("").trim();
            if !matches!(label, "List of Mapped CWEs" | "Related CWEs") {
                continue;
            }
            for capture in CWE_LINK.captures_iter(section) {
                ensure!(!cancelled(), "cancelled");
                if capture[1] != capture[2] {
                    continue;
                }
                let id = format!("CWE-{}", &capture[1]);
                if seen.insert(id.clone()) {
                    mappings.push(CweMapping {
                        cwe_id: id,
                        source_url: file.url.clone(),
                        source_revision: source.source_revision.clone(),
                        section: label.into(),
                    });
                }
            }
        }
        items.push(StandardItem {
            id: expected.into(),
            title: heading
                .trim_start_matches('#')
                .trim()
                .split("![")
                .next()
                .unwrap_or("")
                .trim()
                .into(),
            description: text.chars().take(8192).collect(),
            levels: Vec::new(),
            cwe_mappings: mappings,
            relationships: Vec::new(),
            verification_status: super::ReviewStatus::NotVerified,
        });
    }
    ensure!(
        items.len() == 10,
        "invalid_package: expected exactly ten categories"
    );
    let expected: BTreeSet<_> = (1..=10)
        .map(|n| {
            if source.standard_id == StandardId::OwaspTop10 {
                format!("A{n:02}:{}", source.version)
            } else {
                format!("API{n}:{}", source.version)
            }
        })
        .collect();
    ensure!(
        expected_ids
            .iter()
            .map(|s| s.to_string())
            .collect::<BTreeSet<_>>()
            == expected,
        "invalid_package: category ID set"
    );
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn cwe_fixture(body: &str) -> (StandardSource, BTreeMap<String, Vec<u8>>) {
        let source = standard_source(StandardId::Cwe, "4.20").unwrap();
        (
            source,
            BTreeMap::from([
                ("raw/cwec_v4.20.xml".into(), body.as_bytes().to_vec()),
                ("LICENSE".into(), b"Synthetic MITRE terms fixture".to_vec()),
            ]),
        )
    }

    fn cwe_xml(inner: &str) -> String {
        format!("<Weakness_Catalog xmlns=\"http://cwe.mitre.org/cwe-7\" Version=\"4.20\"><Weaknesses>{inner}</Weaknesses></Weakness_Catalog>")
    }

    #[test]
    fn security_cwe_version_relationships_and_builtin_entities() {
        let xml = cwe_xml("<Weakness ID=\"79\" Name=\"Synthetic &amp; safe\"><Description>Text &lt;x&gt; &#65; &#x42;<![CDATA[ &raw; ]]></Description><Related_Weaknesses><Related_Weakness Nature=\"ChildOf\" CWE_ID=\"20\" View_ID=\"1000\"/></Related_Weaknesses></Weakness>");
        let (source, files) = cwe_fixture(&xml);
        let parsed = parse_standard(&source, &files, &|| false).unwrap();
        assert_eq!(parsed.version, "4.20");
        assert_eq!(parsed.items[0].id, "CWE-79");
        assert_eq!(parsed.items[0].title, "Synthetic & safe");
        assert_eq!(parsed.items[0].description, "Text <x> A B &raw; ");
        assert_eq!(parsed.items[0].relationships[0].cwe_id, "CWE-20");
        assert_eq!(parsed.items[0].relationships[0].nature, "ChildOf");
        assert_eq!(
            parsed.items[0].relationships[0].view_id.as_deref(),
            Some("1000")
        );
        let wrong = xml.replace("Version=\"4.20\"", "Version=\"4.19\"");
        assert!(parse_standard(&source, &cwe_fixture(&wrong).1, &|| false)
            .unwrap_err()
            .to_string()
            .contains("version_mismatch"));
    }

    #[test]
    fn security_cwe_rejects_dtd_external_unknown_entities_and_malformed_structure() {
        let item =
            "<Weakness ID=\"79\" Name=\"Synthetic\"><Description>text</Description></Weakness>";
        for xml in [
            format!(
                "<!DOCTYPE x [<!ENTITY evil SYSTEM \"file:///must-not-read\">]>{}",
                cwe_xml(item)
            ),
            format!(
                "<!DOCTYPE x SYSTEM \"https://must-not-fetch.invalid\">{}",
                cwe_xml(item)
            ),
            cwe_xml(&item.replace("text", "&unknown;")),
            cwe_xml(&item.replace("Synthetic", "&unknown;")),
            cwe_xml(&format!("{item}{item}")),
            cwe_xml("<Weakness ID=\"x\" Name=\"Bad\"><Description>x</Description></Weakness>"),
            cwe_xml("<Weakness ID=\"79\" Name=\"Missing description\"/>"),
            cwe_xml(item).replace("</Weaknesses>", "</Other>"),
            format!("{}{}", cwe_xml(item), cwe_xml(item)),
        ] {
            let (source, files) = cwe_fixture(&xml);
            assert!(parse_standard(&source, &files, &|| false).is_err());
        }
    }

    #[test]
    fn security_cwe_depth_item_limits_and_cancellation() {
        let inner = format!(
            "<Weakness ID=\"79\" Name=\"Nested\"><Description>{}text{}</Description></Weakness>",
            "<p>".repeat(60),
            "</p>".repeat(60)
        );
        let (source, files) = cwe_fixture(&cwe_xml(&inner));
        assert!(
            parse_standard(&source, &files, &|| false).is_ok(),
            "depth exactly 64"
        );
        let too_deep = inner.replace("text", "<p>text</p>");
        assert!(
            parse_standard(&source, &cwe_fixture(&cwe_xml(&too_deep)).1, &|| false)
                .unwrap_err()
                .to_string()
                .contains("size_limit")
        );
        let many = (1..=MAX_STANDARD_ITEMS + 1).map(|n| format!("<Weakness ID=\"{n}\" Name=\"Synthetic\"><Description>x</Description></Weakness>")).collect::<String>();
        assert!(
            parse_standard(&source, &cwe_fixture(&cwe_xml(&many)).1, &|| false)
                .unwrap_err()
                .to_string()
                .contains("size_limit")
        );
        let calls = std::cell::Cell::new(0);
        let error = parse_standard(&source, &files, &|| {
            calls.set(calls.get() + 1);
            calls.get() > 10
        })
        .unwrap_err();
        assert!(error.to_string().contains("cancelled"));
    }

    fn asvs_fixture(
        version: &str,
        leaves: serde_json::Value,
    ) -> (StandardSource, BTreeMap<String, Vec<u8>>) {
        let source = standard_source(StandardId::OwaspAsvs, version).unwrap();
        let file = source
            .files
            .iter()
            .find(|f| f.name.ends_with(".json"))
            .unwrap();
        let data = serde_json::json!({"Version":version,"Requirements":[{"Shortcode":"V1","Items":[{"Shortcode":"V1.1","Items":leaves}]}]});
        let files = BTreeMap::from([
            (file.name.clone(), serde_json::to_vec(&data).unwrap()),
            ("LICENSE".into(), b"Synthetic licensed fixture".to_vec()),
        ]);
        (source, files)
    }

    #[test]
    fn security_asvs_version_specific_levels() {
        let (source, files) = asvs_fixture(
            "4.0.3",
            serde_json::json!([
                {"Shortcode":"V1.1.1","Description":"Synthetic one","L1":{"Required":true},"L2":{"Required":false},"L3":{"Required":true}},
                {"Shortcode":"V1.1.2","Description":"Synthetic two","L1":{"Required":false},"L2":{"Required":true},"L3":{"Required":true}}
            ]),
        );
        let parsed = parse_standard(&source, &files, &|| false).unwrap();
        assert_eq!(
            parsed.items[0].levels,
            [1, 3],
            "do not infer cumulative levels in 4.0.3"
        );
        assert_eq!(parsed.items[1].levels, [2, 3]);
        let (source, files) = asvs_fixture(
            "5.0.0",
            serde_json::json!([
                {"Shortcode":"V1.1.1","Description":"Synthetic one","L":"1"},
                {"Shortcode":"V1.1.2","Description":"Synthetic two","L":"2"},
                {"Shortcode":"V1.1.3","Description":"Synthetic three","L":"3"}
            ]),
        );
        let parsed = parse_standard(&source, &files, &|| false).unwrap();
        assert_eq!(parsed.items[0].levels, [1, 2, 3]);
        assert_eq!(parsed.items[1].levels, [2, 3]);
        assert_eq!(parsed.items[2].levels, [3]);
        assert_eq!(
            parsed
                .items_for_level(super::super::policy::AsvsLevel::L2)
                .count(),
            2
        );
        assert!(parsed.items.iter().all(|i| i.cwe_mappings.is_empty()));
        assert!(parsed
            .items
            .iter()
            .all(|i| i.verification_status == super::super::ReviewStatus::NotVerified));
    }

    #[test]
    fn security_asvs_rejects_wrong_version_missing_fields_and_duplicate_ids() {
        for (version, leaves) in [
            (
                "4.0.3",
                serde_json::json!([{"Shortcode":"V1.1.1","Description":"Missing levels"}]),
            ),
            (
                "5.0.0",
                serde_json::json!([{"Shortcode":"V1.1.1","Description":"Missing L"}]),
            ),
            (
                "5.0.0",
                serde_json::json!([{"Shortcode":"V1.1.1","Description":"Bad L","L":"4"}]),
            ),
            (
                "5.0.0",
                serde_json::json!([{"Shortcode":"V1.1.1","Description":"One","L":"1"},{"Shortcode":"V1.1.1","Description":"Duplicate","L":"1"}]),
            ),
            (
                "5.0.0",
                serde_json::json!([{"Shortcode":"made-up","Description":"Bad official ID","L":"1"}]),
            ),
        ] {
            let (source, files) = asvs_fixture(version, leaves);
            assert!(parse_standard(&source, &files, &|| false).is_err());
        }
        let (source, mut files) = asvs_fixture("5.0.0", serde_json::json!([]));
        let key = source
            .files
            .iter()
            .find(|f| f.name.ends_with(".json"))
            .unwrap()
            .name
            .clone();
        files.insert(key, b"{\"Version\":\"4.0.3\",\"Requirements\":[]}".to_vec());
        assert!(parse_standard(&source, &files, &|| false)
            .unwrap_err()
            .to_string()
            .contains("version_mismatch"));
    }

    #[test]
    #[ignore = "requires explicitly downloaded official fixtures; never downloads during tests"]
    fn security_standards_official_repository_snapshot() {
        use sha2::{Digest, Sha256};
        let root = std::env::var_os("SPECLINK_SECURITY_OFFICIAL_FIXTURES")
            .expect("set the official fixture directory");
        for source in source_catalog().unwrap() {
            let dir = std::path::Path::new(&root).join(format!(
                "{}_{}",
                source.standard_id.as_str(),
                source.version
            ));
            let mut files: BTreeMap<_, _> = source
                .files
                .iter()
                .map(|file| {
                    let bytes = std::fs::read(
                        dir.join(file.name.strip_prefix("raw/").unwrap_or(&file.name)),
                    )
                    .unwrap();
                    if let Some(hash) = &file.sha256 {
                        assert_eq!(&format!("{:x}", Sha256::digest(&bytes)), hash);
                    }
                    (file.name.clone(), bytes)
                })
                .collect();
            if source.standard_id == StandardId::Cwe {
                let name = format!("cwec_v{}.xml", source.version);
                files.insert(
                    format!("raw/{name}"),
                    std::fs::read(dir.join(name)).unwrap(),
                );
            }
            let parsed = parse_standard(&source, &files, &|| false).unwrap();
            if source.standard_id == StandardId::Cwe {
                assert!(parsed.items.iter().any(|i| i.id == "CWE-79"));
                assert!(parsed.items.iter().any(|i| i.id == "CWE-798"));
            } else if source.standard_id == StandardId::OwaspAsvs {
                assert!(!parsed.items.is_empty());
                assert!(
                    parsed
                        .items
                        .iter()
                        .all(|i| i.id.starts_with('V')
                            && i.levels.iter().all(|n| (1..=3).contains(n)))
                );
            } else if source.standard_id == StandardId::OwaspApiTop10 {
                assert_eq!(parsed.items.len(), 10);
                assert!(parsed.items.iter().all(|item| item.cwe_mappings.is_empty()));
            } else {
                assert_eq!(parsed.items.len(), 10);
                assert!(parsed
                    .items
                    .iter()
                    .all(|item| !item.cwe_mappings.is_empty()));
            }
        }
    }

    // Synthetic text, not copies of the official requirements or risk descriptions.
    fn markdown_fixture(source: &StandardSource) -> BTreeMap<String, Vec<u8>> {
        source.files.iter().map(|file| {
            let text = match file.item_id.as_deref() {
                Some(id) => format!("# {id} Synthetic category\n\n## Description\nCWE-999 is a narrative mention.\n\n## List of Mapped CWEs\n[CWE-79 Synthetic weakness](https://cwe.mitre.org/data/definitions/79.html)\n\n## Other\n[CWE-89 Other](https://cwe.mitre.org/data/definitions/89.html)\n"),
                None => "Creative Commons Attribution-ShareAlike 4.0 International Public License\nSynthetic fixture.\n".into(),
            };
            (file.name.clone(), text.into_bytes())
        }).collect()
    }

    #[test]
    fn security_standards_parse_ten_categories_with_versioned_identity() {
        for (id, versions) in [
            (StandardId::OwaspTop10, ["2021", "2025"]),
            (StandardId::OwaspApiTop10, ["2019", "2023"]),
        ] {
            let mut keys = std::collections::BTreeSet::new();
            for version in versions {
                let source = standard_source(id, version).unwrap();
                let parsed =
                    parse_standard(&source, &markdown_fixture(&source), &|| false).unwrap();
                assert_eq!(parsed.standard_id, id);
                assert_eq!(parsed.version, version);
                assert_eq!(parsed.items.len(), 10);
                for item in &parsed.items {
                    assert!(keys.insert(parsed.item_key(item)));
                    assert_eq!(
                        item.cwe_mappings
                            .iter()
                            .map(|m| m.cwe_id.as_str())
                            .collect::<Vec<_>>(),
                        ["CWE-79"]
                    );
                    assert_eq!(item.cwe_mappings[0].source_revision, source.source_revision);
                    assert_eq!(item.cwe_mappings[0].section, "List of Mapped CWEs");
                }
            }
            assert_eq!(keys.len(), 20);
        }
    }

    #[test]
    fn security_standards_reject_incomplete_duplicate_wrong_version_and_cancelled() {
        let source = standard_source(StandardId::OwaspTop10, "2025").unwrap();
        let files = markdown_fixture(&source);
        let mut missing = files.clone();
        missing.remove(&source.files[0].name);
        assert!(parse_standard(&source, &missing, &|| false).is_err());
        let mut wrong = files.clone();
        let first = source.files.iter().find(|f| f.item_id.is_some()).unwrap();
        wrong.insert(first.name.clone(), b"# A01:2021 Wrong version\n".to_vec());
        assert!(parse_standard(&source, &wrong, &|| false)
            .unwrap_err()
            .to_string()
            .contains("version_mismatch"));
        let mut duplicate = source.clone();
        let categories: Vec<_> = duplicate
            .files
            .iter()
            .enumerate()
            .filter(|(_, f)| f.item_id.is_some())
            .map(|(i, _)| i)
            .collect();
        duplicate.files[categories[1]].item_id = duplicate.files[categories[0]].item_id.clone();
        assert!(parse_standard(&duplicate, &files, &|| false).is_err());
        assert!(parse_standard(&source, &files, &|| true)
            .unwrap_err()
            .to_string()
            .contains("cancelled"));
        assert!(standard_source(StandardId::OwaspTop10, "latest").is_err());
    }

    #[test]
    fn security_standards_no_narrative_mapping_or_cwe_install_dependency() {
        let source = standard_source(StandardId::OwaspApiTop10, "2023").unwrap();
        let mut files = markdown_fixture(&source);
        for file in source.files.iter().filter(|f| f.item_id.is_some()) {
            files.insert(file.name.clone(), format!("# {} Synthetic\n\n## References\n[CWE-79 Mention](https://cwe.mitre.org/data/definitions/79.html)\n", file.item_id.as_deref().unwrap()).into_bytes());
        }
        let parsed = parse_standard(&source, &files, &|| false).unwrap();
        assert!(parsed.items.iter().all(|i| i.cwe_mappings.is_empty()));
        let top = standard_source(StandardId::OwaspTop10, "2025").unwrap();
        assert_eq!(
            parse_standard(&top, &markdown_fixture(&top), &|| false)
                .unwrap()
                .items
                .len(),
            10
        );
    }

    #[test]
    fn security_mapping_uses_only_selected_version_witnesses_and_asvs_stays_unverified() {
        use crate::security::{
            data::{InstalledStandard, IntegrityKind, ValidatedStandard},
            input::SourceDocument,
            policy::SecurityPolicy,
            rules::RuleSet,
            ReviewStatus,
        };
        let mut source = standard_source(StandardId::OwaspTop10, "2025").unwrap();
        source.files.iter_mut().for_each(|f| f.sha256 = None);
        let mut files = markdown_fixture(&source);
        let first = source
            .files
            .iter()
            .find(|f| f.item_id.as_deref() == Some("A01:2025"))
            .unwrap();
        files.insert(first.name.clone(),b"# A01:2025 Synthetic category\n## List of Mapped CWEs\n[CWE-798](https://cwe.mitre.org/data/definitions/798.html)\n".to_vec());
        let data =
            ValidatedStandard::new(source, files, IntegrityKind::Imported, &|| false).unwrap();
        let installed = InstalledStandard {
            metadata: data
                .metadata("generation".into(), "2026-10-17T00:00:00Z".into())
                .unwrap(),
            snapshot: data.snapshot().clone(),
        };
        let mut policy: SecurityPolicy = serde_yaml::from_str(
            "enabled: true\nstandards:\n  owasp_top10: {enabled: true, version: '2025'}\n",
        )
        .unwrap();
        let mut results = RuleSet::builtin()
            .unwrap()
            .review_code(
                &SourceDocument::new("app.ts".into(), "const token = 'real-value-123';".into()),
                &[],
                &|| None,
            )
            .unwrap();
        let assessments = associate_standards(&policy, &[installed], &mut results, &|| None);
        let top = assessments
            .iter()
            .find(|s| s.standard_id == StandardId::OwaspTop10)
            .unwrap();
        assert_eq!(top.version.as_deref(), Some("2025"));
        assert_eq!(top.controls[0].status, ReviewStatus::Finding);
        let reference = &results
            .iter()
            .flat_map(|r| &r.findings)
            .find(|f| f.rule_id == "SL-SEC-001")
            .unwrap()
            .references[0];
        assert_eq!(reference.version, "2025");
        assert_eq!(reference.control_id, "A01:2025");
        assert!(reference
            .source_url
            .starts_with("https://raw.githubusercontent.com/OWASP/Top10/"));
        assert_eq!(reference.section, "List of Mapped CWEs");
        assert!(!top.controls.iter().any(|c| c.status == ReviewStatus::Pass));
        policy.standards.owasp_top10.version = Some("2021".into());
        let missing = associate_standards(&policy, &[], &mut results, &|| None);
        assert!(missing
            .iter()
            .any(|s| s.standard_id == StandardId::OwaspTop10
                && s.reason.as_deref() == Some("missing_data")));
        assert!(results
            .iter()
            .flat_map(|r| &r.findings)
            .all(|f| f.references.is_empty()));
        policy.standards.owasp_top10.enabled = false;
        policy.standards.owasp_asvs.enabled = true;
        policy.standards.owasp_asvs.version = Some("5.0.0".into());
        let mut source = standard_source(StandardId::OwaspAsvs, "5.0.0").unwrap();
        source.files.iter_mut().for_each(|f| f.sha256 = None);
        let files=std::collections::BTreeMap::from([(source.files.iter().find(|f|f.name.ends_with(".json")).unwrap().name.clone(),br#"{"Version":"5.0.0","Requirements":[{"Shortcode":"V1.1.1","Description":"Synthetic","L":"1"},{"Shortcode":"V1.1.2","Description":"Synthetic higher level","L":"3"}]}"#.to_vec()),("LICENSE".into(),b"Synthetic license".to_vec())]);
        let data =
            ValidatedStandard::new(source, files, IntegrityKind::Imported, &|| false).unwrap();
        let installed = InstalledStandard {
            metadata: data.metadata("generation".into(), "time".into()).unwrap(),
            snapshot: data.snapshot().clone(),
        };
        let assessment = associate_standards(&policy, &[installed], &mut results, &|| None)
            .into_iter()
            .find(|s| s.standard_id == StandardId::OwaspAsvs)
            .unwrap();
        assert_eq!(assessment.controls.len(), 1);
        assert_eq!(assessment.controls[0].status, ReviewStatus::NotVerified);
    }
    #[test]
    fn security_mapping_cancel_and_deadline_interrupt_inside_one_control() {
        use crate::security::{
            data::{InstalledStandard, IntegrityKind, ValidatedStandard},
            input::SourceDocument,
            policy::SecurityPolicy,
            rules::RuleSet,
            ReviewStatus,
        };
        use std::cell::Cell;
        let (source, files) = cwe_fixture(&cwe_xml(
            "<Weakness ID=\"798\" Name=\"Synthetic\"><Description>Fixture</Description></Weakness>",
        ));
        let validated =
            ValidatedStandard::new(source, files, IntegrityKind::Imported, &|| false).unwrap();
        let installed = InstalledStandard {
            metadata: validated
                .metadata("generation".into(), "time".into())
                .unwrap(),
            snapshot: validated.snapshot().clone(),
        };
        let policy: SecurityPolicy = serde_yaml::from_str(
            "enabled: true\nstandards:\n  cwe: {enabled: true, version: '4.20'}\n",
        )
        .unwrap();
        let rule_results = RuleSet::builtin()
            .unwrap()
            .review_code(
                &SourceDocument::new("app.ts".into(), "const token = 'production-value';".into()),
                &[],
                &|| None,
            )
            .unwrap();
        let finding = rule_results
            .into_iter()
            .find(|r| !r.findings.is_empty())
            .unwrap();
        for reason in ["cancelled", "budget_exceeded"] {
            for limit in [0, 16] {
                let calls = Cell::new(0);
                let mut results = vec![finding.clone(); 1000];
                let assessments = associate_standards(
                    &policy,
                    std::slice::from_ref(&installed),
                    &mut results,
                    &|| {
                        calls.set(calls.get() + 1);
                        (calls.get() > limit).then_some(reason)
                    },
                );
                let cwe = assessments
                    .iter()
                    .find(|s| s.standard_id == StandardId::Cwe)
                    .unwrap();
                assert_eq!(cwe.status, ReviewStatus::NotVerified);
                assert_eq!(cwe.reason.as_deref(), Some(reason));
                assert!(cwe.controls.iter().all(|c| c.status != ReviewStatus::Pass));
                assert!(calls.get() < 100, "must stop without walking all findings");
                assert!(assessments
                    .iter()
                    .filter(|s| s.standard_id != StandardId::Cwe)
                    .all(|s| s.status == ReviewStatus::Disabled));
            }
        }
    }
}
