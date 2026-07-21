//! Read-only server surfaces that are not Engine verbs: scope discovery,
//! canonical/archived documents, and desktop-aligned workspace search.

use crate::auth::{Binding, IdentityOnly};
use crate::error::ApiError;
use crate::state::AppState;
use crate::verb;
use axum::extract::{Path, Query, State};
use axum::http::header::ETAG;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use speclink_core::model::ChangeMeta;
use speclink_protocol::binding::ScopeRef;
use speclink_protocol::query::{
    ArchivedItem, ArchivedListResponse, ProjectScope, ScopesResponse, SearchHit, SearchResponse,
    SpecDocumentResponse,
};
use speclink_store::{Bundle, DocumentId};
use std::collections::{BTreeMap, BTreeSet, HashSet};

const SNIPPET_CONTEXT: usize = 30;

/// A scope's exported documents at one revision. `export` is the TeamStore's
/// enumeration seam and returns one consistent bundle for the bound repo.
struct ReadSnapshot {
    documents: BTreeMap<DocumentId, String>,
    etag: String,
}

impl From<Bundle> for ReadSnapshot {
    fn from(bundle: Bundle) -> Self {
        Self {
            documents: bundle
                .documents
                .into_iter()
                .map(|document| (document.doc, document.content))
                .collect(),
            etag: format!("\"{}\"", bundle.project_revision.0),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SearchQuery {
    #[serde(default)]
    q: String,
}

fn ok<T: Serialize>(dto: T, etag: &str) -> Response {
    ([(ETAG, etag.to_string())], Json(dto)).into_response()
}

async fn read_snapshot(state: &AppState, binding: &Binding) -> Result<ReadSnapshot, ApiError> {
    let store = state.store.clone();
    let scope = verb::scope_of(binding);
    tokio::task::spawn_blocking(move || store.export(&scope).map(ReadSnapshot::from))
        .await
        .map_err(|error| ApiError::internal(format!("blocking task failed: {error}")))?
        .map_err(ApiError::from)
}

/// `GET /api/speclink/v1/scopes` — membership-filtered identity scope, before
/// project/repo binding exists.
pub async fn scopes(
    State(state): State<AppState>,
    identity: IdentityOnly,
) -> Result<Json<ScopesResponse>, ApiError> {
    let memberships: HashSet<String> = state
        .identity
        .list_memberships(&identity.user.id)
        .map_err(|_| ApiError::internal("identity store unavailable"))?
        .into_iter()
        .collect();
    let projects = state
        .identity
        .list_projects()
        .map_err(|_| ApiError::internal("identity store unavailable"))?;
    let mut visible = Vec::new();
    for project in projects {
        if !memberships.contains(&project.key) {
            continue;
        }
        let repos = state
            .identity
            .list_repos(&project.key)
            .map_err(|_| ApiError::internal("identity store unavailable"))?
            .into_iter()
            .map(|repo| ScopeRef {
                id: format!("repo_{}", repo.key),
                key: repo.key,
                name: repo.name,
            })
            .collect();
        visible.push(ProjectScope {
            id: format!("prj_{}", project.key),
            key: project.key,
            name: project.name,
            repos,
        });
    }
    Ok(Json(ScopesResponse { projects: visible }))
}

/// `GET /specs/{capability}/document`.
pub async fn spec_document(
    State(state): State<AppState>,
    binding: Binding,
    Path((_project, capability)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let snapshot = read_snapshot(&state, &binding).await?;
    let content = snapshot
        .documents
        .get(&DocumentId::CanonicalSpec {
            capability: capability.clone(),
        })
        .cloned()
        .ok_or_else(|| ApiError::not_found(format!("Spec '{capability}' not found.")))?;
    Ok(ok(SpecDocumentResponse { content }, &snapshot.etag))
}

/// `GET /archived`.
pub async fn archived_list(
    State(state): State<AppState>,
    binding: Binding,
) -> Result<Response, ApiError> {
    let snapshot = read_snapshot(&state, &binding).await?;
    let mut names: Vec<String> = snapshot
        .documents
        .keys()
        .filter_map(|document| match document {
            DocumentId::ArchivedChange { change, .. } => Some(change.clone()),
            _ => None,
        })
        .collect();
    names.sort_by(|a, b| b.cmp(a));
    names.dedup();

    let archived = names
        .into_iter()
        .map(|dated_name| archived_item(&snapshot.documents, dated_name))
        .collect();
    Ok(ok(ArchivedListResponse { archived }, &snapshot.etag))
}

/// `GET /archived/{datedName}/artifacts/{*artifact}`.
pub async fn archived_artifact(
    State(state): State<AppState>,
    binding: Binding,
    Path((_project, dated_name, artifact)): Path<(String, String, String)>,
) -> Result<Response, ApiError> {
    let snapshot = read_snapshot(&state, &binding).await?;
    let content = snapshot
        .documents
        .get(&DocumentId::ArchivedChange {
            change: dated_name.clone(),
            doc: artifact.clone(),
        })
        .cloned()
        .ok_or_else(|| {
            ApiError::not_found(format!(
                "Archived artifact '{artifact}' not found in '{dated_name}'."
            ))
        })?;
    Ok(ok(SpecDocumentResponse { content }, &snapshot.etag))
}

/// `GET /archived/{datedName}/capabilities`.
pub async fn archived_capabilities(
    State(state): State<AppState>,
    binding: Binding,
    Path((_project, dated_name)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let snapshot = read_snapshot(&state, &binding).await?;
    let capabilities = archived_capability_names(&snapshot.documents, &dated_name);
    Ok(ok(capabilities, &snapshot.etag))
}

/// `GET /search?q=...` — the desktop D6 algorithm, including fixed artifact
/// order, one hit per card, char-safe snippets and live discussions only.
pub(crate) async fn search(
    State(state): State<AppState>,
    binding: Binding,
    Query(query): Query<SearchQuery>,
) -> Result<Response, ApiError> {
    let snapshot = read_snapshot(&state, &binding).await?;
    let q = query.q.trim();
    if q.is_empty() {
        return Ok(ok(SearchResponse { hits: Vec::new() }, &snapshot.etag));
    }

    let mut hits = Vec::new();
    let changes: BTreeSet<String> = snapshot
        .documents
        .keys()
        .filter_map(|document| match document {
            DocumentId::ChangeMeta { change } => Some(change.clone()),
            _ => None,
        })
        .collect();
    for change in changes {
        let mut artifacts = vec![
            "proposal.md".to_string(),
            "design.md".to_string(),
            "tasks.md".to_string(),
        ];
        artifacts.extend(
            delta_capability_names(&snapshot.documents, &change)
                .into_iter()
                .map(|capability| format!("specs/{capability}/spec.md")),
        );
        for artifact in artifacts {
            let document = DocumentId::ChangeArtifact {
                change: change.clone(),
                artifact: artifact.clone(),
            };
            let Some(content) = snapshot.documents.get(&document) else {
                continue;
            };
            if let Some(snippet) = find_snippet(content, q) {
                hits.push(SearchHit {
                    kind: "change".to_string(),
                    id: change.clone(),
                    artifact,
                    snippet,
                });
                break;
            }
        }
    }

    for (document, content) in &snapshot.documents {
        let DocumentId::Discussion {
            slug,
            archived: false,
        } = document
        else {
            continue;
        };
        if let Some(snippet) = find_snippet(content, q) {
            hits.push(SearchHit {
                kind: "discussion".to_string(),
                id: slug.clone(),
                artifact: format!("{slug}.md"),
                snippet,
            });
        }
    }

    Ok(ok(SearchResponse { hits }, &snapshot.etag))
}

fn archived_item(documents: &BTreeMap<DocumentId, String>, dated_name: String) -> ArchivedItem {
    let (date, name) = split_dated_name(&dated_name);
    let tasks = documents
        .get(&DocumentId::ArchivedChange {
            change: dated_name.clone(),
            doc: "tasks.md".to_string(),
        })
        .map(|text| {
            let parsed = speclink_core::tasks::parse(text);
            let (total, done, _) = speclink_core::tasks::progress(&parsed);
            (total, done)
        });
    let meta = documents.get(&DocumentId::ArchivedChange {
        change: dated_name.clone(),
        doc: ".openspec.yaml".to_string(),
    });
    let parsed_meta = ChangeMeta::from_text(meta.map(String::as_str)).unwrap_or_default();
    let from_discussions = parsed_meta.from_discussions();
    ArchivedItem {
        dated_name: dated_name.clone(),
        date: date.to_string(),
        name: name.to_string(),
        tasks_total: tasks.map(|counts| counts.0),
        tasks_done: tasks.map(|counts| counts.1),
        spec_count: archived_capability_names(documents, &dated_name).len(),
        created_by: parsed_meta.created_by,
        from_discussions,
    }
}

fn archived_capability_names(
    documents: &BTreeMap<DocumentId, String>,
    dated_name: &str,
) -> Vec<String> {
    documents
        .keys()
        .filter_map(|document| match document {
            DocumentId::ArchivedChange { change, doc } if change == dated_name => {
                spec_capability(doc)
            }
            _ => None,
        })
        .collect()
}

fn delta_capability_names(
    documents: &BTreeMap<DocumentId, String>,
    change_name: &str,
) -> Vec<String> {
    documents
        .keys()
        .filter_map(|document| match document {
            DocumentId::ChangeArtifact { change, artifact } if change == change_name => {
                spec_capability(artifact)
            }
            _ => None,
        })
        .collect()
}

fn spec_capability(artifact: &str) -> Option<String> {
    let capability = artifact.strip_prefix("specs/")?.strip_suffix("/spec.md")?;
    if capability.is_empty() || capability.contains('/') {
        None
    } else {
        Some(capability.to_string())
    }
}

fn split_dated_name(dated_name: &str) -> (&str, &str) {
    let bytes = dated_name.as_bytes();
    if bytes.len() > 11
        && bytes.get(4) == Some(&b'-')
        && bytes.get(7) == Some(&b'-')
        && bytes.get(10) == Some(&b'-')
        && dated_name.is_char_boundary(10)
        && dated_name.is_char_boundary(11)
    {
        (&dated_name[..10], &dated_name[11..])
    } else {
        ("", dated_name)
    }
}

fn find_snippet(text: &str, query: &str) -> Option<String> {
    let haystack: Vec<char> = text.chars().collect();
    let folded_haystack: Vec<char> = haystack.iter().map(lower_first).collect();
    let needle: Vec<char> = query
        .chars()
        .map(|character| lower_first(&character))
        .collect();
    if needle.is_empty() || folded_haystack.len() < needle.len() {
        return None;
    }
    let start = (0..=folded_haystack.len() - needle.len())
        .find(|&index| folded_haystack[index..index + needle.len()] == needle[..])?;
    let end = start + needle.len();
    let from = start.saturating_sub(SNIPPET_CONTEXT);
    let to = (end + SNIPPET_CONTEXT).min(haystack.len());
    let mut snippet = String::new();
    if from > 0 {
        snippet.push('…');
    }
    snippet.extend(haystack[from..to].iter().map(|character| match character {
        '\n' | '\r' => ' ',
        character => *character,
    }));
    if to < haystack.len() {
        snippet.push('…');
    }
    Some(snippet)
}

fn lower_first(character: &char) -> char {
    character.to_lowercase().next().unwrap_or(*character)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dated_name_and_capability_parsers_are_fail_closed() {
        assert_eq!(split_dated_name("2026-07-20-demo"), ("2026-07-20", "demo"));
        assert_eq!(split_dated_name("demo"), ("", "demo"));
        assert_eq!(
            spec_capability("specs/auth/spec.md").as_deref(),
            Some("auth")
        );
        assert!(spec_capability("specs/nested/auth/spec.md").is_none());
    }

    #[test]
    fn snippet_matches_the_desktop_char_level_contract() {
        let text = format!("{}MagicToken{}", "前".repeat(80), "後".repeat(80));
        let snippet = find_snippet(&text, "magictoken").expect("case-insensitive hit");
        assert!(snippet.starts_with('…') && snippet.ends_with('…'));
        assert!(snippet.contains("MagicToken"));
        assert!(snippet.chars().count() <= 30 + 10 + 30 + 2);
    }
}
