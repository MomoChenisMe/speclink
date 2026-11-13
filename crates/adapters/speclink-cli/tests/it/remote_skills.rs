//! `speclink update` in a remote project takes its skills from the Server
//! (spec workspace-tools「remote 專案的技能取自 Server」, and the remote
//! scenarios of「worktree 技能的政策條件式生成」and「受管檔再生的降級守門」).
//!
//! Two kinds of server stand in for the real one. The door (`door.rs`) is the
//! host kit's HTTP door on a memory store: what it gives is what a real Server gives.
//! The fake answers fixed text, for the cases a real Server of this version
//! cannot make: no skills declared, another skill version, a failing route.
//!
//! Credential isolation: every run points HOME, USERPROFILE and
//! XDG_CONFIG_HOME at a throwaway folder; the credential is `SPECLINK_TOKEN`.

use crate::door::{door, Door, PROJECT};
use crate::fake_server::Fake;
use speclink_core::config::{CustomTool, Invocation};
use speclink_core::init::{managed_skills, ASSET_VERSION};
use speclink_core::skills::{RenderTarget, SpecLocation, Tool};
use speclink_core::workspace::DEFAULT_SPEC_DIR;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const WORKTREE_SKILLS: [&str; 2] = ["speclink-apply-with-worktree", "speclink-worktree-merge"];

/// 2026-11-12T03:00:00Z, in milliseconds.
const AT_MS: u64 = 1_794_452_400_000;

/// Write the workflow config of the door's scope.
fn set_config(door: &Door, content: &str) {
    door.put("config.yaml", content, AT_MS);
}

// --- the fake: fixed answers ---

fn binding(skills_version: Option<&str>) -> String {
    let capabilities = match skills_version {
        Some(version) => format!(r#"{{"skills":{{"version":"{version}"}}}}"#),
        None => "{}".to_string(),
    };
    format!(
        r#"{{"actor":{{"id":"u_1","name":"Tester"}},"project":{{"id":"prj_demo","key":"demo","name":"Demo"}},"repo":{{"id":"repo_backend","key":"backend","name":"Backend"}},"apiVersion":"1","engineVersion":"0.12.0","capabilities":{capabilities}}}"#
    )
}

/// A skill file as a Server of `version` would give it, with `mark` as its body.
fn served_skill(name: &str, version: &str, mark: &str) -> String {
    format!("---\nname: speclink-{name}\nmetadata:\n  version: \"{version}\"\n---\n\n{mark}\n")
}

/// A skills response of `version`: each tool gets `names`, each skill's body is the tool's name.
fn skills_body(version: &str, tools: &[&str], names: &[&str]) -> String {
    let targets: Vec<serde_json::Value> = tools
        .iter()
        .map(|tool| {
            let skills: Vec<serde_json::Value> = names
                .iter()
                .map(|name| {
                    serde_json::json!({ "name": name, "digest": "sha256:00", "content": served_skill(name, version, tool) })
                })
                .collect();
            serde_json::json!({ "tool": tool, "skills": skills })
        })
        .collect();
    serde_json::json!({ "version": version, "targets": targets }).to_string()
}

/// A fake that declares skills of `version` and gives `names` to each of `tools`.
fn serving(fake: &Fake, version: &str, tools: &[&str], names: &[&str]) {
    fake.route(&format!("{PROJECT}/binding"), 200, &binding(Some(version)));
    fake.route(&format!("{PROJECT}/skills"), 200, &skills_body(version, tools, names));
}

/// A fake of a Server from before skills: its handshake declares none.
fn old_server() -> Fake {
    let fake = Fake::new();
    fake.route(&format!("{PROJECT}/binding"), 200, &binding(None));
    fake
}

// --- the checkout ---

struct Checkout {
    base: PathBuf,
    dir: PathBuf,
    home: PathBuf,
}

impl Checkout {
    /// A checkout that holds only `.speclink.yaml`: `tools` (YAML list lines) and,
    /// with a `url`, the connection to it.
    fn new(tag: &str, tools: &str, url: Option<&str>) -> Checkout {
        let base = std::env::temp_dir().join(format!("speclink-cli-remote-skills-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let (dir, home) = (base.join("project"), base.join("home"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        let remote = url.map(|url| format!("remote:\n  url: {url}\n  repo: backend\n")).unwrap_or_default();
        std::fs::write(dir.join(".speclink.yaml"), format!("tools:\n{tools}{remote}")).unwrap();
        Checkout { base, dir, home }
    }

    /// claude and codex, connected to `url`.
    fn remote(tag: &str, url: &str) -> Checkout {
        Checkout::new(tag, "  - claude\n  - codex\n", Some(url))
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args).current_dir(&self.dir);
        for key in [
            "SPECLINK_LOCALE",
            "SPECLINK_SPEC_LOCALE",
            "SPECLINK_STORE_URL",
            "SPECLINK_WORKTREE",
            "SPECLINK_TOKEN",
            "NO_COLOR",
            "CLICOLOR",
            "CLICOLOR_FORCE",
        ] {
            cmd.env_remove(key);
        }
        cmd.env("USERPROFILE", &self.home).env("HOME", &self.home).env("XDG_CONFIG_HOME", &self.home);
        cmd
    }

    /// Run with a credential.
    fn run(&self, args: &[&str]) -> Output {
        self.command(args).env("SPECLINK_TOKEN", "tok").output().expect("run speclink binary")
    }

    /// Run with no credential anywhere.
    fn run_without_credential(&self, args: &[&str]) -> Output {
        self.command(args).output().expect("run speclink binary")
    }

    /// The managed skill files under `skills_dir`: `speclink-<name>` → SKILL.md text.
    fn skills(&self, skills_dir: &str) -> BTreeMap<String, String> {
        let root = self.dir.join(skills_dir.split('/').collect::<PathBuf>());
        let Ok(entries) = std::fs::read_dir(&root) else {
            return BTreeMap::new();
        };
        entries
            .flatten()
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                let text = std::fs::read_to_string(entry.path().join("SKILL.md")).ok()?;
                name.starts_with("speclink-").then_some((name, text))
            })
            .collect()
    }

    /// Every file of the checkout with its bytes.
    fn snapshot(&self) -> Vec<(String, Vec<u8>)> {
        fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, Vec<u8>)>) {
            for entry in std::fs::read_dir(dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, root, out);
                } else {
                    let rel = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
                    out.push((rel, std::fs::read(&path).unwrap()));
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.dir, &self.dir, &mut out);
        out.sort();
        out
    }
}

