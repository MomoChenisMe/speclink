//! The skill version notice (spec workspace-tools「CLI 的技能版本提醒」): after a
//! remote verb's handshake, one stderr line when this CLI's skill templates are
//! older than the Server's — at most once a calendar day for each Server.
//!
//! It never stops a verb: stdout, `--json` and the exit code stay as they are,
//! and a record that cannot be read or written means no notice and no error.

use speclink_core::init::{is_newer_version, ASSET_VERSION};
use std::collections::BTreeMap;
use std::path::Path;

/// How a user updates Speclink: by the channel it was installed from.
const HOW_TO_UPDATE: &str =
    "npm i -g @speclink/cli, brew upgrade speclink, the install script, or the desktop app's updater";

/// The record of the last notice per Server, in the user-level config directory:
/// a JSON object from a Server's address to a calendar day. It holds no credential.
const RECORD_FILE: &str = "skill-notice.json";

/// Print the notice, if one is due, for the Server at `origin` that declared
/// `declared` as its skill template version. Called once, after the handshake of a
/// remote verb; `speclink update` does not call it (its own summary has the version).
pub(crate) fn after_handshake(origin: &str, declared: Option<&str>) {
    let dir = speclink_remote::auth::speclink_config_dir();
    if let Some(line) = notice(&dir, origin, declared, ASSET_VERSION, &today()) {
        eprintln!("{line}");
    }
}

/// The local calendar day, `YYYY-MM-DD`.
fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// The notice line, when `local` is numerically older than `declared` and `server`
/// was not told on `today` yet. A due notice is recorded before it is returned: a
/// record that cannot be written gives no notice.
fn notice(dir: &Path, server: &str, declared: Option<&str>, local: &str, today: &str) -> Option<String> {
    let declared = declared?.trim();
    // False when either version does not parse as numbers.
    if !is_newer_version(declared, local) {
        return None;
    }
    let server = without_user_info(server);
    let path = dir.join(RECORD_FILE);
    // No record yet, or one that is not a record: nothing was told today.
    let mut told: BTreeMap<String, String> =
        std::fs::read_to_string(&path).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default();
    if told.get(&server).map(String::as_str) == Some(today) {
        return None;
    }
    // Only today's entries can stop a notice, so the record keeps no other.
    told.retain(|_, day| day == today);
    told.insert(server, today.to_string());
    std::fs::create_dir_all(dir).ok()?;
    std::fs::write(&path, serde_json::to_string_pretty(&told).ok()?).ok()?;
    Some(format!(
        "speclink: this CLI's skill templates ({local}) are older than this server's ({declared}) — update Speclink: {HOW_TO_UPDATE}"
    ))
}

