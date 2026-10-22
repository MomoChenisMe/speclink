//! Approved offline guidance. Never follows links found in reference documents.
use super::{
    data::{sha256, StandardMetadata},
    standards::{source_catalog, valid_data_name, SourceFile, StandardId},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeRegistry {
    pub format_version: u8,
    pub standard_catalog: String,
    pub resources: Vec<KnowledgeResource>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeResource {
    pub id: String,
    pub role: String,
    pub applies_to: Vec<StandardId>,
    pub source_url: String,
    pub source_revision: String,
    pub files: Vec<SourceFile>,
    pub license_name: String,
    pub license_url: String,
    pub attribution: String,
}

#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct KnowledgeIdentity {pub standard_id:StandardId,pub version:String,pub data_sha256:String,pub source_revision:String,pub parser_version:String}
impl From<&StandardMetadata> for KnowledgeIdentity {
    fn from(m:&StandardMetadata)->Self {Self{standard_id:m.standard_id,version:m.version.clone(),data_sha256:m.sha256.clone(),source_revision:m.source_revision.clone(),parser_version:m.parser_version.clone()}}
}
pub fn snapshot_digest(project_key:&str,policy_digest:&str,files:&BTreeMap<String,String>,generations:&[StandardMetadata])->Result<String> {
    let mut identities=generations.iter().map(KnowledgeIdentity::from).collect::<Vec<_>>();
    identities.sort_by(|a,b|(&a.standard_id,&a.version).cmp(&(&b.standard_id,&b.version)));
    Ok(sha256(&serde_json::to_vec(&(project_key,policy_digest,files,identities))?))
}

pub fn snapshot_path(registry:&KnowledgeRegistry,id:StandardId,version:&str,name:&str)->String {
    if registry.files().any(|f|f.name==name) {
        if name=="raw/CheatSheetSeries_LICENSE.md" {"cheatsheets/LICENSE.md".into()}else{format!("cheatsheets/{}",name.trim_start_matches("raw/"))}
    }else{format!("standards/{}/{version}/{name}",id.as_str())}
}
pub fn snapshot_mappings(snapshots:&[super::standards::StandardSnapshot])->Result<BTreeMap<String,Vec<u8>>> {
    let mut files=BTreeMap::new();
    if let Some(cwe)=snapshots.iter().find(|s|s.standard_id==StandardId::Cwe) {
        let known:BTreeSet<_>=cwe.items.iter().map(|i|i.id.as_str()).collect();
        for snapshot in snapshots.iter().filter(|s|s.standard_id!=StandardId::Cwe) {
            let mapping:BTreeMap<_,_>=snapshot.items.iter().map(|item|(&item.id,item.cwe_mappings.iter().filter(|m|known.contains(m.cwe_id.as_str())).collect::<Vec<_>>())).collect();
            let prefix=match snapshot.standard_id {StandardId::OwaspTop10=>"owasp",StandardId::OwaspApiTop10=>"owasp_api",StandardId::OwaspAsvs=>"asvs",StandardId::Cwe=>unreachable!()};
            files.insert(format!("mappings/{prefix}_{}_cwe.json",snapshot.version),serde_json::to_vec_pretty(&mapping)?);
        }
    }
    Ok(files)
}
/// The complete allowlist comes from installed, validated generations, never from a KB manifest.
pub fn snapshot_hashes(generations:&[StandardMetadata],snapshots:&[super::standards::StandardSnapshot])->Result<BTreeMap<String,String>> {
    let registry=registry()?;let mut hashes=BTreeMap::new();
    for metadata in generations {
        ensure!(registry.status_digests(&metadata.files)?=="ready","missing_knowledge");
        let source=super::standards::standard_source(metadata.standard_id,&metadata.version)?;
        for (name,digest) in &metadata.files {
            let expanded_cwe=metadata.standard_id==StandardId::Cwe&&*name==format!("raw/cwec_v{}.xml",metadata.version);
            ensure!(expanded_cwe||source.files.iter().any(|f|f.name==*name)||registry.files().any(|f|f.name==*name),"invalid_knowledge: unselected source");
            let path=snapshot_path(&registry,metadata.standard_id,&metadata.version,name);
            if let Some(old)=hashes.insert(path,digest.clone()){ensure!(old==*digest,"integrity_error: knowledge generations");}
        }
    }
    for (path,bytes) in snapshot_mappings(snapshots)? {hashes.insert(path,sha256(&bytes));}
    Ok(hashes)
}
pub fn registry() -> Result<KnowledgeRegistry> {
    parse_registry(include_bytes!("../../assets/security/spec_registry.json"))
}
pub fn parse_registry(bytes: &[u8]) -> Result<KnowledgeRegistry> {
    let registry: KnowledgeRegistry = serde_json::from_slice(bytes)?;
    ensure!(
        registry.format_version == 1
            && registry.standard_catalog == "sources.json"
            && !registry.resources.is_empty(),
        "invalid_registry"
    );
    let mut names = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for resource in &registry.resources {
        ensure!(
            ids.insert(&resource.id)
                && resource.role == "guidance"
                && !resource.applies_to.is_empty(),
            "invalid_registry"
        );
        ensure!(
            resource.source_revision.len() == 40
                && resource
                    .source_revision
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit()),
            "invalid_registry: floating revision"
        );
        let base = format!(
            "https://raw.githubusercontent.com/OWASP/CheatSheetSeries/{}/",
            resource.source_revision
        );
        ensure!(
            resource.source_url
                == format!(
                    "https://github.com/OWASP/CheatSheetSeries/tree/{}",
                    resource.source_revision
                ),
            "invalid_registry: source"
        );
        ensure!(
            resource.license_name == "CC-BY-SA-4.0"
                && resource.license_url == format!("{base}LICENSE.md")
                && !resource.attribution.trim().is_empty(),
            "license_missing"
        );
        ensure!(
            !resource.files.is_empty()
                && resource.files.iter().any(|f| f.url == resource.license_url),
            "license_missing"
        );
        for file in &resource.files {
            let upstream = if file.name == "raw/CheatSheetSeries_LICENSE.md" {
                "LICENSE.md".to_owned()
            } else {
                format!(
                    "cheatsheets/{}",
                    file.name.strip_prefix("raw/").unwrap_or("")
                )
            };
            ensure!(
                valid_data_name(&file.name)
                    && names.insert(file.name.to_ascii_lowercase())
                    && file.url == format!("{base}{upstream}"),
                "invalid_registry: file"
            );
            ensure!(
                file.sha256.as_ref().is_some_and(
                    |hash| hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit())
                ),
                "invalid_registry: fingerprint"
            );
        }
    }
    Ok(registry)
}
impl KnowledgeRegistry {
    pub fn files(&self) -> impl Iterator<Item = &SourceFile> {
        self.resources.iter().flat_map(|r| &r.files)
    }
    pub fn status(&self, files: &BTreeMap<String, Vec<u8>>) -> Result<&'static str> {
        self.status_digests(
            &files
                .iter()
                .map(|(name, bytes)| (name.clone(), sha256(bytes)))
                .collect(),
        )
    }
    pub fn status_digests(&self, files: &BTreeMap<String, String>) -> Result<&'static str> {
        let catalog = source_catalog()?;
        for name in files.keys() {
            let expanded_cwe = name
                .strip_prefix("raw/cwec_v")
                .and_then(|n| n.strip_suffix(".xml"))
                .is_some_and(super::policy::valid_version);
            ensure!(
                self.files().any(|f| &f.name == name)
                    || catalog
                        .iter()
                        .any(|s| s.files.iter().any(|f| &f.name == name))
                    || expanded_cwe,
                "invalid_package: unexpected data file"
            );
        }
        let mut missing = false;
        for file in self.files() {
            if let Some(hash) = files.get(&file.name) {
                ensure!(
                    hash.eq_ignore_ascii_case(file.sha256.as_deref().unwrap()),
                    "integrity_error: guidance fingerprint"
                );
            } else {
                missing = true;
            }
        }
        Ok(if missing { "missing" } else { "ready" })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeManifest {
    pub format_version: u8,
    pub generated_by: String,
    pub project_key: String,
    pub policy_digest: String,
    pub knowledge_digest: String,
    pub files: BTreeMap<String, String>,
    pub data_generations: Vec<StandardMetadata>,
}