impl Drop for Checkout {
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

fn assert_ok(out: &Output) {
    assert!(out.status.success(), "exit {:?}\nstdout: {}\nstderr: {}", out.status.code(), stdout_of(out), stderr_of(out));
}

/// What this CLI's own templates give `target` in a checkout without the spec folder.
fn local_render(target: RenderTarget, worktree_on: bool) -> BTreeMap<String, String> {
    managed_skills(target, worktree_on, DEFAULT_SPEC_DIR, SpecLocation::Elsewhere).into_iter().collect()
}

/// The one stderr line that says the skills came from this CLI's templates.
fn fallback_line(out: &Output) -> String {
    let stderr = stderr_of(out);
    let lines: Vec<&str> = stderr.lines().filter(|line| line.contains("this CLI's templates")).collect();
    assert_eq!(lines.len(), 1, "one line says why the local templates were used: {stderr:?}");
    lines[0].to_string()
}

// --- 自 Server 安裝 ---

#[test]
fn update_installs_the_servers_skills_for_each_tool() {
    let fake = Fake::new();
    serving(&fake, "v1.58.0", &["claude", "codex"], &["apply", "discuss"]);
    let checkout = Checkout::remote("install", &fake.project_url());

    let out = checkout.run(&["update"]);
    assert_ok(&out);
    for (skills_dir, tool) in [(".claude/skills", "claude"), (".agents/skills", "codex")] {
        let expected: BTreeMap<String, String> = ["apply", "discuss"]
            .iter()
            .map(|name| (format!("speclink-{name}"), served_skill(name, "v1.58.0", tool)))
            .collect();
        assert_eq!(checkout.skills(skills_dir), expected, "{skills_dir} holds what the Server gave {tool}");
    }
    let stdout = stdout_of(&out);
    assert!(stdout.contains("Updated skill files for: claude, codex"), "{stdout}");
    assert!(stdout.contains("Skills source: server (v1.58.0)"), "{stdout}");
    assert!(!stderr_of(&out).contains("this CLI's templates"), "no fallback: {}", stderr_of(&out));
}

#[test]
fn update_installs_what_a_real_server_gives() {
    let door = door(&[], AT_MS);
    set_config(&door, "schema: spec-driven\nworktree: true\n");
    let checkout = Checkout::remote("real", &door.base);

    let out = checkout.run(&["update"]);
    assert_ok(&out);
    assert_eq!(checkout.skills(".claude/skills"), local_render(RenderTarget::Builtin(Tool::Claude), true));
    assert_eq!(checkout.skills(".agents/skills"), local_render(RenderTarget::Builtin(Tool::Codex), true));
    assert!(stdout_of(&out).contains(&format!("Skills source: server ({ASSET_VERSION})")), "{}", stdout_of(&out));
    assert_eq!(door.asked(), ["GET /binding", "POST /skills"], "one handshake, one skills request");
}

// --- 退回本機範本 ---

#[test]
fn update_uses_the_local_templates_when_the_server_declares_no_skills() {
    let fake = old_server();
    let checkout = Checkout::remote("old-server", &fake.project_url());

    let out = checkout.run(&["update"]);
    assert_ok(&out);
    assert_eq!(checkout.skills(".claude/skills"), local_render(RenderTarget::Builtin(Tool::Claude), false));
    assert_eq!(checkout.skills(".agents/skills"), local_render(RenderTarget::Builtin(Tool::Codex), false));
    assert!(fallback_line(&out).contains("does not provide skills"), "{}", stderr_of(&out));
    assert!(stdout_of(&out).contains(&format!("Skills source: local ({ASSET_VERSION})")), "{}", stdout_of(&out));
    assert!(!fake.paths().iter().any(|path| path.ends_with("/skills")), "no skills request: {:?}", fake.paths());
}

#[test]
fn update_uses_the_local_templates_when_the_server_cannot_be_reached() {
    // A port nothing listens on: bind one, then close it.
    let closed = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        format!("http://{}{PROJECT}", listener.local_addr().unwrap())
    };
    let checkout = Checkout::remote("unreachable", &closed);

