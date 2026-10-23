//! spec-dir-migration「update 自動遷移舊佈局」與「init 匯入 OpenSpec 專案」:
//! `openspec/` → `speclink/` and `.openspec.yaml` → `.meta.yaml` when it is safe,
//! a stated deferral when it is not, never a commit and never a `spec_dir` line.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Repo {
    base: PathBuf,
    dir: PathBuf,
    home: PathBuf,
}

impl Repo {
    fn new(tag: &str) -> Repo {
        let base = std::env::temp_dir().join(format!(
            "speclink-cli-migration-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        let dir = base.join("project");
        let home = base.join("home");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        Repo { base, dir, home }
    }

    fn write(&self, rel: &str, content: &str) {
        let path = self.dir.join(rel.split('/').collect::<PathBuf>());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn read(&self, rel: &str) -> Vec<u8> {
        std::fs::read(self.dir.join(rel.split('/').collect::<PathBuf>()))
            .unwrap_or_else(|e| panic!("{rel}: {e}"))
    }

    fn exists(&self, rel: &str) -> bool {
        self.dir.join(rel.split('/').collect::<PathBuf>()).exists()
    }

    fn git_in(&self, dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["-c", "user.name=T", "-c", "user.email=t@example.com"])
            .args(args)
            .env("HOME", &self.home)
            .output()
            .expect("run git");
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn git(&self, args: &[&str]) -> String {
        self.git_in(&self.dir, args)
    }

    fn commit_all(&self) {
        self.git(&["init", "-q"]);
        self.git(&["add", "-A"]);
        self.git(&["commit", "-qm", "init"]);
    }

    fn commits(&self) -> String {
        self.git(&["rev-list", "--count", "HEAD"])
    }

    fn run_in(&self, dir: &Path, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(dir)
            .env_remove("SPECLINK_LOCALE")
            .env_remove("SPECLINK_SPEC_LOCALE")
            .env_remove("SPECLINK_TDD")
            .env_remove("SPECLINK_AUDIT")
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .env("USERPROFILE", &self.home)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.home)
            .output()
            .expect("run speclink binary")
    }

    fn run(&self, args: &[&str]) -> Output {
        self.run_in(&self.dir, args)
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.base);
    }
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

const APP: &str = "tools: [claude]\n";
const MANUAL_SKILL: &str = ".claude/skills/speclink-manual/SKILL.md";

/// A committed legacy project: two active and three archived changes.
fn legacy_repo(tag: &str) -> Repo {
    let r = Repo::new(tag);
    r.write(".speclink.yaml", APP);
    for (i, rel) in [
        "openspec/changes/alpha",
        "openspec/changes/beta",
        "openspec/changes/archive/2026-01-01-one",
        "openspec/changes/archive/2026-01-02-two",
        "openspec/changes/archive/2026-01-03-three",
    ]
    .into_iter()
    .enumerate()
    {
        r.write(&format!("{rel}/.openspec.yaml"), &format!("schema: spec-driven\ncreated_by: Author {i}\n"));
        r.write(&format!("{rel}/tasks.md"), "- [ ] 1.1 x\n");
    }
    r.write("openspec/config.yaml", "schema: spec-driven\n");
    r.commit_all();
    r
}

// --- update ---

#[test]
fn update_migrates_a_clean_legacy_project() {
    let r = legacy_repo("clean");
    let before = r.commits();
    let metas: Vec<(String, Vec<u8>)> = ["alpha", "beta", "archive/2026-01-01-one", "archive/2026-01-02-two", "archive/2026-01-03-three"]
        .into_iter()
        .map(|c| (c.to_string(), r.read(&format!("openspec/changes/{c}/.openspec.yaml"))))
        .collect();

    let out = r.run(&["update"]);
    assert!(out.status.success(), "update: {}", stderr_of(&out));
    assert!(!r.exists("openspec"), "openspec/ is gone");
    for (change, bytes) in &metas {
        assert_eq!(&r.read(&format!("speclink/changes/{change}/.meta.yaml")), bytes, "{change}");
        assert!(!r.exists(&format!("speclink/changes/{change}/.openspec.yaml")), "{change}");
    }
    assert_eq!(r.read(".speclink.yaml"), APP.as_bytes(), "no spec_dir is written");
    assert_eq!(r.commits(), before, "no commit is made");

    let stdout = stdout_of(&out);
    for needle in ["5", "speclink/", "openspec/", "upgrade", "scripts", ".gitignore"] {
        assert!(stdout.contains(needle), "stdout names {needle:?}:\n{stdout}");
    }
    let skill = String::from_utf8(r.read(MANUAL_SKILL)).unwrap();
    assert!(
        skill.contains("`speclink/manual/`") && !skill.contains("`openspec/manual/`"),
        "skills render against speclink/: {skill}"
    );
}

#[test]
fn update_defers_while_another_worktree_exists() {
    let r = legacy_repo("worktree");
    let wt = r.base.join("side");
    r.git(&["worktree", "add", "-q", wt.to_str().unwrap(), "-b", "side"]);

    let out = r.run(&["update"]);
    assert!(out.status.success(), "update: {}", stderr_of(&out));
    assert!(r.exists("openspec/changes/alpha/.openspec.yaml") && !r.exists("speclink"));
    let stdout = stdout_of(&out);
    assert!(stdout.contains("worktree") && stdout.contains("side"), "{stdout}");
    let skill = String::from_utf8(r.read(MANUAL_SKILL)).unwrap();
    assert!(skill.contains("`openspec/manual/`"), "skills still sync against openspec/");
}

#[test]
fn update_defers_while_openspec_has_uncommitted_changes() {
    let r = legacy_repo("dirty");
    r.write("openspec/changes/alpha/tasks.md", "- [x] 1.1 x\n");

    let out = r.run(&["update"]);
    assert!(out.status.success(), "update: {}", stderr_of(&out));
    assert!(r.exists("openspec/changes/alpha/.openspec.yaml") && !r.exists("speclink"));
    let stdout = stdout_of(&out);
    assert!(stdout.contains("openspec/") && stdout.contains("uncommitted"), "{stdout}");
}

#[test]
fn update_inside_a_linked_worktree_points_back_at_the_main_checkout() {
    let r = legacy_repo("linked");
    let wt = r.base.join("side");
    r.git(&["worktree", "add", "-q", wt.to_str().unwrap(), "-b", "side"]);

    let out = r.run_in(&wt, &["update"]);
    assert!(out.status.success(), "update: {}", stderr_of(&out));
    assert!(wt.join("openspec").is_dir() && !wt.join("speclink").exists());
    assert!(r.exists("openspec") && !r.exists("speclink"), "the main checkout is untouched");
    assert!(stdout_of(&out).contains("main checkout"), "{}", stdout_of(&out));
}

#[test]
fn update_renames_a_leftover_legacy_data_file() {
    let r = Repo::new("leftover");
    r.write(".speclink.yaml", APP);
    r.write("speclink/changes/done/.meta.yaml", "schema: spec-driven\n");
    r.write("speclink/changes/late/.openspec.yaml", "schema: spec-driven\nfrom: old-branch\n");
    r.commit_all();

    let out = r.run(&["update"]);
    assert!(out.status.success(), "update: {}", stderr_of(&out));
    assert_eq!(r.read("speclink/changes/late/.meta.yaml"), b"schema: spec-driven\nfrom: old-branch\n");
    assert!(!r.exists("speclink/changes/late/.openspec.yaml"));
    assert!(stdout_of(&out).contains("1 "), "{}", stdout_of(&out));
}

#[test]
fn update_with_an_explicit_spec_dir_moves_nothing() {
    let r = Repo::new("explicit");
    r.write(".speclink.yaml", "tools: [claude]\nspec_dir: openspec\n");
    r.write("openspec/changes/kept/.openspec.yaml", "schema: spec-driven\n");
    r.commit_all();

    let out = r.run(&["update"]);
    assert!(out.status.success(), "update: {}", stderr_of(&out));
    assert!(r.exists("openspec/changes/kept/.openspec.yaml") && !r.exists("speclink"));
    let stdout = stdout_of(&out);
    for needle in ["openspec/", "speclink/", "worktree", "uncommitted"] {
        assert!(!stdout.contains(needle), "no migration hint ({needle:?}):\n{stdout}");
    }
}

// --- init imports an OpenSpec project ---

const OPENSPEC_CONFIG: &str = "schema: spec-driven\ncontext: |\n  Our own context.\nrules:\n  proposal:\n    - Keep it short\n";

/// A committed OpenSpec project: custom config, a custom schema, two
/// canonical specs and one change.
fn openspec_repo(tag: &str) -> Repo {
    let r = Repo::new(tag);
    r.write("openspec/config.yaml", OPENSPEC_CONFIG);
    r.write("openspec/schemas/my-flow/schema.yaml", "name: my-flow\n");
    r.write("openspec/specs/auth/spec.md", "# auth\n");
    r.write("openspec/specs/billing/spec.md", "# billing\n");
    r.write("openspec/changes/demo/.openspec.yaml", "schema: spec-driven\ncreated: 2026-09-01\n");
    r.write("openspec/changes/demo/proposal.md", "## Why\n\nx\n");
    r.commit_all();
    r
}

#[test]
fn init_imports_an_openspec_project() {
    let r = openspec_repo("import");
    let out = r.run(&["init", "--tools", "claude"]);
    assert!(out.status.success(), "init: {}", stderr_of(&out));

    assert_eq!(r.read("speclink/config.yaml"), OPENSPEC_CONFIG.as_bytes(), "config.yaml kept byte-for-byte");
    assert_eq!(r.read("speclink/schemas/my-flow/schema.yaml"), b"name: my-flow\n");
    assert_eq!(r.read("speclink/specs/auth/spec.md"), b"# auth\n");
    assert_eq!(r.read("speclink/specs/billing/spec.md"), b"# billing\n");
    assert_eq!(r.read("speclink/changes/demo/.meta.yaml"), b"schema: spec-driven\ncreated: 2026-09-01\n");
    assert!(!r.exists("speclink/changes/demo/.openspec.yaml"));
    assert!(!r.exists("openspec"), "openspec/ is gone");
    assert!(r.exists(".speclink.yaml") && r.exists(MANUAL_SKILL));

    let stdout = stdout_of(&out);
    for needle in ["speclink/", "openspec/", ".speclink.yaml", "OpenSpec CLI", "OpenSpec's own"] {
        assert!(stdout.contains(needle), "stdout names {needle:?}:\n{stdout}");
    }
}

#[test]
fn init_refuses_to_import_with_uncommitted_changes_and_writes_nothing() {
    let r = openspec_repo("import-dirty");
    r.write("openspec/changes/demo/proposal.md", "## Why\n\nchanged\n");

    let out = r.run(&["init", "--tools", "claude"]);
    assert!(!out.status.success(), "init must refuse");
    let stderr = stderr_of(&out);
    assert!(stderr.contains("openspec/") && stderr.contains("uncommitted"), "{stderr}");
    for rel in [".speclink.yaml", "speclink", ".claude"] {
        assert!(!r.exists(rel), "{rel} must not be written");
    }
    assert!(r.exists("openspec/changes/demo/.openspec.yaml"));
}

#[test]
fn init_on_an_initialized_project_keeps_already_initialized() {
    let r = legacy_repo("initialized");
    let out = r.run(&["init", "--tools", "claude"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("Already initialized"), "{}", stderr_of(&out));
    assert!(r.exists("openspec/changes/alpha/.openspec.yaml") && !r.exists("speclink"));
    assert!(!r.exists(".claude"), "nothing is generated");
}

// --- sharp edges (task 8.1) ---

#[test]
fn without_git_on_path_update_skips_the_checks_and_migrates() {
    // design Risks「git 不在 PATH」：視為不在 git repo，跳過三項檢查照常遷移（既定取捨）。
    let r = legacy_repo("no-git-on-path");
    r.write("openspec/changes/alpha/tasks.md", "- [x] 1.1 x\n");
    let out = Command::new(env!("CARGO_BIN_EXE_speclink"))
        .arg("update")
        .current_dir(&r.dir)
        .env("PATH", "")
        .env("USERPROFILE", &r.home)
        .env("HOME", &r.home)
        .env("XDG_CONFIG_HOME", &r.home)
        .output()
        .expect("run speclink binary");
    assert!(out.status.success(), "update: {}", stderr_of(&out));
    assert!(r.exists("speclink/changes/alpha/.meta.yaml") && !r.exists("openspec"));
}

// --- round 1 findings ---

#[test]
fn update_in_an_openspec_project_moves_nothing_and_points_at_init() {
    let r = openspec_repo("update-openspec");
    let out = r.run(&["update"]);
    assert!(out.status.success(), "update: {}", stderr_of(&out));
    assert!(r.exists("openspec/changes/demo/.openspec.yaml") && !r.exists("speclink"));
    assert!(!r.exists(".speclink.yaml"), "update writes no config");
    assert!(stdout_of(&out).contains("speclink init"), "{}", stdout_of(&out));
}

#[test]
fn init_force_on_a_legacy_project_reinitializes_in_place() {
    let r = legacy_repo("force-legacy");
    let out = r.run(&["init", "--tools", "claude", "--force"]);
    assert!(out.status.success(), "init --force: {}", stderr_of(&out));
    assert!(!r.exists("speclink"), "no second, empty spec folder");
    assert!(r.exists("openspec/changes/alpha/.openspec.yaml"));
    let app = String::from_utf8(r.read(".speclink.yaml")).unwrap();
    assert!(
        !app.lines().any(|l| l.trim_start().starts_with("spec_dir:")),
        "the project stays on the legacy layout, free to migrate later: {app}"
    );
    let list = r.run(&["list", "--json"]);
    assert!(stdout_of(&list).contains("alpha"), "openspec/ is still read: {}", stdout_of(&list));
}
