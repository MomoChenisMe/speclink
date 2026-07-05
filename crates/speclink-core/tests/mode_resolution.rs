//! Mode resolution: the `remote:` section of `.speclink.yaml` IS the mode signal.
//!
//! Pinned behavior: no remote section = fs mode; section present = remote mode;
//! section + local spec dir coexisting = remote wins with a one-line warning
//! flag; SPECLINK_STORE_URL overrides (or supplies) the section url — never the
//! mode; url missing from both the section and the env var is a semantic error
//! naming both settings; a leftover `.speclink.remote.yaml` is flagged for the
//! migration warning but never affects mode determination.

use speclink_core::workspace::{ModeResolution, StoreMode, Workspace};
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

    fn with_app_yaml(self, yaml: &str) -> TempProject {
        std::fs::write(self.dir.join(".speclink.yaml"), yaml).unwrap();
        self
    }

    fn with_leftover_remote_file(self, yaml: &str) -> TempProject {
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
fn remote_section_means_remote_mode() {
    let p = TempProject::new("remote")
        .with_app_yaml(&format!("remote:\n  url: {URL}\n  repo: backend\n"));
    let res = resolve(&p.workspace(), None);
    match res.mode {
        StoreMode::Remote(conn) => {
            assert_eq!(conn.url, URL);
            assert_eq!(conn.repo.as_deref(), Some("backend"));
        }
        StoreMode::Fs => panic!("expected remote mode"),
    }
    assert!(!res.coexists);
    assert!(!res.leftover_remote_file);
}

#[test]
fn no_remote_section_means_fs_mode() {
    let p = TempProject::new("fs")
        .with_openspec()
        .with_app_yaml("tools:\n  - claude\n");
    let res = resolve(&p.workspace(), None);
    assert!(matches!(res.mode, StoreMode::Fs));
    assert!(!res.coexists);
    assert!(!res.leftover_remote_file);
}

#[test]
fn repo_is_optional_in_the_remote_section() {
    let p = TempProject::new("no-repo").with_app_yaml(&format!("remote:\n  url: {URL}\n"));
    let res = resolve(&p.workspace(), None);
    match res.mode {
        StoreMode::Remote(conn) => assert_eq!(conn.repo, None),
        StoreMode::Fs => panic!("expected remote mode"),
    }
}

// --- url missing from both the section and the env var ---

#[test]
fn missing_url_everywhere_is_a_semantic_error_naming_both_settings() {
    let p = TempProject::new("no-url").with_app_yaml("remote:\n  repo: backend\n");
    let err = p.workspace().resolve_mode_with(None).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("remote.url"), "error names the field: {msg}");
    assert!(
        msg.contains("SPECLINK_STORE_URL"),
        "error names the env var alternative: {msg}"
    );
}

#[test]
fn missing_url_never_falls_back_to_fs_mode() {
    // Section present + no url anywhere must FAIL, not silently run in fs mode —
    // even when a local openspec/ tree would make fs mode "work".
    let p = TempProject::new("no-url-no-fallback")
        .with_openspec()
        .with_app_yaml("remote:\n  repo: backend\n");
    assert!(p.workspace().resolve_mode_with(None).is_err());
}

// --- SPECLINK_STORE_URL override / supply ---