    let out = checkout.run(&["update"]);
    assert_ok(&out);
    assert_eq!(checkout.skills(".claude/skills"), local_render(RenderTarget::Builtin(Tool::Claude), false));
    assert!(fallback_line(&out).contains("unreachable"), "{}", stderr_of(&out));
    assert!(stdout_of(&out).contains("Skills source: local"), "{}", stdout_of(&out));
}

#[test]
fn update_uses_the_local_templates_without_a_credential() {
    let fake = Fake::new();
    serving(&fake, "v1.58.0", &["claude", "codex"], &["apply"]);
    let checkout = Checkout::remote("no-credential", &fake.project_url());

    let out = checkout.run_without_credential(&["update"]);
    assert_ok(&out);
    assert_eq!(checkout.skills(".claude/skills"), local_render(RenderTarget::Builtin(Tool::Claude), false));
    assert!(fallback_line(&out).contains("speclink auth login"), "{}", stderr_of(&out));
    assert!(fake.paths().is_empty(), "nothing was asked without a credential: {:?}", fake.paths());
}

#[test]
fn update_uses_the_local_templates_when_the_server_refuses_the_handshake() {
    let fake = Fake::new();
    fake.route(
        &format!("{PROJECT}/binding"),
        403,
        r#"{"status":403,"reason":"permission_denied","message":"you are not a member of project 'demo'"}"#,
    );
    let checkout = Checkout::remote("refused", &fake.project_url());

    let out = checkout.run(&["update"]);
    assert_ok(&out);
    assert_eq!(checkout.skills(".claude/skills"), local_render(RenderTarget::Builtin(Tool::Claude), false));
    assert!(fallback_line(&out).contains("access denied"), "{}", stderr_of(&out));
    assert!(stdout_of(&out).contains("Skills source: local"), "{}", stdout_of(&out));
    assert!(!fake.paths().iter().any(|path| path.ends_with("/skills")), "no skills request: {:?}", fake.paths());
}

// --- 宣告了卻取不到 ---

