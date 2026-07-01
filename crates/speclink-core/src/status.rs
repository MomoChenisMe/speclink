//! Artifact DAG status.

use crate::model::{self, Change};
use crate::schema::Schema;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ArtifactStatusJson {
    pub id: String,
    #[serde(rename = "outputPath")]
    pub output_path: String,
    pub status: String,
    #[serde(rename = "missingDeps", skip_serializing_if = "Vec::is_empty")]
    pub blocked_by: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct StatusReport {
    #[serde(rename = "changeName")]
    pub change_name: String,
    #[serde(rename = "schemaName")]
    pub schema_name: String,
    #[serde(rename = "isComplete")]
    pub is_complete: bool,
    #[serde(rename = "applyRequires")]
    pub apply_requires: Vec<String>,
    pub artifacts: Vec<ArtifactStatusJson>,
}

/// Dependency tier (longest chain to a root) for topological display ordering.
fn tier(schema: &Schema, id: &str, memo: &mut std::collections::HashMap<String, usize>) -> usize {
    if let Some(v) = memo.get(id) {
        return *v;
    }
    let t = match schema.artifact(id) {
        Some(a) if a.requires.is_empty() => 0,
        Some(a) => a.requires.iter().map(|r| tier(schema, r, memo) + 1).max().unwrap_or(0),
        None => 0,
    };
    memo.insert(id.to_string(), t);
    t
}

/// Artifacts in status display order (topological, alphabetical tiebreak).
fn display_order(schema: &Schema) -> Vec<&crate::schema::Artifact> {
    let mut memo = std::collections::HashMap::new();
    let mut arts: Vec<&crate::schema::Artifact> = schema.artifacts.iter().collect();
    arts.sort_by(|a, b| {
        let (ta, tb) = (tier(schema, a.id, &mut memo), tier(schema, b.id, &mut memo));
        ta.cmp(&tb).then_with(|| a.id.cmp(b.id))
    });
    arts
}

pub fn build(change: &Change, schema: &Schema) -> StatusReport {
    let statuses = model::artifact_statuses(schema, &change.dir);
    let mut artifacts = Vec::new();
    for a in display_order(schema) {
        let status = statuses
            .iter()
            .find(|(id, _)| id == a.id)
            .map(|(_, s)| s.as_str())
            .unwrap_or("blocked");
        let blocked_by = if status == "blocked" {
            model::blocked_by(schema, &change.dir, a.id)
        } else {
            Vec::new()
        };
        artifacts.push(ArtifactStatusJson {
            id: a.id.to_string(),
            output_path: a.output_path.to_string(),
            status: status.to_string(),
            blocked_by,
        });
    }
    StatusReport {
        change_name: change.name.clone(),
        schema_name: schema.name.to_string(),
        is_complete: model::is_complete(schema, &change.dir),
        apply_requires: schema.apply_requires.iter().map(|s| s.to_string()).collect(),
        artifacts,
    }
}