#[test]
fn env_url_overrides_the_section_url() {
    let p = TempProject::new("env-override")
        .with_app_yaml(&format!("remote:\n  url: {URL}\n  repo: backend\n"));
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
fn env_url_supplies_a_missing_section_url() {
    // Committed files may omit url; the env var supplies it at runtime. A bare
    // empty section (env-only setup) is the extreme of the same case.
    for yaml in ["remote:\n  repo: backend\n", "remote:\n"] {
        let p = TempProject::new("env-supply").with_app_yaml(yaml);
        let res = resolve(&p.workspace(), Some(URL));
        match res.mode {
            StoreMode::Remote(conn) => assert_eq!(conn.url, URL, "for {yaml:?}"),
            StoreMode::Fs => panic!("expected remote mode for {yaml:?}"),
        }
    }
}

#[test]
fn empty_env_url_counts_as_unset() {
    let p = TempProject::new("env-empty").with_app_yaml(&format!("remote:\n  url: {URL}\n"));
    let res = resolve(&p.workspace(), Some("  "));
    match res.mode {
        StoreMode::Remote(conn) => assert_eq!(conn.url, URL),
        StoreMode::Fs => panic!("expected remote mode"),
    }
    // Empty env + no section url = still the missing-url error, not fs fallback.
    let p = TempProject::new("env-empty-no-url").with_app_yaml("remote:\n  repo: backend\n");
    assert!(p
        .workspace()
        .resolve_mode_with(Some("  ".to_string()))
        .is_err());
}

#[test]
fn env_url_never_flips_fs_into_remote() {
    let p = TempProject::new("env-no-flip")
        .with_openspec()
        .with_app_yaml("tools:\n  - claude\n");
    let res = resolve(&p.workspace(), Some(URL));
    assert!(
        matches!(res.mode, StoreMode::Fs),
        "the remote section, not the env var, is the mode signal"
    );
}

// --- coexistence ---

#[test]
fn coexisting_spec_dir_flags_the_warning_and_remote_wins() {
    let p = TempProject::new("coexist")
        .with_openspec()
        .with_app_yaml(&format!("remote:\n  url: {URL}\n"));
    let res = resolve(&p.workspace(), None);
    assert!(matches!(res.mode, StoreMode::Remote(_)), "remote wins");
    assert!(res.coexists, "coexistence must surface exactly one warning");
}

// --- leftover .speclink.remote.yaml: flagged, never parsed, never mode-relevant ---

#[test]
fn leftover_remote_file_is_flagged_but_does_not_affect_mode() {
    // No remote section → fs mode, even though the leftover file names a url.
    let p = TempProject::new("leftover-fs")
        .with_openspec()
        .with_app_yaml("tools:\n  - claude\n")
        .with_leftover_remote_file(&format!("url: {URL}\nrepo: backend\n"));
    let res = resolve(&p.workspace(), None);
    assert!(
        matches!(res.mode, StoreMode::Fs),
        "leftover file must not flip the mode"
    );
    assert!(res.leftover_remote_file, "leftover must be flagged");
}

#[test]
fn leftover_remote_file_content_is_never_parsed() {
    // Remote section wins; the leftover file's differing url is ignored — and so
    // is its malformed-ness (never parsed at all).
    let p = TempProject::new("leftover-remote")
        .with_app_yaml(&format!("remote:\n  url: {URL}\n"))
        .with_leftover_remote_file(": not yaml : [\n");
    let res = resolve(&p.workspace(), None);
    match res.mode {
        StoreMode::Remote(conn) => assert_eq!(conn.url, URL, "url comes from the section"),
        StoreMode::Fs => panic!("expected remote mode"),
    }
    assert!(res.leftover_remote_file);
}

// --- discovery ---

#[test]
fn discovery_finds_root_by_app_yaml_alone() {
    // A remote workspace has no openspec/ tree — .speclink.yaml marks the root.
    let p = TempProject::new("discover").with_app_yaml(&format!("remote:\n  url: {URL}\n"));
    let nested = p.dir.join("src").join("deep");
    std::fs::create_dir_all(&nested).unwrap();
    let ws = Workspace::discover(&nested).expect("root found from nested dir");
    assert_eq!(ws.root, p.dir);
}

#[test]
fn discovery_still_finds_root_by_leftover_file_alone() {
    // An unmigrated project (leftover file only) must still be discovered so the
    // CLI can print the migration warning instead of "not in a project".
    let p = TempProject::new("discover-leftover")
        .with_leftover_remote_file(&format!("url: {URL}\n"));
    let ws = Workspace::discover(&p.dir).expect("root found");
    assert_eq!(ws.root, p.dir);
    let res = resolve(&ws, None);
    assert!(matches!(res.mode, StoreMode::Fs), "leftover never sets the mode");
    assert!(res.leftover_remote_file);
}