#[test]
fn update_fails_and_writes_nothing_when_the_declared_skills_cannot_be_fetched() {
    let fake = old_server();
    let checkout = Checkout::remote("declared-500", &fake.project_url());
    assert_ok(&checkout.run(&["update"]));
    let before = checkout.snapshot();

    fake.route(&format!("{PROJECT}/binding"), 200, &binding(Some("v1.58.0")));
    fake.route(&format!("{PROJECT}/skills"), 500, "{}");
    let out = checkout.run(&["update"]);
    assert!(!out.status.success(), "stdout: {}", stdout_of(&out));
    let stderr = stderr_of(&out);
    assert!(stderr.contains("could not get the skills from the server"), "{stderr}");
    assert!(!stderr.contains("this CLI's templates"), "no fallback to the local templates: {stderr}");
    assert_eq!(checkout.snapshot(), before, "every file is as it was");
}

// --- 清理 ---

#[test]
fn update_removes_a_skill_the_server_no_longer_gives() {
    let fake = old_server();
    let checkout = Checkout::remote("prune", &fake.project_url());
    assert_ok(&checkout.run(&["update"]));
    assert!(checkout.skills(".claude/skills").contains_key("speclink-improve"), "the local templates have improve");

    serving(&fake, ASSET_VERSION, &["claude", "codex"], &["apply"]);
    assert_ok(&checkout.run(&["update"]));
    let left: Vec<String> = checkout.skills(".claude/skills").into_keys().collect();
    assert_eq!(left, ["speclink-apply"], "speclink-improve and the rest are gone");
    assert!(!checkout.dir.join(".claude").join("skills").join("speclink-improve").exists());
}

// --- 自訂工具 ---

#[test]
fn update_installs_a_custom_tools_skills_from_the_server() {
    let door = door(&[], AT_MS);
    let tools = "  - name: my-agent\n    skills_dir: .my-agent/skills\n    invocation: tool-call\n";
    let checkout = Checkout::new("custom", tools, Some(&door.base));

    assert_ok(&checkout.run(&["update"]));
    let custom = CustomTool {
        name: "my-agent".into(),
        skills_dir: ".my-agent/skills".into(),
        instructions_file: None,
        invocation: Invocation::ToolCall,
    };
    let got = checkout.skills(".my-agent/skills");
    assert_eq!(got, local_render(RenderTarget::Custom(&custom), false));
    assert!(got["speclink-apply"].contains("calling the speclink tool"), "the neutral tool-call render");
    assert_eq!(door.asked(), ["GET /binding", "POST /skills"]);
}

// --- 本機專案不連線 ---

#[test]
fn update_of_a_local_project_asks_no_server_and_prints_what_it_did_before() {
    let checkout = Checkout::new("local", "  - claude\n", None);
    std::fs::create_dir_all(checkout.dir.join("speclink").join("changes")).unwrap();

    let out = checkout.run(&["update"]);
    assert_ok(&out);
    assert_eq!(stdout_of(&out), "✓ Updated skill files for: claude\n");
    assert_eq!(stderr_of(&out), "");
    let in_checkout: BTreeMap<String, String> =
        managed_skills(RenderTarget::Builtin(Tool::Claude), false, DEFAULT_SPEC_DIR, SpecLocation::InCheckout)
            .into_iter()
            .collect();
    assert_eq!(checkout.skills(".claude/skills"), in_checkout);
}

// --- worktree 技能跟著 Server 的流程設定 ---

#[test]
fn update_follows_the_servers_worktree_policy_on_and_then_off() {
    let door = door(&[], AT_MS);
    set_config(&door, "schema: spec-driven\nworktree: true\n");
    let checkout = Checkout::remote("worktree", &door.base);

    assert_ok(&checkout.run(&["update"]));
    for skills_dir in [".claude/skills", ".agents/skills"] {
        for skill in WORKTREE_SKILLS {
            assert!(checkout.skills(skills_dir).contains_key(skill), "{skills_dir}/{skill} after the policy is on");
        }
    }

    set_config(&door, "schema: spec-driven\nworktree: false\n");
    assert_ok(&checkout.run(&["update"]));
    for skills_dir in [".claude/skills", ".agents/skills"] {
        let left = checkout.skills(skills_dir);
        for skill in WORKTREE_SKILLS {
            assert!(!left.contains_key(skill), "{skills_dir}/{skill} after the policy is off");
        }
        assert!(left.contains_key("speclink-apply"), "the other skills stay in {skills_dir}");
    }
}

