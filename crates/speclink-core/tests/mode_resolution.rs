//! Mode resolution: `.speclink.remote.yaml` presence IS the mode signal.
//!
//! Pinned behavior: no connection file = fs mode; file present = remote mode;
//! file + local spec dir coexisting = remote wins with a one-line warning
//! flag; SPECLINK_STORE_URL overrides the connection url (never the mode);
//! `url` is required and its absence is a semantic error.

use speclink_core::workspace::{ModeResolution, RemoteConnection, StoreMode, Workspace};
use std::path::PathBuf;

/// Throwaway project root, removed on drop.
struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn new(tag: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-core-mode-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        TempProject { dir }
    }

    fn with_openspec(self) -> TempProject {
        std::fs::create_dir_all(self.dir.join("openspec").join("changes")).unwrap();
        self
    }

    fn with_remote_file(self, yaml: &str) -> TempProject {
        std::fs::write(self.dir.join(".speclink.remote.yaml"), yaml).unwrap();
        self
    }

    fn workspace(&self) -> Workspace {
        Workspace::discover(&self.dir).expect("project root discovered")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

const URL: &str = "https://team.example.com/api/speclink/v1/projects/foo";

fn resolve(ws: &Workspace, env: Option<&str>) -> ModeResolution {
    ws.resolve_mode_with(env.map(str::to_string))
        .expect("mode resolves")
}

// --- mode signal ---

#[test]
fn no_connection_file_means_fs_mode() {
    let p = TempProject::new("fs").with_openspec();
    let res = resolve(&p.workspace(), None);
    assert!(matches!(res.mode, StoreMode::Fs));
    assert!(!res.coexists);
}

#[test]
fn connection_file_means_remote_mode() {
    let p = TempProject::new("remote")
        .with_remote_file(&format!("url: {URL}\nrepo: backend\n"));
    let res = resolve(&p.workspace(), None);
    match res.mode {
        StoreMode::Remote(conn) => {
            assert_eq!(conn.url, URL);
            assert_eq!(conn.repo.as_deref(), Some("backend"));
        }
        StoreMode::Fs => panic!("expected remote mode"),
    }
    assert!(!res.coexists);
}

#[test]
fn discovery_finds_root_by_connection_file_alone() {
    // A remote workspace has no openspec/ tree and may have no .speclink.yaml.
    let p = TempProject::new("discover").with_remote_file(&format!("url: {URL}\n"));
    let nested = p.dir.join("src").join("deep");
    std::fs::create_dir_all(&nested).unwrap();
    let ws = Workspace::discover(&nested).expect("root found from nested dir");
    assert_eq!(ws.root, p.dir);
}

#[test]
fn repo_is_optional_in_the_connection_file() {
    let p = TempProject::new("no-repo").with_remote_file(&format!("url: {URL}\n"));
    let res = resolve(&p.workspace(), None);
    match res.mode {
        StoreMode::Remote(conn) => assert_eq!(conn.repo, None),
        StoreMode::Fs => panic!("expected remote mode"),
    }
}

// --- coexistence ---

#[test]
fn coexisting_spec_dir_flags_the_warning_and_remote_wins() {
    let p = TempProject::new("coexist")
        .with_openspec()
        .with_remote_file(&format!("url: {URL}\n"));
    let res = resolve(&p.workspace(), None);
    assert!(matches!(res.mode, StoreMode::Remote(_)), "remote wins");
    assert!(res.coexists, "coexistence must surface exactly one warning");
}

// --- SPECLINK_STORE_URL override ---

#[test]
fn env_url_overrides_the_connection_url() {
    let p = TempProject::new("env-override")
        .with_remote_file(&format!("url: {URL}\nrepo: backend\n"));
    let other = "https://staging.example.com/api/speclink/v1/projects/foo";
    let res = resolve(&p.workspace(), Some(other));
    match res.mode {
        StoreMode::Remote(conn) => {
            assert_eq!(conn.url, other);
            assert_eq!(conn.repo.as_deref(), Some("backend"), "repo untouched");
        }
        StoreMode::Fs => panic!("expected remote mode"),
    }
}

#[test]
fn empty_env_url_counts_as_unset() {
    let p = TempProject::new("env-empty").with_remote_file(&format!("url: {URL}\n"));
    let res = resolve(&p.workspace(), Some("  "));
    match res.mode {
        StoreMode::Remote(conn) => assert_eq!(conn.url, URL),
        StoreMode::Fs => panic!("expected remote mode"),
    }
}

#[test]
fn env_url_never_flips_fs_into_remote() {
    let p = TempProject::new("env-no-flip").with_openspec();
    let res = resolve(&p.workspace(), Some(URL));
    assert!(
        matches!(res.mode, StoreMode::Fs),
        "the connection file, not the env var, is the mode signal"
    );
}

// --- connection file parsing ---

#[test]
fn parse_requires_url() {
    let err = RemoteConnection::from_text("repo: backend\n").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("url"), "error names the field: {msg}");
    assert!(
        msg.contains(".speclink.remote.yaml"),
        "error names the file: {msg}"
    );
}

#[test]
fn parse_rejects_empty_url() {
    let err = RemoteConnection::from_text("url: \"\"\n").unwrap_err();
    assert!(err.to_string().contains("url"));
}

#[test]
fn parse_reports_malformed_yaml_semantically() {
    let err = RemoteConnection::from_text(": not yaml : [\n").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains(".speclink.remote.yaml"),
        "error names the file: {msg}"
    );
}

#[test]
fn missing_url_in_workspace_resolution_is_a_semantic_error() {
    let p = TempProject::new("bad-file").with_remote_file("repo: backend\n");
    let err = p.workspace().resolve_mode_with(None).unwrap_err();
    assert!(err.to_string().contains("url"));
}
