//! Project/Repo binding resolution — fail closed.
//!
//! A binding names the (project, repo) pair an execution is scoped to.
//! Resolution rejects with a typed reason when the binding is missing,
//! not permitted, or ambiguous — it never auto-picks the first candidate
//! (platform architecture §4.7). Local fs mode maps the workspace root to
//! a fixed default project/repo with zero configuration; remote candidate
//! discovery (the network handshake) is Phase 2 — this module owns the
//! validation logic and the error shapes only.

use speclink_store::{ProjectId, RepoId};

/// One available binding a resolution can land on. Phase 2's server
/// handshake supplies these; local fs mode maps the workspace root to the
/// single fixed default candidate. `project`/`repo` are the immutable
/// identities; the keys are the human-readable names shown in rejections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingCandidate {
    pub project: ProjectId,
    pub repo: RepoId,
    pub project_key: String,
    pub repo_key: String,
    /// Authorization hook seam: local mode always permits. A candidate that
    /// exists but is not permitted rejects with PermissionDenied — it never
    /// silently counts as missing.
    pub permitted: bool,
}

impl BindingCandidate {
    /// The "project/repo" label rejections list.
    fn label(&self) -> String {
        format!("{}/{}", self.project_key, self.repo_key)
    }
}

/// The resolved (project, repo) pair an execution is scoped to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBinding {
    pub project: ProjectId,
    pub repo: RepoId,
}

/// Why a binding resolution refused — the closed reason set. Resolution
/// never auto-picks: every rejection names its cause and the candidates
/// involved so the caller can act.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingError {
    /// No candidate exists for this execution.
    Missing,
    /// Candidates exist but none is permitted for the caller.
    PermissionDenied { candidates: Vec<String> },
    /// More than one permitted candidate qualifies — the caller must choose
    /// explicitly; picking the first would cross a tenant boundary silently.
    Ambiguous { candidates: Vec<String> },
}

/// Resolve a binding from the available candidates, fail closed: exactly one
/// permitted candidate resolves; zero candidates is Missing; candidates with
/// no permitted member is PermissionDenied; more than one permitted member
/// is Ambiguous with the qualifying candidates listed in input order.
pub fn resolve_binding(candidates: Vec<BindingCandidate>) -> Result<ResolvedBinding, BindingError> {
    if candidates.is_empty() {
        return Err(BindingError::Missing);
    }
    let permitted: Vec<&BindingCandidate> = candidates.iter().filter(|c| c.permitted).collect();
    match permitted.as_slice() {
        [] => Err(BindingError::PermissionDenied {
            candidates: candidates.iter().map(BindingCandidate::label).collect(),
        }),
        [one] => Ok(ResolvedBinding {
            project: one.project.clone(),
            repo: one.repo.clone(),
        }),
        many => Err(BindingError::Ambiguous {
            candidates: many.iter().map(|c| c.label()).collect(),
        }),
    }
}

/// The fixed zero-configuration binding of local fs mode: the workspace root
/// is the binding source and maps to the default project/repo (the same
/// identities the TeamStore contract uses for local deployments).
pub fn local_default_binding() -> ResolvedBinding {
    ResolvedBinding {
        project: ProjectId::new("default"),
        repo: RepoId::new("main"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_store::{ProjectId, RepoId};

    fn candidate(project: &str, repo: &str, permitted: bool) -> BindingCandidate {
        BindingCandidate {
            project: ProjectId::new(project),
            repo: RepoId::new(repo),
            project_key: project.to_string(),
            repo_key: repo.to_string(),
            permitted,
        }
    }

    // --- Project 與 Repo binding 驗證 fail closed ---

    #[test]
    fn ambiguous_binding_rejects_and_lists_candidates() {
        // 兩個同時合格的候選 → 帶「多個候選」原因的拒絕並列出候選，
        // 不自動選第一個。
        let err = resolve_binding(vec![
            candidate("acme", "web", true),
            candidate("acme", "api", true),
        ])
        .expect_err("two qualified candidates must reject");
        match err {
            BindingError::Ambiguous { candidates } => {
                assert_eq!(candidates, vec!["acme/web".to_string(), "acme/api".to_string()]);
            }
            other => panic!("expected Ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn missing_binding_rejects_with_missing_reason() {
        let err = resolve_binding(Vec::new()).expect_err("no candidate must reject");
        assert!(matches!(err, BindingError::Missing));
    }

    #[test]
    fn unpermitted_binding_rejects_with_permission_reason() {
        // 候選存在但全部無權限 → PermissionDenied，不得靜默當缺失。
        let err = resolve_binding(vec![candidate("acme", "web", false)])
            .expect_err("unpermitted candidate must reject");
        match err {
            BindingError::PermissionDenied { candidates } => {
                assert_eq!(candidates, vec!["acme/web".to_string()]);
            }
            other => panic!("expected PermissionDenied, got {other:?}"),
        }
    }

    #[test]
    fn single_permitted_candidate_resolves() {
        let binding = resolve_binding(vec![candidate("acme", "web", true)])
            .expect("exactly one qualified candidate resolves");
        assert_eq!(binding.project.as_str(), "acme");
        assert_eq!(binding.repo.as_str(), "web");
    }

    #[test]
    fn one_permitted_among_unpermitted_resolves_to_the_permitted_one() {
        // 合格＝有權限：無權限的候選不參與多義判定。
        let binding = resolve_binding(vec![
            candidate("acme", "web", false),
            candidate("acme", "api", true),
        ])
        .expect("the single permitted candidate resolves");
        assert_eq!(binding.repo.as_str(), "api");
    }

    #[test]
    fn local_fs_mode_maps_default_project_repo_with_zero_config() {
        // 本地 fs 模式：workspace root 即 binding 來源，零設定映射固定
        // default project/repo。
        let binding = local_default_binding();
        assert_eq!(binding.project.as_str(), "default");
        assert_eq!(binding.repo.as_str(), "main");
    }

    #[test]
    fn binding_error_reasons_are_a_closed_set() {
        // 錯誤形狀區分缺失、無權限、多義（design 實作契約）。
        for err in [
            BindingError::Missing,
            BindingError::PermissionDenied { candidates: vec![] },
            BindingError::Ambiguous { candidates: vec![] },
        ] {
            match err {
                BindingError::Missing => {}
                BindingError::PermissionDenied { .. } => {}
                BindingError::Ambiguous { .. } => {}
            }
        }
    }
}