// --- 降級守門 ---

#[test]
fn update_takes_an_older_version_from_the_server_and_says_so() {
    let fake = Fake::new();
    serving(&fake, "v1.60.0", &["claude", "codex"], &["apply"]);
    let checkout = Checkout::remote("older", &fake.project_url());
    assert_ok(&checkout.run(&["update"]));
    let apply = checkout.dir.join(".claude").join("skills").join("speclink-apply").join("SKILL.md");
    assert_eq!(speclink_core::testkit::skill_frontmatter_version(&apply), "v1.60.0");

    serving(&fake, "v1.58.0", &["claude", "codex"], &["apply"]);
    let out = checkout.run(&["update"]);
    assert_ok(&out);
    assert_eq!(speclink_core::testkit::skill_frontmatter_version(&apply), "v1.58.0");
    let stdout = stdout_of(&out);
    assert!(
        stdout.contains("Skills replaced with the server's version v1.58.0 (this checkout had v1.60.0)"),
        "{stdout}"
    );
    assert!(stdout.contains("Skills source: server (v1.58.0)"), "{stdout}");
}

#[test]
fn update_keeps_the_downgrade_guard_when_it_uses_the_local_templates() {
    let fake = old_server();
    let checkout = Checkout::remote("guard", &fake.project_url());
    assert_ok(&checkout.run(&["update"]));
    speclink_core::testkit::set_skill_version(&checkout.dir.join(".claude").join("skills"), "v99.0.0");
    let before = checkout.snapshot();

    let out = checkout.run(&["update"]);
    assert!(!out.status.success(), "stdout: {}", stdout_of(&out));
    let stderr = stderr_of(&out);
    assert!(
        stderr.contains(&format!("Workspace skill files (v99.0.0) are newer than this engine ({ASSET_VERSION})")),
        "{stderr}"
    );
    assert_eq!(checkout.snapshot(), before, "nothing was written");
}

// --- init --store remote 的提示 ---

#[test]
fn init_remote_ends_with_the_hint_to_log_in_and_update() {
    let fake = old_server();
    let checkout = Checkout::new("init-hint", "", None);
    std::fs::remove_file(checkout.dir.join(".speclink.yaml")).unwrap();

    let out = checkout.run_without_credential(&["init", "--store", "remote", "--url", &fake.project_url(), "--tools", "claude"]);
    assert_ok(&out);
    let stdout = stdout_of(&out);
    let last = stdout.lines().last().unwrap_or_default();
    assert!(
        last.contains("speclink auth login") && last.contains("speclink update") && last.contains("server"),
        "the last line is the hint: {stdout:?}"
    );
    assert_eq!(checkout.skills(".claude/skills"), local_render(RenderTarget::Builtin(Tool::Claude), false), "init still uses the local templates");
}

// --- CLI 的技能版本提醒 (spec workspace-tools「CLI 的技能版本提醒」) ---
//
// The day and the record's failures are tested where the day can be given
// (`src/skill_notice.rs`); here the real binary shows where the notice is
// printed and where it is not.

/// A skill version newer than any this CLI can have.
const NEWER: &str = "v999.0.0";