/// `origin` without the `user:password@` a URL can carry: the record is no place
/// for a credential, and the same Server is one Server with or without it.
fn without_user_info(origin: &str) -> String {
    match origin.split_once("://") {
        Some((scheme, authority)) => {
            let host = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
            format!("{scheme}://{host}")
        }
        None => origin.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SERVER: &str = "https://team.example.test";

    /// A throwaway directory, removed on drop.
    struct TempDir(std::path::PathBuf);

    impl TempDir {
        fn new() -> TempDir {
            static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let dir = std::env::temp_dir().join(format!("speclink-cli-skill-notice-{}-{n}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn record(dir: &Path) -> String {
        std::fs::read_to_string(dir.join(RECORD_FILE)).expect("the record exists")
    }

    #[test]
    fn an_older_cli_gets_one_line_with_both_versions_and_how_to_update() {
        let dir = TempDir::new();
        let line = notice(dir.path(), SERVER, Some("v1.58.0"), "v1.57.0", "2026-11-12").expect("a notice");
        assert_eq!(
            line,
            format!(
                "speclink: this CLI's skill templates (v1.57.0) are older than this server's (v1.58.0) — update Speclink: {HOW_TO_UPDATE}"
            )
        );
        assert!(!line.contains('\n'), "one line");
    }

    #[test]
    fn the_notice_stays_one_line_when_the_declared_version_has_white_space_around_it() {
        let dir = TempDir::new();
        let line = notice(dir.path(), SERVER, Some(" v1.58.0\n"), "v1.57.0", "2026-11-12").expect("a notice");
        assert!(line.contains("(v1.58.0)") && !line.contains('\n'), "{line:?}");
    }

    #[test]
    fn the_same_day_and_server_get_no_second_notice_and_the_next_day_does() {
        let dir = TempDir::new();
        assert!(notice(dir.path(), SERVER, Some("v1.58.0"), "v1.57.0", "2026-11-12").is_some());
        assert_eq!(notice(dir.path(), SERVER, Some("v1.58.0"), "v1.57.0", "2026-11-12"), None);
        assert!(notice(dir.path(), SERVER, Some("v1.58.0"), "v1.57.0", "2026-11-13").is_some(), "the next day");
        assert_eq!(notice(dir.path(), SERVER, Some("v1.58.0"), "v1.57.0", "2026-11-13"), None);
    }

    #[test]
    fn each_server_counts_by_itself() {
        let dir = TempDir::new();
        assert!(notice(dir.path(), SERVER, Some("v1.58.0"), "v1.57.0", "2026-11-12").is_some());
        let other = "https://other.example.test";
        assert!(notice(dir.path(), other, Some("v1.58.0"), "v1.57.0", "2026-11-12").is_some(), "another Server");
        assert_eq!(notice(dir.path(), other, Some("v1.58.0"), "v1.57.0", "2026-11-12"), None);
        assert_eq!(notice(dir.path(), SERVER, Some("v1.58.0"), "v1.57.0", "2026-11-12"), None, "the first is still recorded");
    }

    #[test]
    fn nothing_is_said_or_recorded_when_this_cli_is_not_older() {
        let dir = TempDir::new();
        for (declared, local) in [
            (Some("v1.57.0"), "v1.57.0"),
            (Some("v1.56.9"), "v1.57.0"),
            (Some("v1.57"), "v1.57.0"),
            (None, "v1.57.0"),
            (Some("latest"), "v1.57.0"),
            (Some("v1.58.0"), "dev"),
            (Some(""), "v1.57.0"),
        ] {
            assert_eq!(notice(dir.path(), SERVER, declared, local, "2026-11-12"), None, "{declared:?} / {local}");
        }
        assert!(!dir.path().join(RECORD_FILE).exists(), "no record without a notice");
    }

    #[test]
    fn a_record_that_cannot_be_written_means_no_notice_and_no_error() {
        let dir = TempDir::new();
        // The config directory's place is taken by a file: nothing can be created under it.
        let taken = dir.path().join("speclink");
        std::fs::write(&taken, "not a directory").unwrap();
        assert_eq!(notice(&taken, SERVER, Some("v1.58.0"), "v1.57.0", "2026-11-12"), None);
        assert_eq!(std::fs::read_to_string(&taken).unwrap(), "not a directory");
    }

    #[test]
    fn a_record_that_is_not_readable_as_a_record_is_replaced() {
        let dir = TempDir::new();
        std::fs::write(dir.path().join(RECORD_FILE), "{ not json").unwrap();
        assert!(notice(dir.path(), SERVER, Some("v1.58.0"), "v1.57.0", "2026-11-12").is_some());
        assert_eq!(notice(dir.path(), SERVER, Some("v1.58.0"), "v1.57.0", "2026-11-12"), None);
    }

    #[test]
    fn the_record_holds_the_servers_address_and_the_day_and_nothing_else() {
        let dir = TempDir::new();
        notice(dir.path(), "https://alice:s3cret@team.example.test", Some("v1.58.0"), "v1.57.0", "2026-11-12").unwrap();
        let text = record(dir.path());
        let parsed: std::collections::BTreeMap<String, String> = serde_json::from_str(&text).expect("a JSON object");
        assert_eq!(parsed.into_iter().collect::<Vec<_>>(), [(SERVER.to_string(), "2026-11-12".to_string())]);
        assert!(!text.contains("s3cret") && !text.contains("alice"), "no user info of the URL: {text}");
        // The same Server with or without user info is one Server.
        assert_eq!(notice(dir.path(), SERVER, Some("v1.58.0"), "v1.57.0", "2026-11-12"), None);
    }

    #[test]
    fn the_record_keeps_only_the_days_that_still_count() {
        let dir = TempDir::new();
        notice(dir.path(), SERVER, Some("v1.58.0"), "v1.57.0", "2026-11-12").unwrap();
        notice(dir.path(), "https://other.example.test", Some("v1.58.0"), "v1.57.0", "2026-11-13").unwrap();
        let parsed: std::collections::BTreeMap<String, String> = serde_json::from_str(&record(dir.path())).unwrap();
        assert_eq!(parsed.len(), 1, "yesterday's entry is gone: {parsed:?}");
    }

    #[test]
    fn today_is_a_calendar_day() {
        let day = today();
        let parts: Vec<&str> = day.split('-').collect();
        assert!(parts.len() == 3 && parts[0].len() == 4 && parts[1].len() == 2 && parts[2].len() == 2, "{day}");
    }
}
