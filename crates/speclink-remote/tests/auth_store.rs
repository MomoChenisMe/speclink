//! PAT storage in the credentials file.
//!
//! Pinned behavior: credentials live in a YAML map keyed by URL origin in
//! the user-level config directory (never inside a repo), and the file is
//! owner-only on Unix (0600). Which layer wins when several hold a credential
//! is the resolution ladder's contract, pinned in `credential_ladder.rs`.

use speclink_remote::auth;
use std::path::PathBuf;

/// Throwaway directory standing in for the user-level config dir.
struct TempDir {
    dir: PathBuf,
}

impl TempDir {
    fn new(tag: &str) -> TempDir {
        let dir = std::env::temp_dir().join(format!(
            "speclink-remote-auth-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        TempDir { dir }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

const ORIGIN: &str = "https://team.example.com";

#[test]
fn origin_of_strips_the_project_path() {
    assert_eq!(
        auth::origin_of("https://team.example.com/api/speclink/v1/projects/foo"),
        "https://team.example.com"
    );
    assert_eq!(
        auth::origin_of("http://localhost:8080/api/speclink/v1/projects/foo"),
        "http://localhost:8080"
    );
}

#[test]
fn save_token_writes_the_credentials_file_under_the_config_dir() {
    let tmp = TempDir::new("save");
    let path = auth::save_token_at(&tmp.dir, ORIGIN, "pat-abc").expect("save token");
    assert_eq!(path, auth::credentials_path_in(&tmp.dir));
    assert!(
        path.starts_with(&tmp.dir),
        "credentials must live in the user-level dir, got {}",
        path.display()
    );
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains(ORIGIN), "file keys tokens by origin: {text}");
    assert!(text.contains("pat-abc"), "file holds the token: {text}");
}

#[cfg(unix)]
#[test]
fn credentials_file_is_owner_only_on_unix() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = TempDir::new("perm");
    let path = auth::save_token_at(&tmp.dir, ORIGIN, "pat-abc").expect("save token");
    let mode = std::fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600, "expected 0600, got {:o}", mode & 0o777);
}

#[test]
fn load_returns_the_saved_token_per_origin() {
    let tmp = TempDir::new("load");
    auth::save_token_at(&tmp.dir, ORIGIN, "pat-abc").unwrap();
    auth::save_token_at(&tmp.dir, "https://other.example.com", "pat-xyz").unwrap();
    assert_eq!(auth::load_token_at(&tmp.dir, ORIGIN).as_deref(), Some("pat-abc"));
    assert_eq!(
        auth::load_token_at(&tmp.dir, "https://other.example.com").as_deref(),
        Some("pat-xyz")
    );
}

#[test]
fn missing_credentials_mean_not_logged_in() {
    let tmp = TempDir::new("missing");
    assert_eq!(auth::load_token_at(&tmp.dir, ORIGIN), None);
}

#[test]
fn saving_a_second_origin_keeps_the_first() {
    let tmp = TempDir::new("two-origins");
    auth::save_token_at(&tmp.dir, ORIGIN, "pat-abc").unwrap();
    auth::save_token_at(&tmp.dir, "https://other.example.com", "pat-xyz").unwrap();
    // Re-saving the first origin overwrites its token only.
    auth::save_token_at(&tmp.dir, ORIGIN, "pat-rotated").unwrap();
    assert_eq!(auth::load_token_at(&tmp.dir, ORIGIN).as_deref(), Some("pat-rotated"));
    assert_eq!(
        auth::load_token_at(&tmp.dir, "https://other.example.com").as_deref(),
        Some("pat-xyz")
    );
}