/// A fake that declares `skills_version` and has an empty change list.
fn listing(skills_version: Option<&str>) -> Fake {
    let fake = Fake::new();
    fake.route(&format!("{PROJECT}/binding"), 200, &binding(skills_version));
    fake.route(&format!("{PROJECT}/changes"), 200, r#"{"changes":[]}"#);
    fake
}

/// The stderr lines that are the skill version notice.
fn notices(out: &Output) -> Vec<String> {
    stderr_of(out).lines().filter(|line| line.contains("skill templates")).map(str::to_string).collect()
}

#[test]
fn a_remote_verb_tells_an_older_cli_once_a_day_for_each_server() {
    let fake = listing(Some(NEWER));
    let checkout = Checkout::remote("notice", &fake.project_url());

    let first = checkout.run(&["list", "--json"]);
    assert_ok(&first);
    let told = notices(&first);
    assert_eq!(told.len(), 1, "stderr: {}", stderr_of(&first));
    assert!(
        told[0].contains(ASSET_VERSION) && told[0].contains(NEWER) && told[0].contains("update Speclink: npm i -g @speclink/cli"),
        "both versions and how to update: {}",
        told[0]
    );
    let _: serde_json::Value = serde_json::from_slice(&first.stdout).expect("stdout is still the list's JSON");

    let second = checkout.run(&["list", "--json"]);
    assert_ok(&second);
    assert_eq!(notices(&second), Vec::<String>::new(), "the same day, the same Server");
    assert_eq!(second.stdout, first.stdout, "the notice changes no stdout");

    // Another Server, the same user: it is told on its own.
    let other = listing(Some(NEWER));
    let config = format!("tools:\n  - claude\nremote:\n  url: {}\n  repo: backend\n", other.project_url());
    std::fs::write(checkout.dir.join(".speclink.yaml"), config).unwrap();
    assert_eq!(notices(&checkout.run(&["list", "--json"])).len(), 1, "another Server");

    // The record is in the user's config directory, not in the project, and holds no credential.
    let in_project: Vec<String> = checkout.snapshot().into_iter().map(|(file, _)| file).collect();
    assert_eq!(in_project, [".speclink.yaml"], "nothing was written into the project");
    let mut records = Vec::new();
    collect_named(&checkout.home, "skill-notice.json", &mut records);
    assert_eq!(records.len(), 1, "one record under the user's home: {records:?}");
    let record = std::fs::read_to_string(&records[0]).unwrap();
    assert!(!record.contains("tok"), "{record}");
}

fn collect_named(dir: &Path, name: &str, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_named(&path, name, out);
        } else if path.file_name().is_some_and(|file| file == name) {
            out.push(path);
        }
    }
}

#[test]
fn a_remote_verb_tells_nothing_when_this_cli_is_not_older() {
    for (tag, declared) in [("same", Some(ASSET_VERSION)), ("older", Some("v0.0.1")), ("none", None), ("garbage", Some("latest"))] {
        let fake = listing(declared);
        let checkout = Checkout::remote(&format!("no-notice-{tag}"), &fake.project_url());
        let out = checkout.run(&["list", "--json"]);
        assert_ok(&out);
        assert_eq!(notices(&out), Vec::<String>::new(), "{declared:?}");
    }
}

#[test]
fn update_prints_no_notice_and_does_not_use_up_the_day() {
    let fake = listing(Some(NEWER));
    fake.route(&format!("{PROJECT}/skills"), 200, &skills_body(NEWER, &["claude", "codex"], &["apply"]));
    let checkout = Checkout::remote("notice-update", &fake.project_url());

    let update = checkout.run(&["update"]);
    assert_ok(&update);
    assert_eq!(notices(&update), Vec::<String>::new(), "update's own summary has the version");
    assert!(stdout_of(&update).contains(&format!("Skills source: server ({NEWER})")), "{}", stdout_of(&update));

    assert_eq!(notices(&checkout.run(&["list", "--json"])).len(), 1, "the first verb after it is still told");
}

// --- sharp-edges audit: what a Server answers cannot leave the skills directories ---

#[test]
fn audit_an_answer_with_a_name_that_is_a_path_writes_nothing() {
    let fake = old_server();
    let checkout = Checkout::remote("audit-path", &fake.project_url());
    assert_ok(&checkout.run(&["update"]));
    let before = checkout.snapshot();

    for bad in ["../../escape", "a/b", "..", ""] {
        fake.route(&format!("{PROJECT}/binding"), 200, &binding(Some("v1.58.0")));
        fake.route(&format!("{PROJECT}/skills"), 200, &skills_body("v1.58.0", &["claude", "codex"], &["apply", bad]));
        let out = checkout.run(&["update"]);
        assert!(!out.status.success(), "{bad:?}: stdout: {}", stdout_of(&out));
        assert!(stderr_of(&out).contains("could not get the skills from the server"), "{bad:?}: {}", stderr_of(&out));
        assert_eq!(checkout.snapshot(), before, "{bad:?}: every file is as it was");
    }
    assert!(!checkout.base.join("escape").exists() && !checkout.dir.join("escape").exists());
}

