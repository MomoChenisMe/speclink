//! The scope of a project and a repo as a store-port scope id.
//!
//! A host that names its scopes by project and repo, as the reference server
//! does, turns the pair into the id with [`scope_id`] on every route: the id
//! is what the store port, the host kit and the event stream take.
//! [`project_and_repo`] is the reverse.

use speclink_store_port::{PortError, ScopeId};

/// The project a scope belongs to. Local single-machine deployments map to a
/// fixed default project.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectId(String);

impl ProjectId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The repository within a project a scope belongs to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepoId(String);

impl RepoId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn backend(message: String) -> PortError {
    PortError::Backend { message }
}

/// The store-port scope id of a project and a repo: `<project>/<repo>`.
///
/// A project or repo name can hold a `/`, so the project is written with its
/// `%` and `/` escaped (`%25`, `%2F`): the first `/` of an id always ends the
/// project, and no two scopes share an id. Fails when the names cannot make
/// a scope id (one longer than a scope id may be).
pub fn scope_id(project: &ProjectId, repo: &RepoId) -> Result<ScopeId, PortError> {
    let project = project.as_str().replace('%', "%25").replace('/', "%2F");
    ScopeId::new(format!("{project}/{}", repo.as_str())).map_err(|e| backend(e.to_string()))
}

/// The project and the repo a scope id names: the reverse of [`scope_id`].
/// An id that is not `<project>/<repo>`, or whose project has an escape other
/// than `%25` and `%2F`, is a backend error that names the id.
pub fn project_and_repo(scope: &ScopeId) -> Result<(ProjectId, RepoId), PortError> {
    let malformed = || backend(format!("the scope id {} is not <project>/<repo>", scope.as_str()));
    let (project, repo) = scope.as_str().split_once('/').ok_or_else(malformed)?;
    if project.is_empty() || repo.is_empty() {
        return Err(malformed());
    }
    let mut name = String::with_capacity(project.len());
    let mut rest = project;
    while let Some(at) = rest.find('%') {
        name.push_str(&rest[..at]);
        name.push(match rest.get(at..at + 3) {
            Some("%25") => '%',
            Some("%2F") => '/',
            _ => return Err(malformed()),
        });
        rest = &rest[at + 3..];
    }
    name.push_str(rest);
    Ok((ProjectId::new(name), RepoId::new(repo)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_store_port::MAX_SCOPE_BYTES;

    fn id_of(project: &str, repo: &str) -> Result<ScopeId, PortError> {
        scope_id(&ProjectId::new(project), &RepoId::new(repo))
    }

    fn names_of(id: &str) -> Result<(ProjectId, RepoId), PortError> {
        project_and_repo(&ScopeId::new(id).unwrap())
    }

    /// The error of a scope id that has the wrong form: it names the id.
    fn malformed(id: &str) -> String {
        match names_of(id) {
            Err(PortError::Backend { message }) => {
                assert!(message.contains(id), "{id}: {message}");
                message
            }
            other => panic!("{id}: expected a backend error, got {other:?}"),
        }
    }

    #[test]
    fn a_project_and_a_repo_make_one_scope_id_and_it_reads_back() {
        // 「含斜線的專案名稱」「含百分號的專案名稱」
        let rows = [
            ("default", "main", "default/main"),
            ("team/alpha", "main", "team%2Falpha/main"),
            ("team", "alpha/main", "team/alpha/main"),
            ("50%", "main", "50%25/main"),
            ("100%", "r", "100%25/r"),
        ];
        for (project, repo, id) in rows {
            let made = id_of(project, repo).unwrap();
            assert_eq!(made.as_str(), id, "{project} + {repo}");
            assert_eq!(
                project_and_repo(&made).unwrap(),
                (ProjectId::new(project), RepoId::new(repo)),
                "{id}"
            );
        }
        // A `/` in the project and a `/` in the repo give two ids.
        assert_ne!(id_of("team/alpha", "main").unwrap(), id_of("team", "alpha/main").unwrap());
    }

    #[test]
    fn a_scope_id_of_the_wrong_form_is_an_error_that_names_it() {
        // 「範圍識別字的形式不合」: no `/`, an empty part, an escape that is
        // neither `%25` nor `%2F`.
        for bad in ["no-slash", "/main", "team/", "a%2G/main"] {
            assert_eq!(malformed(bad), format!("the scope id {bad} is not <project>/<repo>"));
        }
    }

    #[test]
    fn an_empty_name_makes_an_id_that_does_not_read_back() {
        // As before the move: the id is made, and the reverse refuses it.
        for (project, repo, id) in [("", "main", "/main"), ("team", "", "team/"), ("", "", "/")] {
            assert_eq!(id_of(project, repo).unwrap().as_str(), id);
            malformed(id);
        }
    }

    #[test]
    fn a_name_with_a_control_character_makes_no_id() {
        for (project, repo) in [("a\nb", "main"), ("team", "a\nb"), ("a\0b", "main"), ("team", "a\0b")] {
            match id_of(project, repo) {
                Err(PortError::Backend { message }) => {
                    assert!(message.contains("must not contain control characters"), "{message}")
                }
                other => panic!("{project:?} + {repo:?}: expected a backend error, got {other:?}"),
            }
        }
    }

    #[test]
    fn the_reverse_refuses_a_cut_or_unknown_escape_and_does_not_panic() {
        // A `%` at the end, a cut escape, an escape of another character, a
        // lowercase escape, and a `%` before a character of more than 1 byte.
        for bad in ["a%", "a%2", "a%25", "a%/m", "a%2/m", "a%zz/main", "a%2f/m", "a%𠮷/m", "a%é/m", "%"] {
            malformed(bad);
        }
    }

    #[test]
    fn only_the_first_slash_ends_the_project() {
        let names = |id| names_of(id).map(|(p, r)| (p.as_str().to_string(), r.as_str().to_string())).unwrap();
        assert_eq!(names("a//b"), ("a".to_string(), "/b".to_string()));
        assert_eq!(names("a/b/"), ("a".to_string(), "b/".to_string()));
    }

    #[test]
    fn names_that_make_an_id_longer_than_a_scope_id_may_be_are_an_error() {
        let long = "p".repeat(MAX_SCOPE_BYTES);
        assert!(matches!(id_of(&long, "main"), Err(PortError::Backend { .. })));
        assert!(matches!(id_of("team", &long), Err(PortError::Backend { .. })));
        // An escape makes the id longer than the names: 127 `/` are 381 bytes.
        assert!(matches!(id_of(&"/".repeat(127), "r"), Err(PortError::Backend { .. })));
        // The longest id: 256 bytes with the `/` between the two names.
        let fits = "p".repeat(MAX_SCOPE_BYTES - 2);
        assert_eq!(id_of(&fits, "r").unwrap().as_str().len(), MAX_SCOPE_BYTES);
    }
}