#[test]
fn audit_an_answer_for_another_tool_or_without_text_writes_nothing() {
    let fake = old_server();
    let checkout = Checkout::remote("audit-fit", &fake.project_url());
    assert_ok(&checkout.run(&["update"]));
    let before = checkout.snapshot();
    fake.route(&format!("{PROJECT}/binding"), 200, &binding(Some("v1.58.0")));

    // A tool the checkout did not ask for, and one it asked for left out.
    fake.route(&format!("{PROJECT}/skills"), 200, &skills_body("v1.58.0", &["claude", "cursor"], &["apply"]));
    let out = checkout.run(&["update"]);
    assert!(!out.status.success(), "stdout: {}", stdout_of(&out));
    assert_eq!(checkout.snapshot(), before);

    // Digests only where the texts were asked for.
    let digests_only = r#"{"version":"v1.58.0","targets":[{"tool":"claude","skills":[{"name":"apply","digest":"sha256:00"}]},{"tool":"codex","skills":[]}]}"#;
    fake.route(&format!("{PROJECT}/skills"), 200, digests_only);
    let out = checkout.run(&["update"]);
    assert!(!out.status.success(), "stdout: {}", stdout_of(&out));
    assert!(stderr_of(&out).contains("without its text"), "{}", stderr_of(&out));
    assert_eq!(checkout.snapshot(), before);
}

// --- Server 對某個工具不提供任何技能（design D9） ---

/// A skills response of `version` that gives claude `apply` and codex nothing.
fn claude_only_body(version: &str) -> String {
    serde_json::json!({
        "version": version,
        "targets": [
            { "tool": "claude", "skills": [{ "name": "apply", "digest": "sha256:00", "content": served_skill("apply", version, "claude") }] },
            { "tool": "codex", "skills": [] }
        ]
    })
    .to_string()
}

#[test]
fn update_leaves_a_tool_the_server_gives_no_skill_as_it_is() {
    let fake = old_server();
    let checkout = Checkout::remote("unserved", &fake.project_url());
    assert_ok(&checkout.run(&["update"]));
    let codex_before = checkout.skills(".agents/skills");
    assert!(codex_before.contains_key("speclink-apply"), "codex has skills from the local templates");

    fake.route(&format!("{PROJECT}/binding"), 200, &binding(Some("v1.58.0")));
    fake.route(&format!("{PROJECT}/skills"), 200, &claude_only_body("v1.58.0"));
    let out = checkout.run(&["update"]);
    assert_ok(&out);

    assert_eq!(checkout.skills(".agents/skills"), codex_before, "codex: every file is as it was");
    let claude: Vec<(String, String)> = checkout.skills(".claude/skills").into_iter().collect();
    assert_eq!(claude, [("speclink-apply".to_string(), served_skill("apply", "v1.58.0", "claude"))]);
    let stdout = stdout_of(&out);
    assert!(stdout.contains("Updated skill files for: claude\n"), "only claude was updated: {stdout}");
    assert!(
        stdout.contains("! The server gives no skills for: codex — their skill files were left as they are\n"),
        "{stdout}"
    );
    assert!(stdout.contains("Skills source: server (v1.58.0)"), "{stdout}");
}

#[test]
fn update_writes_nothing_when_the_server_gives_no_tool_a_skill() {
    let fake = old_server();
    let checkout = Checkout::remote("all-unserved", &fake.project_url());
    assert_ok(&checkout.run(&["update"]));
    let before = checkout.snapshot();

    fake.route(&format!("{PROJECT}/binding"), 200, &binding(Some("v1.58.0")));
    fake.route(&format!("{PROJECT}/skills"), 200, &skills_body("v1.58.0", &["claude", "codex"], &[]));
    let out = checkout.run(&["update"]);
    assert_ok(&out);
    assert_eq!(checkout.snapshot(), before, "no file changed");
    let stdout = stdout_of(&out);
    assert!(stdout.contains("! The server gives no skills for: claude, codex — their skill files were left as they are\n"), "{stdout}");
    assert!(!stdout.contains("No AI tool configurations found"), "the tools are configured: {stdout}");
    assert!(!stdout.contains("Updated skill files for"), "{stdout}");
}
