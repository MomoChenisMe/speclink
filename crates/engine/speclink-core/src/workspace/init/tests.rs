use super::*;

/// Throwaway project root, removed on drop.
struct TempRoot {
    dir: PathBuf,
}

impl TempRoot {
    fn new(tag: &str) -> TempRoot {
        let dir = std::env::temp_dir().join(format!(
            "speclink-init-test-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        TempRoot { dir }
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.dir.join(rel.split('/').collect::<PathBuf>())).unwrap()
    }

    fn at(&self, rel: &str) -> PathBuf {
        self.dir.join(rel.split('/').collect::<PathBuf>())
    }

    fn write(&self, rel: &str, content: &str) {
        let path = self.at(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    fn exists(&self, rel: &str) -> bool {
        self.at(rel).exists()
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

// Spec requirement: init 範本的政策寫入位置 — policy examples live in the
// openspec/config.yaml template; the .speclink.yaml template carries none.

#[test]
fn init_workflow_config_template_has_commented_policy_examples() {
    let root = TempRoot::new("wf-template");
    init(&root.dir, &[], false, "openspec").unwrap();
    let wf = root.read("openspec/config.yaml");
    for example in ["# locale:", "# spec_locale:", "# tdd:", "# audit:"] {
        assert!(
            wf.contains(example),
            "config.yaml template must show a commented {example} example:\n{wf}"
        );
    }
}

#[test]
fn init_app_config_template_has_no_policy_keys() {
    let root = TempRoot::new("app-template");
    init(&root.dir, &[], false, "openspec").unwrap();
    let app = root.read(".speclink.yaml");
    // Not even commented examples: the workspace file must not teach policy keys.
    // ("locale" also catches "spec_locale".)
    for word in ["locale", "tdd", "audit"] {
        assert!(
            !app.contains(word),
            ".speclink.yaml template must not mention policy key {word}:\n{app}"
        );
    }
    // Its actual concerns stay: workspace binding (tools) and spec_dir.
    assert!(app.contains("tools"));
    assert!(app.contains("spec_dir"));
}

#[test]
fn init_app_config_template_links_the_real_repository() {
    let root = TempRoot::new("app-template-url");
    init(&root.dir, &[], false, "openspec").unwrap();
    let app = root.read(".speclink.yaml");
    assert!(
        app.contains("https://github.com/MomoChenisMe/speclink"),
        ".speclink.yaml template must link the project repository:\n{app}"
    );
    assert!(
        !app.contains("speclink-app"),
        ".speclink.yaml template must not link the wrong organization:\n{app}"
    );
}

#[test]
fn init_over_a_speclink_entry_that_is_no_spec_folder_names_it() {
    // 同名的 speclink 不是規格資料夾（例如同名的 repo clone）：說清楚原因、零寫入，
    // 不回誤導的「Already initialized」。
    let root = TempRoot::new("init-speclink-taken");
    root.write("speclink/src/main.rs", "fn main() {}\n");
    let before = snapshot(&root);
    let Err(err) = init(&root.dir, &[Tool::Claude], false, DEFAULT_SPEC_DIR) else {
        panic!("init must refuse");
    };
    let err = err.to_string();
    assert!(err.contains("speclink") && err.contains("not a spec folder"), "{err}");
    assert_eq!(snapshot(&root), before, "零寫入");
}

#[test]
fn init_with_tools_records_selection_without_policy_keys() {
    let root = TempRoot::new("app-tools");
    init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();
    let app = root.read(".speclink.yaml");
    assert!(app.contains("tools: [claude, codex]"));
    for word in ["locale", "tdd", "audit"] {
        assert!(!app.contains(word), "no policy key {word} expected:\n{app}");
    }
}

// --- 共用 built-in tools reconciliation ---
// Spec requirement: 「built-in tools 權威收斂」＋ design「Core 單一 Workspace 工具同步入口」
// ／「Built-in 選擇收斂且保留自訂描述子」與 Implementation Contract 的
// 「Core and configuration contract」。

const CUSTOM_DESCRIPTOR: &str = "  - name: wad-harness\n    skills_dir: .wad/skills\n    instructions_file: WAD.md\n";
/// 第二份描述子夾具：規格「技能檔過期探測」Example 的字面值（name my-harness、
/// skills_dir .my-harness/skills），且不帶已棄用的 instructions_file——探測測試要的是
/// 乾淨的描述子，不是剝除路徑。
const MY_HARNESS_DESCRIPTOR: &str = "  - name: my-harness\n    skills_dir: .my-harness/skills\n";
const REMOTE_URL: &str = "https://team.example.test/api/speclink/v1/projects/acme";
const CLAUDE_USER_TEXT: &str = "使用者寫在 CLAUDE.md 的段落";
const CODEX_USER_TEXT: &str = "使用者寫在 AGENTS.md 的段落";

fn instructions_file(tool: Tool) -> &'static str {
    instructions_path(tool).expect("只有 claude 與 codex 有指令檔")
}

fn user_text(tool: Tool) -> &'static str {
    match tool {
        Tool::Claude => CLAUDE_USER_TEXT,
        Tool::Codex => CODEX_USER_TEXT,
        Tool::Copilot | Tool::Pi | Tool::Cursor => unreachable!("{} 沒有指令檔", tool.name()),
    }
}

fn propose_skill(tool: Tool) -> String {
    format!("{}/speclink-propose/SKILL.md", tool.skills_dir())
}

/// Remote 模式 workspace，其 `.speclink.yaml` 除 built-in 選集外還帶 custom
/// descriptor、remote section 與未知頂層鍵；兩份指令檔先有使用者自有文字。
fn seed_remote_workspace(root: &TempRoot, builtins: &[Tool]) {
    root.write("CLAUDE.md", &format!("{CLAUDE_USER_TEXT}\n"));
    root.write("AGENTS.md", &format!("{CODEX_USER_TEXT}\n"));
    let listed: String = builtins.iter().map(|t| format!("  - {}\n", t.name())).collect();
    root.write(
        ".speclink.yaml",
        &format!(
            "tools:\n{listed}{CUSTOM_DESCRIPTOR}remote:\n  url: {REMOTE_URL}\n  repo: desktop\nfuture_top_level: keep me\n"
        ),
    );
    update(&root.dir, false).expect("seed update");
}

fn builtin_names(root: &TempRoot) -> Vec<String> {
    let app = crate::config::AppConfig::load(&root.at(".speclink.yaml")).expect("config parses");
    let mut names: Vec<String> = app
        .tools
        .iter()
        .filter_map(|e| match e {
            ToolEntry::Builtin(n) => Some(n.clone()),
            ToolEntry::Descriptor(_) => None,
        })
        .collect();
    names.sort();
    names
}

fn marker_count(text: &str) -> usize {
    text.matches("<!-- SPECLINK:START").count()
}

/// 目錄快照（檔案內容與目錄項目），供「零寫入」斷言逐位元組比對。
fn snapshot(root: &TempRoot) -> Vec<(String, Vec<u8>)> {
    walk_snapshot(root, false)
}

fn walk_snapshot(root: &TempRoot, skip_git: bool) -> Vec<(String, Vec<u8>)> {
    fn walk(dir: &Path, prefix: &str, skip_git: bool, out: &mut Vec<(String, Vec<u8>)>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            if skip_git && prefix.is_empty() && name == ".git" {
                continue;
            }
            let rel = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };
            // symlink 記錄它指向哪裡、不跟進去：懸空連結讀不到內容，指向目錄的連結也不該
            // 把同一批檔案記兩次。
            if let Ok(target) = std::fs::read_link(entry.path()) {
                out.push((format!("{rel}@"), target.to_string_lossy().as_bytes().to_vec()));
            } else if entry.path().is_dir() {
                out.push((format!("{rel}/"), Vec::new()));
                walk(&entry.path(), &rel, skip_git, out);
            } else {
                out.push((rel, std::fs::read(entry.path()).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(&root.dir, "", skip_git, &mut out);
    out
}

/// Spec example「built-in 選集轉換」逐列：轉換後 built-in 集合等於請求，
/// custom descriptor／未知頂層鍵／remote 原值保留，marker 不重複，使用者文字不動。
#[test]
fn reconcile_converts_builtin_selection_row_by_row() {
    let rows: [(&[Tool], &[Tool]); 10] = [
        (&[Tool::Claude], &[Tool::Codex]),
        (&[Tool::Claude, Tool::Codex], &[Tool::Claude]),
        (&[Tool::Codex], &[Tool::Claude, Tool::Codex]),
        (&[Tool::Claude], &[Tool::Claude, Tool::Copilot]),
        (&[Tool::Claude, Tool::Copilot], &[Tool::Claude]),
        (&[Tool::Codex], &[Tool::Copilot]),
        (&[Tool::Codex], &[Tool::Codex, Tool::Pi]),
        (&[Tool::Codex, Tool::Pi], &[Tool::Codex]),
        (&[Tool::Claude], &[Tool::Claude, Tool::Cursor]),
        (&[Tool::Claude, Tool::Cursor], &[Tool::Claude]),
    ];
    for (i, (from, to)) in rows.iter().enumerate() {
        let root = TempRoot::new(&format!("reconcile-row-{i}"));
        seed_remote_workspace(&root, from);
        // Spec Scenario 的 GIVEN：指令檔「同時含遺留 Speclink 區塊和使用者文字」
        // ——seed 的 update 已把區塊剝掉，reconcile 前重新注入，讓收斂路徑自己剝。
        for tool in [Tool::Claude, Tool::Codex] {
            root.write(
                instructions_file(tool),
                &format!(
                    "<!-- SPECLINK:START v1.0.0 -->\n舊路由表\n<!-- SPECLINK:END -->\n{}\n",
                    user_text(tool)
                ),
            );
        }

        reconcile_builtin_tools(&root.dir, to).expect("reconcile succeeds");

        let want: Vec<String> = {
            let mut v: Vec<String> = to.iter().map(|t| t.name().to_string()).collect();
            v.sort();
            v
        };
        assert_eq!(builtin_names(&root), want, "row {i}: built-in 集合須等於請求");

        let app_text = root.read(".speclink.yaml");
        assert!(app_text.contains("wad-harness"), "row {i}: custom descriptor 須保留:\n{app_text}");
        assert!(app_text.contains("keep me"), "row {i}: 未知頂層鍵須保留:\n{app_text}");
        let app = crate::config::AppConfig::load(&root.at(".speclink.yaml")).expect("config parses");
        let remote = app.remote.as_ref().expect("remote section 須保留");
        assert_eq!(remote.url.as_deref(), Some(REMOTE_URL), "row {i}");
        assert_eq!(remote.repo.as_deref(), Some("desktop"), "row {i}");

        for tool in Tool::ALL {
            let skill = propose_skill(tool);
            if to.contains(&tool) {
                assert!(root.exists(&skill), "row {i}: {skill} 應被補齊");
            } else {
                assert!(!root.exists(&skill), "row {i}: {skill} 應被清理");
            }
            // 指令檔已退出受管：不論選取與否都只剩使用者自己的內容（copilot 沒有指令檔）。
            let Some(md) = instructions_path(tool) else {
                continue;
            };
            let text = root.read(md);
            assert_eq!(marker_count(&text), 0, "row {i}: {md} 不得帶受管區塊:\n{text}");
            assert_eq!(text, format!("{}\n", user_text(tool)), "row {i}: {md} 須位元級不變");
        }
        assert!(!root.exists("openspec"), "row {i}: remote 模式不得建立 openspec/");
    }
}

/// 既有選集缺少產物時自動補齊，其他使用者檔案不受影響。
#[test]
fn reconcile_backfills_missing_managed_artifacts() {
    let root = TempRoot::new("reconcile-backfill");
    seed_remote_workspace(&root, &[Tool::Codex]);
    root.write("AGENTS.md", &format!("{CODEX_USER_TEXT}\n"));
    std::fs::remove_dir_all(root.at(".agents/skills/speclink-propose")).unwrap();
    root.write("docs/notes.md", "使用者檔案\n");

    reconcile_builtin_tools(&root.dir, &[Tool::Codex]).expect("reconcile succeeds");

    let text = root.read("AGENTS.md");
    assert_eq!(text, format!("{CODEX_USER_TEXT}\n"), "指令檔須位元級不變:\n{text}");
    assert!(root.exists(".agents/skills/speclink-propose/SKILL.md"), "缺席的 Skill 應補齊");
    assert_eq!(root.read("docs/notes.md"), "使用者檔案\n");
}

/// 空選集在任何寫入之前被拒（build-in 選集是非空契約）。
#[test]
fn reconcile_rejects_an_empty_selection_without_writing() {
    let root = TempRoot::new("reconcile-empty");
    seed_remote_workspace(&root, &[Tool::Codex]);
    let before = snapshot(&root);

    let err = reconcile_builtin_tools(&root.dir, &[]).expect_err("空選集必須失敗");

    let message = err.to_string();
    assert!(message.contains("claude") && message.contains("codex"), "{message}");
    assert_eq!(snapshot(&root), before, "失敗不得留下任何寫入");
}

/// 壞 YAML 以單行錯誤失敗，設定與受管產物逐位元組不變。
#[test]
fn reconcile_bad_config_fails_loud_with_zero_writes() {
    let root = TempRoot::new("reconcile-bad-yaml");
    seed_remote_workspace(&root, &[Tool::Codex]);
    root.write(".speclink.yaml", "tools: [unclosed\n");
    let before = snapshot(&root);

    let err = reconcile_builtin_tools(&root.dir, &[Tool::Claude]).expect_err("壞 YAML 必須失敗");

    let message = err.to_string();
    assert!(message.contains(".speclink.yaml"), "錯誤須指名檔案：{message}");
    assert_eq!(message.lines().count(), 1, "錯誤須為單行：{message}");
    assert_eq!(snapshot(&root), before, "失敗不得留下任何寫入");
}

/// Remote 模式不生成任何指令檔內容，也不建立本機規格樹。
#[test]
fn reconcile_in_remote_mode_writes_no_instruction_file_and_no_spec_tree() {
    let root = TempRoot::new("reconcile-remote-wording");
    seed_remote_workspace(&root, &[Tool::Claude]);

    reconcile_builtin_tools(&root.dir, &[Tool::Claude, Tool::Codex]).expect("reconcile succeeds");

    for tool in [Tool::Claude, Tool::Codex] {
        let md = instructions_file(tool);
        let text = root.read(md);
        assert_eq!(text, format!("{}\n", user_text(tool)), "{md} 須位元級不變:\n{text}");
    }
    assert!(!root.exists("openspec"), "remote 模式不得建立 openspec/");
}

// --- adopt：工作區補齊入口 ---
// Spec requirement:「工作區補齊入口」（desktop-enable-speclink-prompt；desktop-spec-dir-migration
// D3）——OpenSpec 專案先以 init 匯入的同一套程式改名為 speclink/，再冪等補骨架缺件、寫 tools、
// 生成受管檔；既有規格文件內容不被改寫。

const CUSTOM_WORKFLOW_CONFIG: &str =
    "schema: spec-driven\nlocale: tw\nrules:\n  proposal:\n    - 保持提案精簡\n";

/// 未啟用目錄：openspec/ 內有規格文件、變更、討論與自訂 config.yaml，
/// 但專案根無 .speclink.yaml。
fn seed_unadopted(root: &TempRoot) {
    root.write("openspec/specs/auth/spec.md", "## Purpose\n既有規格文件。\n");
    root.write("openspec/changes/add-auth/proposal.md", "## Why\n既有變更。\n");
    root.write("openspec/changes/add-auth/.openspec.yaml", "schema: spec-driven\ncreated: 2026-10-01\n");
    root.write("openspec/discussions/auth-scope.md", "# Discussion\n既有討論。\n");
    root.write("openspec/config.yaml", CUSTOM_WORKFLOW_CONFIG);
}

/// 目錄快照去掉 `.git/`：git status 會刷新 index，零寫入斷言只看專案檔。
/// 同 snapshot，但不走進 `.git/`：commit 後 git 可能在背景整理，`.git/` 裡的檔案
/// 會在讀到一半時消失。
fn project_snapshot(root: &TempRoot) -> Vec<(String, Vec<u8>)> {
    walk_snapshot(root, true)
}

/// Spec Scenario「匯入後既有內容位元級不變」：openspec/ 改名為 speclink/、資料檔改名為
/// .meta.yaml，內容逐位元組不變；.speclink.yaml 記錄 tools、不寫 spec_dir。
#[test]
fn adopt_imports_an_openspec_project_byte_for_byte() {
    let root = TempRoot::new("adopt-import");
    seed_unadopted(&root);
    commit_all(&root);
    let before = project_snapshot(&root);

    adopt(&root.dir, &[Tool::Claude]).expect("adopt succeeds");

    assert!(!root.exists("openspec"), "openspec/ 必須已改名");
    let after = project_snapshot(&root);
    for (rel, bytes) in before.iter().filter(|(rel, _)| rel.starts_with("openspec/") && !rel.ends_with('/')) {
        let moved = rel.replacen("openspec/", "speclink/", 1).replace(".openspec.yaml", ".meta.yaml");
        let found = after.iter().find(|(r, _)| *r == moved).map(|(_, b)| b);
        assert_eq!(found, Some(bytes), "{rel} 必須位元級搬到 {moved}");
    }
    assert!(!root.exists("speclink/changes/add-auth/.openspec.yaml"));
    let app = root.read(".speclink.yaml");
    assert!(app.contains("claude"), "tools 須記錄 claude：{app}");
    assert!(!app.lines().any(|l| l.trim_start().starts_with("spec_dir")), "不得寫入 spec_dir：{app}");
    assert!(!root.exists("CLAUDE.md"), "工作區補齊不得產生指令檔");
    assert!(root.exists(".claude/skills/speclink-propose/SKILL.md"));
}

/// Spec Scenario「骨架缺件補齊」：缺 specs/ 與 config.yaml 時，匯入後在 speclink/ 補齊目錄與範本。
#[test]
fn adopt_backfills_missing_skeleton() {
    let root = TempRoot::new("adopt-skeleton");
    root.write("openspec/changes/add-auth/proposal.md", "## Why\n既有變更。\n");

    adopt(&root.dir, &[Tool::Claude]).expect("adopt succeeds");

    assert!(!root.exists("openspec"));
    assert!(root.at("speclink/specs").is_dir());
    assert!(root.at("speclink/changes/archive").is_dir());
    assert_eq!(root.read("speclink/config.yaml"), WORKFLOW_CONFIG_TEMPLATE);
}

/// Spec Scenario「有未提交修改時拒絕匯入」：單行錯誤、零寫入；另有 worktree 時同樣拒絕。
#[test]
fn adopt_refuses_an_unsafe_import_with_zero_writes() {
    let root = TempRoot::new("adopt-dirty");
    seed_unadopted(&root);
    commit_all(&root);
    root.write("openspec/changes/add-auth/proposal.md", "## Why\n改到一半。\n");
    let before = project_snapshot(&root);

    let err = adopt(&root.dir, &[Tool::Claude]).expect_err("未提交修改必須拒絕").to_string();
    assert!(err.contains("openspec/ has uncommitted changes"), "{err}");
    assert_eq!(err.lines().count(), 1, "錯誤須為單行：{err}");
    assert_eq!(project_snapshot(&root), before, "拒絕不得留下任何寫入");

    let clean = TempRoot::new("adopt-worktree");
    seed_unadopted(&clean);
    commit_all(&clean);
    git(&clean, &["worktree", "add", "-q", clean.at("wt").to_str().unwrap(), "-b", "side"]);
    let err = adopt(&clean.dir, &[Tool::Claude]).expect_err("另有 worktree 必須拒絕").to_string();
    assert!(err.contains("other worktrees"), "{err}");
    assert!(clean.exists("openspec") && !clean.exists("speclink") && !clean.exists(".speclink.yaml"));
}

/// adopt 依實際的規格資料夾補骨架：只有 `speclink/` 的專案不得另建 `openspec/`。
#[test]
fn adopt_fills_the_folder_the_project_actually_uses() {
    let root = TempRoot::new("adopt-speclink-only");
    root.write("speclink/changes/add-auth/proposal.md", "## Why\n既有變更。\n");

    adopt(&root.dir, &[Tool::Claude]).expect("adopt succeeds");

    assert!(root.at("speclink/specs").is_dir());
    assert!(root.at("speclink/config.yaml").is_file());
    assert!(!root.at("openspec").exists(), "no second spec folder");
}

/// Spec Scenario「工作資料夾納入版控忽略」：.gitignore 缺席時建立並涵蓋 `.speclink/`。
#[test]
fn adopt_creates_gitignore_covering_the_work_dir() {
    let root = TempRoot::new("adopt-gitignore-new");
    seed_unadopted(&root);

    adopt(&root.dir, &[Tool::Claude]).expect("adopt succeeds");

    assert!(root.read(".gitignore").contains(".speclink/"));
}

/// Spec Example「既有 .gitignore 追加而非覆寫」逐值：原有兩行保留，多出 `.speclink/`。
#[test]
fn adopt_appends_to_an_existing_gitignore_without_overwriting() {
    let root = TempRoot::new("adopt-gitignore-append");
    seed_unadopted(&root);
    root.write(".gitignore", "node_modules/\ndist/\n");

    adopt(&root.dir, &[Tool::Claude]).expect("adopt succeeds");

    let text = root.read(".gitignore");
    for line in ["node_modules/", "dist/", ".speclink/"] {
        assert!(text.contains(line), "{line} 須存在於 .gitignore：\n{text}");
    }
}

/// 已涵蓋時重跑不重複追加（檔案位元級不變）。
#[test]
fn adopt_does_not_duplicate_an_existing_work_dir_entry() {
    let root = TempRoot::new("adopt-gitignore-idem");
    seed_unadopted(&root);
    adopt(&root.dir, &[Tool::Claude]).expect("first adopt");
    let first = root.read(".gitignore");

    adopt(&root.dir, &[Tool::Claude]).expect("second adopt");

    assert_eq!(root.read(".gitignore"), first, "重跑不得重複追加");
    assert_eq!(first.matches(".speclink/").count(), 1, "條目須恰有一筆：\n{first}");
}

// --- 工具檔生成不寫入 AI 工具的使用者設定檔 ---
// Spec requirement:「工具檔生成不寫入 AI 工具的使用者設定檔」
// （remove-claude-settings-write）——settings.json 屬使用者資料，
// 任何生成路徑不得建立或改寫。

const USER_SETTINGS: &str =
    "{\"enabledPlugins\":{\"frontend-design\":true},\"includeGitInstructions\":false}";

/// Spec Scenario「init 不產生使用者設定檔」。
#[test]
fn init_does_not_create_the_user_settings_file() {
    let root = TempRoot::new("no-settings-init");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    assert!(root.exists(".claude/skills/speclink-propose/SKILL.md"), "技能檔照常生成");
    assert!(!root.exists(".claude/settings.json"), "不得產生使用者設定檔");
}

/// Spec Scenario「既有使用者設定檔在工具同步後位元級不變」
/// ＋ Example「自訂外掛設定不被清空」逐值。
#[test]
fn update_leaves_an_existing_user_settings_file_untouched() {
    let root = TempRoot::new("no-settings-update");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write(".claude/settings.json", USER_SETTINGS);
    std::fs::remove_dir_all(root.at(".claude/skills/speclink-propose")).unwrap();

    update(&root.dir, false).expect("update succeeds");

    assert_eq!(root.read(".claude/settings.json"), USER_SETTINGS, "使用者設定檔位元級不變");
    assert!(root.exists(".claude/skills/speclink-propose/SKILL.md"), "受管技能檔照常再生");
}

/// Spec Scenario「工作區補齊不產生使用者設定檔」。
#[test]
fn adopt_does_not_create_the_user_settings_file() {
    let root = TempRoot::new("no-settings-adopt");
    seed_unadopted(&root);
    adopt(&root.dir, &[Tool::Claude]).expect("adopt succeeds");
    assert!(!root.exists(".claude/settings.json"), "不得產生使用者設定檔");
}

/// Spec Scenario「重複執行冪等」：相同 tools 連續執行兩次，全樹位元級相同。
#[test]
fn adopt_twice_with_same_tools_is_idempotent() {
    let root = TempRoot::new("adopt-idem");
    seed_unadopted(&root);
    adopt(&root.dir, &[Tool::Claude, Tool::Codex]).expect("first adopt");
    let first = snapshot(&root);

    adopt(&root.dir, &[Tool::Claude, Tool::Codex]).expect("second adopt");

    assert_eq!(snapshot(&root), first, "重複執行必須收斂於相同結果");
}

/// Spec Scenario「tools 空清單拒絕」：回單行錯誤且目錄零寫入。
#[test]
fn adopt_rejects_empty_tools_with_zero_writes() {
    let root = TempRoot::new("adopt-empty");
    seed_unadopted(&root);
    let before = snapshot(&root);

    let err = adopt(&root.dir, &[]).expect_err("空 tools 必須失敗");

    let message = err.to_string();
    assert!(message.contains("claude") && message.contains("codex"), "{message}");
    assert_eq!(message.lines().count(), 1, "錯誤須為單行：{message}");
    assert_eq!(snapshot(&root), before, "失敗不得留下任何寫入");
}

/// 同一選集下，既有 Workspace 收斂的受管產物與 filesystem init 的產物相同。
#[test]
fn reconcile_matches_init_output_for_the_same_selection() {
    let both = [Tool::Claude, Tool::Codex];
    let fresh = TempRoot::new("reconcile-parity-init");
    init(&fresh.dir, &both, false, "openspec").unwrap();

    let converged = TempRoot::new("reconcile-parity-converged");
    init(&converged.dir, &[Tool::Claude], false, "openspec").unwrap();
    reconcile_builtin_tools(&converged.dir, &both).expect("reconcile succeeds");

    for tool in both {
        let skill = propose_skill(tool);
        assert_eq!(converged.read(&skill), fresh.read(&skill), "{skill} 須與 init 相同");
    }
    assert!(converged.exists("openspec/specs"), "既有 filesystem 規格樹須保留");
}

// --- 遺留 marker 剝除（design D2；規格「built-in tools 權威收斂」
// 「描述子的同步與清理生命週期」） ---

/// 舊版引擎注入過的指令檔：marker 區塊在上，使用者段落在下。
fn legacy_marker_file(user_text: &str) -> String {
    format!(
        "<!-- SPECLINK:START v1.0.0 -->\n\n# Speclink Instructions\n\n舊版注入的路由表。\n\n<!-- SPECLINK:END -->\n{user_text}"
    )
}

#[test]
fn update_strips_a_legacy_marker_and_keeps_user_content() {
    // Scenario「更新時剝除內建工具的遺留 marker」：區塊消失、使用者段落原樣
    // 保留，摘要列出被剝除的檔案，技能檔照常再生。
    let root = TempRoot::new("strip-keeps-user");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write("CLAUDE.md", &legacy_marker_file(CLAUDE_USER_TEXT));

    let out = update(&root.dir, false).expect("update succeeds");

    let text = root.read("CLAUDE.md");
    assert!(!text.contains("<!-- SPECLINK:START"), "區塊須被剝除:\n{text}");
    assert_eq!(text, CLAUDE_USER_TEXT, "使用者段落須原樣保留:\n{text}");
    assert_eq!(out.stripped, vec!["CLAUDE.md".to_string()], "摘要須列出剝除的檔案");
    assert!(root.exists(&propose_skill(Tool::Claude)), "技能檔照常再生");
}

#[test]
fn update_deletes_an_instruction_file_that_was_only_a_marker() {
    // 剝除後全空的檔案整份刪除——不留一個空殼在專案根。
    let root = TempRoot::new("strip-deletes-empty");
    init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();
    root.write("CLAUDE.md", &legacy_marker_file(""));
    root.write("AGENTS.md", &legacy_marker_file(""));

    let out = update(&root.dir, false).expect("update succeeds");

    assert!(!root.exists("CLAUDE.md"), "純 marker 檔須刪除");
    assert!(!root.exists("AGENTS.md"), "純 marker 檔須刪除");
    assert_eq!(out.stripped, vec!["CLAUDE.md".to_string(), "AGENTS.md".to_string()]);
}

#[test]
fn update_leaves_an_instruction_file_without_a_marker_byte_identical() {
    // 無區塊＝零觸碰：使用者自己寫的 CLAUDE.md 不得被 update 改動一個位元組。
    let root = TempRoot::new("strip-untouched");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    let user_only = "# 我自己的 CLAUDE.md\n\n沒有任何受管區塊。\n";
    root.write("CLAUDE.md", user_only);

    let out = update(&root.dir, false).expect("update succeeds");

    assert_eq!(root.read("CLAUDE.md"), user_only, "無區塊的檔案須位元級不變");
    assert!(out.stripped.is_empty(), "沒剝除任何東西時摘要須為空：{:?}", out.stripped);
}

#[test]
fn update_strips_a_descriptors_legacy_marker() {
    // Scenario「更新時剝除描述子的遺留 marker」：仍帶 instructions_file 欄位的
    // 描述子，其指令檔同受剝除語意。
    let root = TempRoot::new("strip-descriptor");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write(".speclink.yaml", &format!("tools:\n{CUSTOM_DESCRIPTOR}"));
    root.write("WAD.md", &legacy_marker_file("使用者寫在 WAD.md 的段落\n"));

    let out = update(&root.dir, false).expect("update succeeds");

    let text = root.read("WAD.md");
    assert!(!text.contains("<!-- SPECLINK:START"), "描述子的區塊須被剝除:\n{text}");
    assert_eq!(text, "使用者寫在 WAD.md 的段落\n", "使用者段落須原樣保留:\n{text}");
    assert_eq!(out.stripped, vec!["WAD.md".to_string()]);
    assert!(root.exists(".wad/skills/speclink-apply/SKILL.md"), "描述子技能檔照常生成");
}

#[test]
fn init_force_over_a_legacy_workspace_strips_the_marker() {
    // design D2：對既有 marker 的專案 re-init（--force）同樣走剝除。
    let root = TempRoot::new("strip-init-force");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write("CLAUDE.md", &legacy_marker_file(CLAUDE_USER_TEXT));

    init(&root.dir, &[Tool::Claude], true, "openspec").expect("re-init succeeds");

    assert_eq!(root.read("CLAUDE.md"), CLAUDE_USER_TEXT, "區塊須被剝除、使用者段落保留");
}

/// 一個目標 skills 目錄下的 `speclink-*` 目錄名集合（排序後）。
fn skill_dirs(root: &TempRoot, rel: &str) -> Vec<String> {
    let mut names: Vec<String> = match std::fs::read_dir(root.at(rel)) {
        Ok(entries) => entries
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.starts_with("speclink-"))
            .collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    names
}

/// 一個內建工具在 worktree 政策關閉（`init` 樣板的預設）下應有的 `speclink-*`
/// 目錄名集合（排序後）——與生成端同源，取自 `managed_skills`，不另抄一份過濾規則。
fn expected_dirs(tool: Tool) -> Vec<String> {
    let mut names: Vec<String> =
        managed_skills(skills::RenderTarget::Builtin(tool), false, "openspec", SpecLocation::InCheckout)
            .into_iter()
            .map(|(dir, _)| dir)
            .collect();
    names.sort();
    names
}

#[test]
fn init_force_switching_tools_prunes_the_deselected_footprint() {
    // Scenario「init --force 切換工具時清除下架足跡」：--force 的選集是內建工具的
    // 完整期望狀態，未選工具不得留下可載入的技能檔。
    let root = TempRoot::new("init-force-switch");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    assert!(!skill_dirs(&root, ".claude/skills").is_empty(), "前置：Claude 技能已生成");

    init(&root.dir, &[Tool::Codex], true, "openspec").expect("re-init 成功");

    assert!(
        skill_dirs(&root, ".claude/skills").is_empty(),
        "未選工具的技能目錄須全數移除"
    );
    assert!(!root.exists(".claude"), "空掉的 .claude 目錄一併移除");
    assert_eq!(
        skill_dirs(&root, ".agents/skills"),
        expected_dirs(Tool::Codex),
        "選中工具補齊為本次生成集合"
    );
    let app = root.read(".speclink.yaml");
    assert!(app.contains("tools: [codex]"), "{app}");
}

#[test]
fn init_force_prunes_a_renamed_skill_directory() {
    // Scenario「init --force 清除改名技能的舊目錄與描述子足跡」的前半：孤兒清理
    // 不再是 update 專屬。
    let root = TempRoot::new("init-force-orphan");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write(".claude/skills/speclink-onboard/SKILL.md", "舊版生成物\n");

    init(&root.dir, &[Tool::Claude], true, "openspec").expect("re-init 成功");

    assert!(
        !root.exists(".claude/skills/speclink-onboard"),
        "改名前的舊目錄須清除"
    );
    assert!(root.exists(".claude/skills/speclink-baseline"), "現行技能仍在");
}

#[test]
fn init_force_resets_custom_footprint_state() {
    // Scenario「init --force 清除改名技能的舊目錄與描述子足跡」的後半：--force 把
    // .speclink.yaml 寫回樣板（描述子一併清掉），足跡與生成物必須同步歸零。
    let root = TempRoot::new("init-force-custom-reset");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write(".speclink.yaml", &format!("tools:\n  - claude\n{CUSTOM_DESCRIPTOR}"));
    update(&root.dir, false).expect("先生成描述子受管檔與足跡");
    assert!(root.exists(".wad/skills/speclink-propose/SKILL.md"), "前置：描述子生成物在");
    assert!(root.exists(".speclink/generated-tools.yaml"), "前置：足跡已記錄");

    init(&root.dir, &[Tool::Claude], true, "openspec").expect("re-init 成功");

    assert!(
        skill_dirs(&root, ".wad/skills").is_empty(),
        "描述子的生成物須隨設定重寫一併清除"
    );
    assert!(
        !root.exists(".speclink/generated-tools.yaml"),
        "足跡狀態檔須歸零"
    );
    let app = root.read(".speclink.yaml");
    assert!(!app.contains("wad-harness"), "--force 重寫為樣板：{app}");
    assert!(app.contains("tools: [claude]"), "選集重寫為本次的 claude：{app}");
}

#[test]
fn init_strips_the_unselected_tools_legacy_marker() {
    // Scenario「init --force 剝除未選工具的遺留區塊」：剝除與選取與否無關——
    // 舊版注入的路由表留在 CLAUDE.md 會與技能路由並存。
    let root = TempRoot::new("init-force-strip-unselected");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write("CLAUDE.md", &legacy_marker_file(CLAUDE_USER_TEXT));

    init(&root.dir, &[Tool::Codex], true, "openspec").expect("re-init 成功");

    assert_eq!(root.read("CLAUDE.md"), CLAUDE_USER_TEXT, "區塊剝除、使用者段落保留");
    assert!(!root.exists("AGENTS.md"), "不生成指令檔");
}

#[test]
fn init_over_residual_skill_files_rewrites_them() {
    // Scenario「不帶 force 的 init 改寫殘留技能檔」：沒有 .speclink.yaml 也沒有
    // openspec/ 時「已初始化」守門不成立，殘留的舊技能檔靜默保留才是漏洞。
    let root = TempRoot::new("init-residual");
    root.write(".claude/skills/speclink-apply/SKILL.md", "舊版內容\n");
    root.write(".claude/skills/speclink-onboard/SKILL.md", "改名前的舊目錄\n");

    init(&root.dir, &[Tool::Claude], false, "openspec").expect("init 成功");

    let applied = root.read(".claude/skills/speclink-apply/SKILL.md");
    assert!(applied.contains(ASSET_VERSION), "殘留檔須改寫為現版：{applied}");
    assert!(
        !root.exists(".claude/skills/speclink-onboard"),
        "殘留孤兒目錄須清除"
    );
    assert_eq!(
        skill_dirs(&root, ".claude/skills"),
        expected_dirs(Tool::Claude),
        "其餘生成集合補齊"
    );
}

#[test]
fn init_on_a_fresh_project_writes_no_instruction_file() {
    // Scenario「指令檔零受管區塊」：全新目錄以兩個工具 init 後，專案根不存在
    // 任何指令檔，技能檔照常生成。
    let root = TempRoot::new("fresh-no-instructions");
    init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();

    assert!(!root.exists("CLAUDE.md"), "不得生成 CLAUDE.md");
    assert!(!root.exists("AGENTS.md"), "不得生成 AGENTS.md");
    for tool in [Tool::Claude, Tool::Codex] {
        assert!(root.exists(&propose_skill(tool)), "{} 技能檔照常生成", tool.name());
    }
}

#[test]
fn strip_leaves_a_file_with_an_unpaired_start_untouched() {
    // R1：END 行被手動刪掉或 merge conflict 弄壞時，剝除不得把 START 之後的
    // 內容吞掉——不成對就整檔不動（位元級不變），寧可留一塊死文字。
    let root = TempRoot::new("strip-unpaired");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    let broken = "<!-- SPECLINK:START v1.0.0 -->\n\n舊路由表。\n\n使用者的重要內容\n";
    root.write("CLAUDE.md", broken);

    let out = update(&root.dir, false).expect("update succeeds");

    assert_eq!(root.read("CLAUDE.md"), broken, "不成對的檔案必須位元級不變");
    assert!(out.stripped.is_empty(), "不成對不算剝除：{:?}", out.stripped);
}

#[test]
fn strip_removes_every_legacy_block_in_one_run() {
    // R2：多個遺留區塊（壞 merge 疊出來的）一次 update 全剝乾淨，不用跑第二次。
    let root = TempRoot::new("strip-multi");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write(
        "CLAUDE.md",
        "<!-- SPECLINK:START v1.0.0 -->\nA\n<!-- SPECLINK:END -->\n中間的使用者文字\n<!-- SPECLINK:START v1.1.0 -->\nB\n<!-- SPECLINK:END -->\n結尾文字\n",
    );

    let out = update(&root.dir, false).expect("update succeeds");

    let text = root.read("CLAUDE.md");
    assert!(!text.contains("SPECLINK:START"), "兩個區塊都要剝掉:\n{text}");
    assert!(text.contains("中間的使用者文字") && text.contains("結尾文字"), "{text}");
    assert_eq!(out.stripped, vec!["CLAUDE.md".to_string()]);
}

#[test]
fn strip_handles_crlf_files_without_leaving_blank_lines() {
    // R3：Windows checkout（CRLF）剝除後不得留下前導空行。
    let root = TempRoot::new("strip-crlf");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write(
        "CLAUDE.md",
        "<!-- SPECLINK:START v1.0.0 -->\r\n\r\n舊路由表。\r\n\r\n<!-- SPECLINK:END -->\r\n使用者段落\r\n",
    );

    update(&root.dir, false).expect("update succeeds");

    assert_eq!(root.read("CLAUDE.md"), "使用者段落\r\n", "CRLF 分隔空行須一併消失");
}

// --- 技能檔過期探測（規格「技能檔過期探測」；design D6） ---

/// 比現版領先一個主版號的版號：工作區檔案由更新的引擎生成的情境。
fn ahead_of_current() -> String {
    let major: u64 = ASSET_VERSION
        .trim_start_matches('v')
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .expect("ASSET_VERSION 主版號可解析");
    format!("v{}.0.0", major + 1)
}

/// 把某工具 skills 目錄下每份技能檔的 frontmatter 版號改成指定值
///（模擬以別版引擎生成的工作區——舊值模擬落後、新值模擬領先）。
fn set_skill_version(root: &TempRoot, tool: Tool, version: &str) {
    crate::testkit::set_skill_version(&root.at(tool.skills_dir()), version);
}

#[test]
fn skill_probe_reports_stale_and_lists_differing_files() {
    // Scenario「舊版工作區判過期並列差異檔」：技能版號舊於現版即過期。
    let root = TempRoot::new("skill-probe-stale");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    set_skill_version(&root, Tool::Claude, "v0.9.0");

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Stale, "{probe:?}");
    assert_eq!(probe.current_version, ASSET_VERSION);
    assert_eq!(probe.tools.len(), 1);
    assert_eq!(probe.tools[0].tool, "claude");
    assert_eq!(probe.tools[0].workspace_version.as_deref(), Some("v0.9.0"));
    assert!(probe.tools[0].stale && !probe.tools[0].missing && !probe.tools[0].newer);
    assert!(
        probe.differing_files.contains(&propose_skill(Tool::Claude)),
        "改動過的技能檔須列入差異清單：{:?}",
        probe.differing_files
    );
}

#[test]
fn skill_probe_reports_newer_when_the_workspace_leads_the_engine() {
    // Example「引擎 v1.11.0 探測 v1.14.0 工作區」的字面版號由
    // `workspace_version_direction_only_orders_parsable_versions` 釘住方向判定；
    // 這裡用 ahead_of_current() 讓斷言不隨 ASSET_VERSION 遞增而過期。
    // Scenario「工作區檔案領先引擎判較新」。
    let root = TempRoot::new("skill-probe-newer");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    let ahead = ahead_of_current();
    set_skill_version(&root, Tool::Claude, &ahead);

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Newer, "{probe:?}");
    assert_eq!(probe.tools[0].workspace_version.as_deref(), Some(ahead.as_str()));
    assert!(probe.tools[0].newer && !probe.tools[0].stale && !probe.tools[0].missing);
    assert!(!probe.differing_files.is_empty(), "較新時仍須回報差異檔清單");
}

#[test]
fn skill_probe_reports_missing_when_the_skills_dir_has_no_speclink_skill() {
    // Scenario「技能目錄缺少判缺失」：整組技能不在（clone 後技能未進版控），
    // 且缺失勝過另一支的過期。
    let root = TempRoot::new("skill-probe-missing");
    init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();
    std::fs::remove_dir_all(root.at(Tool::Codex.skills_dir())).unwrap();
    set_skill_version(&root, Tool::Claude, "v0.9.0");

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Missing, "{probe:?}");
    let codex = probe.tools.iter().find(|t| t.tool == "codex").expect("codex 在列");
    assert!(codex.missing && !codex.stale, "{codex:?}");
    assert_eq!(codex.workspace_version, None);
    assert!(
        probe.differing_files.contains(&propose_skill(Tool::Codex)),
        "不存在的受管檔須列入（內容視為空）：{:?}",
        probe.differing_files
    );
}

#[test]
fn skill_probe_prefers_newer_over_missing_and_stale() {
    // Scenario「較新優先於缺失與過期」。
    let root = TempRoot::new("skill-probe-newer-wins");
    init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();
    set_skill_version(&root, Tool::Claude, &ahead_of_current());
    std::fs::remove_dir_all(root.at(Tool::Codex.skills_dir())).unwrap();

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Newer, "{probe:?}");
    let codex = probe.tools.iter().find(|t| t.tool == "codex").expect("codex 在列");
    assert!(codex.missing && !codex.newer, "缺失的工具不得被標成較新：{codex:?}");
}

#[test]
fn skill_probe_falls_back_to_equality_for_an_unparsable_version() {
    // Scenario「無法解析的版號退回相等判定」。
    let root = TempRoot::new("skill-probe-unparsable");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    set_skill_version(&root, Tool::Claude, "v-not-a-version");

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Stale, "{probe:?}");
    assert!(probe.tools[0].stale && !probe.tools[0].newer, "{:?}", probe.tools[0]);
}

#[test]
fn dropping_the_deprecated_instructions_file_field_does_not_prune_the_descriptor() {
    // R6：使用者照棄用提示把 instructions_file 從描述子移除——同一工具不得被
    // 誤判「已下架」而整組刪掉重建（pruned 與 updated 同列一個工具的自相矛盾）。
    let root = TempRoot::new("descriptor-drop-deprecated-field");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write(".speclink.yaml", &format!("tools:\n{CUSTOM_DESCRIPTOR}"));
    update(&root.dir, false).expect("先以帶欄位的描述子生成足跡");

    // 移除 instructions_file 欄位（skills_dir 不變）。
    root.write(
        ".speclink.yaml",
        "tools:\n  - name: wad-harness\n    skills_dir: .wad/skills\n",
    );
    // 在技能目錄放一個使用者自己的檔案：整組 prune 重建會讓它消失。
    root.write(".wad/skills/my-note.md", "使用者自己的檔案\n");

    let out = update(&root.dir, false).expect("update succeeds");

    assert!(
        !out.pruned.contains(&"wad-harness".to_string()),
        "移除棄用欄位不得觸發 prune：{:?}",
        out.pruned
    );
    assert!(out.updated.contains(&"wad-harness".to_string()));
    assert_eq!(root.read(".wad/skills/my-note.md"), "使用者自己的檔案\n");
}

#[test]
fn skill_version_parsing_stays_inside_the_frontmatter() {
    // skill_version_of 只認 frontmatter：body 裡恰好叫 version: 的內文行不算，
    // body 的 ---- 分隔線也不會提前截斷 frontmatter 搜尋。
    assert_eq!(
        skill_version_of("---\nname: x\nmetadata:\n  version: \"v9.9.9\"\n---\n\nbody version: \"v0.0.1\"\n"),
        Some("v9.9.9")
    );
    assert_eq!(
        skill_version_of("---\nname: x\n---\n\n----\n\nversion: \"v0.0.1\"\n"),
        None,
        "frontmatter 沒有版本行時，body 的 version 行不得被撿走"
    );
    assert_eq!(skill_version_of("no frontmatter\nversion: \"v1\"\n"), None);
}

#[test]
fn skill_probe_reports_unknown_when_the_version_line_is_gone() {
    // R4：SKILL.md 在但 frontmatter 版本行遺失（手改壞）＝「技能檔存在但讀取
    // 錯誤」→ 無法判定，絕不可回報現版——那會讓壞檔永遠不被提示修復。
    let root = TempRoot::new("skill-probe-no-version");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    for entry in std::fs::read_dir(root.at(Tool::Claude.skills_dir())).unwrap().flatten() {
        let file = entry.path().join("SKILL.md");
        if file.is_file() {
            let text: String = std::fs::read_to_string(&file)
                .unwrap()
                .lines()
                .filter(|l| !l.trim_start().starts_with("version:"))
                .collect::<Vec<_>>()
                .join("\n");
            std::fs::write(&file, text).unwrap();
        }
    }

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Unknown, "{probe:?}");
}

#[test]
fn skill_probe_reports_unknown_when_the_skills_dir_is_unreadable() {
    // R5：read_dir 失敗（路徑是檔案、權限）不得與「從未安裝」混同——回無法
    // 判定，否則 desktop 會給一個按下去必然失敗的「安裝」動作。
    let root = TempRoot::new("skill-probe-dir-is-file");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    std::fs::remove_dir_all(root.at(".claude/skills")).unwrap();
    root.write(".claude/skills", "這是一個檔案不是目錄");

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Unknown, "{probe:?}");
}

#[test]
fn skill_update_refuses_a_workspace_whose_skills_lead_the_engine() {
    // 降級守門的版本來源同步改基準：領先的技能檔不得被任何再生路徑改寫。
    let root = TempRoot::new("skill-guard-refuse");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    let ahead = ahead_of_current();
    set_skill_version(&root, Tool::Claude, &ahead);
    let before = snapshot(&root);

    let err = update(&root.dir, false).expect_err("領先的工作區必須被拒");
    let msg = err.to_string();
    assert!(msg.contains(&ahead) && msg.contains(ASSET_VERSION), "訊息須含兩版號：{msg}");
    assert_eq!(msg.lines().count(), 1, "單行錯誤：{msg}");
    assert_eq!(snapshot(&root), before, "拒絕＝零寫入");

    update(&root.dir, true).expect("明示越過後照常再生");
    assert!(
        root.read(&propose_skill(Tool::Claude)).contains(ASSET_VERSION),
        "受管檔須再生為引擎現版"
    );
}

#[test]
fn workspace_version_direction_only_orders_parsable_versions() {
    // 規格「技能檔過期探測」的數值比較規則：去 v 前綴、以點拆段、逐段數值比較、
    // 段數不足補零；任一邊無法完整解析為數字段時不排序方向——寧可誤報過期，
    // 不可誤報較新（會封鎖 update）。
    // spec Example「引擎 v1.11.0 探測 v1.14.0 工作區」的字面值：
    assert!(workspace_is_newer("v1.14.0", "v1.11.0"), "工作區領先引擎");
    assert!(!workspace_is_newer("v1.11.0", "v1.14.0"), "工作區落後引擎");
    assert!(!workspace_is_newer("v1.14.0", "v1.14.0"), "同版不算領先");
    // 逐段數值（非字典序）：v1.9.0 < v1.10.0
    assert!(workspace_is_newer("v1.10.0", "v1.9.0"), "以數值而非字典序比較");
    // 段數不足補零
    assert!(workspace_is_newer("v1.14.1", "v1.14"), "缺段視為 0");
    assert!(!workspace_is_newer("v1.14", "v1.14.0"), "補零後相等不算領先");
    // 無法解析：兩個方向都不判較新
    assert!(!workspace_is_newer("bogus", "v1.14.0"), "無法解析不得判較新");
    assert!(!workspace_is_newer("v1.14.0-beta", "v1.14.0"), "非純數字段不得判較新");
    assert!(!workspace_is_newer("v1.14.0", "bogus"), "引擎端無法解析亦不判較新");
}

#[test]
fn init_force_refuses_a_workspace_that_leads_the_engine() {
    // `--force` 的語意是「覆蓋既有檔案」，不是「同意降級」——重新初始化同樣
    // 不得把領先的技能檔改寫回舊內容。
    let root = TempRoot::new("init-force-guard");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    let ahead = ahead_of_current();
    set_skill_version(&root, Tool::Claude, &ahead);
    let before = snapshot(&root);

    let Err(err) = init(&root.dir, &[Tool::Claude], true, "openspec") else {
        panic!("領先的工作區不得被 --force 重建改寫");
    };
    let msg = err.to_string();
    assert!(msg.contains(&ahead) && msg.contains(ASSET_VERSION), "訊息須含兩版號：{msg}");
    assert_eq!(snapshot(&root), before, "拒絕＝零寫入");
}

#[test]
fn the_guard_covers_the_legacy_fallback_without_a_tools_list() {
    // 守門與寫入集同源：無 tools: 鍵的工作區走 .claude/ 目錄偵測再生，
    // 探測卻因選集為空判現版——這類 legacy 工作區同樣必須拒絕降級。
    let root = TempRoot::new("guard-legacy");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    // 模擬 legacy 工作區：config 沒有 tools 清單，但 .claude/ 目錄在。
    root.write(".speclink.yaml", "# Speclink application config\n");
    let ahead = ahead_of_current();
    set_skill_version(&root, Tool::Claude, &ahead);
    let before = snapshot(&root);

    let err = update(&root.dir, false).expect_err("legacy fallback 同樣必須被拒");
    let msg = err.to_string();
    assert!(msg.contains(&ahead) && msg.contains(ASSET_VERSION), "{msg}");
    assert_eq!(snapshot(&root), before, "拒絕＝零寫入");

    update(&root.dir, true).expect("明示越過照常再生");
    assert!(root.read(&propose_skill(Tool::Claude)).contains(ASSET_VERSION));
}

#[test]
fn the_guard_covers_custom_descriptor_skill_files() {
    // Scenario「自訂描述子的技能檔同受守門」：只用描述子的工作區不得因判定面
    // 只收 builtin 而被降級。
    let root = TempRoot::new("guard-descriptor");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write(".speclink.yaml", &format!("tools:\n{CUSTOM_DESCRIPTOR}"));
    update(&root.dir, false).expect("先生成描述子受管檔");
    let ahead = ahead_of_current();
    let skill = root.at(".wad/skills/speclink-propose/SKILL.md");
    let text = std::fs::read_to_string(&skill).unwrap().replace(ASSET_VERSION, &ahead);
    std::fs::write(&skill, text).unwrap();
    let before = snapshot(&root);

    let err = update(&root.dir, false).expect_err("領先的描述子技能檔必須被拒");
    assert!(err.to_string().contains(&ahead), "{err}");
    assert_eq!(snapshot(&root), before, "拒絕＝零寫入");
}

#[test]
fn reconcile_refuses_a_leading_workspace_before_touching_the_config() {
    // 方向檢查在 .speclink.yaml 寫入之前：拒絕＝整體零寫入，不留
    // 「config 已改、受管檔未同步」的半狀態。
    let root = TempRoot::new("reconcile-guard");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    set_skill_version(&root, Tool::Claude, &ahead_of_current());
    let before = snapshot(&root);

    let err = reconcile_builtin_tools(&root.dir, &[Tool::Claude, Tool::Codex])
        .expect_err("領先的工作區必須在寫入 config 前被拒");
    assert!(err.to_string().contains(ASSET_VERSION), "{err}");
    assert_eq!(snapshot(&root), before, "拒絕＝零寫入（含 .speclink.yaml）");
}

#[test]
fn probe_reports_current_for_a_freshly_generated_workspace() {
    // Scenario「現版工作區不過期」：差異清單為空。
    let root = TempRoot::new("probe-current");
    init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Current, "{probe:?}");
    assert!(probe.differing_files.is_empty(), "{:?}", probe.differing_files);
    assert!(probe.tools.iter().all(|t| !t.stale && !t.missing));
}

#[test]
fn probe_reports_unknown_for_a_malformed_config() {
    // Scenario「設定損壞回報無法判定」：不得與現版或過期混同。
    let root = TempRoot::new("probe-badconfig");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write(".speclink.yaml", "tools: [\n");

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Unknown, "{probe:?}");
    assert!(probe.tools.is_empty(), "{:?}", probe.tools);
    assert!(probe.differing_files.is_empty(), "{:?}", probe.differing_files);
}

#[test]
fn probe_ignores_line_ending_differences() {
    // Scenario「換行差異不誤報」：CRLF 工作區（Windows core.autocrlf）僅換行
    // 形式不同的檔案不得列入差異清單。
    let root = TempRoot::new("probe-crlf");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    // 只讓一份技能檔落後（探測依此判過期），另一份維持現版內容但改成 CRLF。
    let stale = ".claude/skills/speclink-analyze/SKILL.md";
    root.write(stale, &root.read(stale).replace(ASSET_VERSION, "v0.9.0"));
    let skill = propose_skill(Tool::Claude);
    let crlf = root.read(&skill).replace('\n', "\r\n");
    root.write(&skill, &crlf);

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Stale, "{probe:?}");
    assert!(
        !probe.differing_files.contains(&skill),
        "僅換行形式不同的檔案不得列入：{:?}",
        probe.differing_files
    );
}

/// tools 清單＝claude 加一個描述子，並讓兩邊的受管檔都生成到現版。
fn workspace_with_descriptor(tag: &str) -> TempRoot {
    let root = TempRoot::new(tag);
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write(".speclink.yaml", &format!("tools:\n  - claude\n{MY_HARNESS_DESCRIPTOR}"));
    update(&root.dir, false).expect("先把兩邊的受管檔生成到現版");
    root
}

#[test]
fn probe_reports_missing_for_a_descriptor_without_skills() {
    // Scenario「描述子技能檔缺失判缺失」：描述子的技能檔與內建技能檔同為受管檔，
    // 整組不在即缺失，差異清單以描述子的 skills_dir 起頭。
    let root = workspace_with_descriptor("probe-descriptor-missing");
    std::fs::remove_dir_all(root.at(".my-harness/skills")).unwrap();

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Missing, "{probe:?}");
    let harness = probe
        .tools
        .iter()
        .find(|t| t.tool == "my-harness")
        .expect("逐工具資訊須含描述子的 name");
    assert!(harness.missing, "{harness:?}");
    assert_eq!(harness.workspace_version, None, "{harness:?}");
    let claude = probe.tools.iter().find(|t| t.tool == "claude").unwrap();
    assert!(!claude.missing && !claude.stale && !claude.newer, "{claude:?}");
    assert!(
        !probe.differing_files.is_empty(),
        "缺失須列出描述子的受管檔"
    );
    for path in &probe.differing_files {
        assert!(
            path.starts_with(".my-harness/skills/speclink-") && path.ends_with("/SKILL.md"),
            "差異清單只該含描述子技能檔：{path}"
        );
    }
    // 非 Claude 子集＋worktree 政策關閉：兩顆 worktree 技能不在預期生成集合內。
    for gated in ["speclink-apply-with-worktree", "speclink-worktree-merge"] {
        assert!(
            !probe.differing_files.iter().any(|p| p.contains(gated)),
            "被政策排除的技能不得列入：{:?}",
            probe.differing_files
        );
    }
}

#[test]
fn probe_lists_descriptor_paths_in_differing_files() {
    // Scenario「描述子技能檔過期判過期」：描述子側落後即整體過期，且兩側同時落後
    // 時差異清單同時涵蓋內建與描述子——判定面不再只有 builtin。
    let root = workspace_with_descriptor("probe-descriptor-both");
    for skill in [
        ".claude/skills/speclink-propose/SKILL.md",
        ".my-harness/skills/speclink-propose/SKILL.md",
    ] {
        root.write(skill, &root.read(skill).replace(ASSET_VERSION, "v0.9.0"));
    }

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Stale, "{probe:?}");
    let harness = probe.tools.iter().find(|t| t.tool == "my-harness").unwrap();
    assert!(harness.stale && !harness.newer && !harness.missing, "{harness:?}");
    for expected in [
        ".claude/skills/speclink-propose/SKILL.md",
        ".my-harness/skills/speclink-propose/SKILL.md",
    ] {
        assert!(
            probe.differing_files.iter().any(|p| p == expected),
            "{expected} 須列入：{:?}",
            probe.differing_files
        );
    }
}

#[test]
fn tool_selection_keeps_valid_descriptors_after_a_bad_one() {
    // 第一個壞描述子只決定 descriptor_error；它後面的合法描述子仍要進 customs，
    // 否則探測會看不見那個工具（update 反正在寫入前就以錯誤停下）。
    let root = TempRoot::new("tool-selection-bad-then-good");
    let sel = ToolSelection::resolve(
        &root.dir,
        &app_config(&format!("tools:\n  - codex\n  - name: broken-harness\n{CUSTOM_DESCRIPTOR}")),
    );
    assert_eq!(
        sel.descriptor_error.as_deref(),
        Some("tool descriptor: missing required field 'skills_dir'"),
        "第一個問題被保留"
    );
    assert_eq!(sel.customs.len(), 1, "壞描述子後面的合法描述子仍被收下：{:?}", sel.customs);
    assert_eq!(sel.customs[0].name, "wad-harness");
}

#[test]
fn probe_still_covers_a_valid_descriptor_after_an_invalid_one() {
    // 探測面＝計畫的 targets：合法描述子不得因為前面排了一個壞描述子而消失。
    let root = workspace_with_descriptor("probe-descriptor-after-invalid");
    root.write(
        ".speclink.yaml",
        &format!("tools:\n  - claude\n  - name: broken-harness\n{MY_HARNESS_DESCRIPTOR}"),
    );
    std::fs::remove_dir_all(root.at(".my-harness/skills")).unwrap();

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Missing, "{probe:?}");
    let harness = probe.tools.iter().find(|t| t.tool == "my-harness").expect("my-harness 仍在判定面");
    assert!(harness.missing, "{harness:?}");
}

#[test]
fn probe_normalizes_a_descriptor_skills_dir_with_a_trailing_slash() {
    // 描述子的 skills_dir 是使用者手寫的字串：結尾斜線不得在差異清單裡變成 `//`。
    let root = TempRoot::new("probe-descriptor-trailing-slash");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write(
        ".speclink.yaml",
        "tools:\n  - claude\n  - name: my-harness\n    skills_dir: .my-harness/skills/\n",
    );
    update(&root.dir, false).expect("結尾斜線的 skills_dir 合法，先生成到現版");
    std::fs::remove_dir_all(root.at(".my-harness/skills")).unwrap();

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Missing, "{probe:?}");
    assert!(!probe.differing_files.is_empty());
    for path in &probe.differing_files {
        assert!(
            path.starts_with(".my-harness/skills/speclink-") && !path.contains("//"),
            "路徑須正規化：{path}"
        );
    }
}

#[test]
fn probe_ignores_an_invalid_descriptor() {
    // Scenario「無效描述子不影響探測」：壞描述子不成為 target，探測是唯讀提示面，
    // 「設定裡有個壞描述子」由 update 的錯誤告知，不得讓探測轉為無法判定。
    let root = TempRoot::new("probe-descriptor-invalid");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write(".speclink.yaml", "tools:\n  - claude\n  - name: my-harness\n");

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Current, "{probe:?}");
    assert_eq!(probe.tools.len(), 1, "{:?}", probe.tools);
    assert_eq!(probe.tools[0].tool, "claude");
    assert!(probe.differing_files.is_empty(), "{:?}", probe.differing_files);
}

#[test]
fn probe_with_an_empty_tools_list_reports_current_and_writes_nothing() {
    // tools 空清單＝沒有受管工具可查：回報現版、零差異；且探測全程零寫入。
    let root = TempRoot::new("probe-empty-tools");
    init(&root.dir, &[], false, "openspec").unwrap();
    let before = snapshot(&root);

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Current, "{probe:?}");
    assert!(probe.tools.is_empty());
    assert_eq!(snapshot(&root), before, "探測不得寫入任何檔案");
}

// --- 探測回報遷移狀態（desktop-spec-dir-migration D1；需求「技能檔提示承載規格資料夾遷移」）---

fn git(root: &TempRoot, args: &[&str]) {
    let ok = std::process::Command::new("git")
        .arg("-C")
        .arg(&root.dir)
        .args(["-c", "user.name=T", "-c", "user.email=t@example.com"])
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    assert!(ok, "git {args:?} failed");
}

fn commit_all(root: &TempRoot) {
    git(root, &["init", "-q"]);
    git(root, &["add", "-A"]);
    git(root, &["commit", "-qm", "init"]);
}

/// 舊佈局專案：`.speclink.yaml` 沒有 spec_dir、規格在 openspec/，兩個變更資料檔
///（一個作用中、一個已封存），技能檔為現版。
fn legacy_workspace(tag: &str) -> TempRoot {
    let root = TempRoot::new(tag);
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write(".speclink.yaml", "tools: [claude]\n");
    root.write("openspec/changes/demo/.openspec.yaml", "schema: spec-driven\n");
    root.write("openspec/changes/archive/2026-01-01-done/.openspec.yaml", "schema: spec-driven\n");
    root
}

fn pending(meta_files: usize) -> crate::migrate::MigrationProbe {
    crate::migrate::MigrationProbe::Pending { folder: crate::migrate::SpecFolder::Openspec, meta_files }
}

fn deferred(reason: crate::migrate::DeferCode, worktrees: Vec<String>) -> crate::migrate::MigrationProbe {
    crate::migrate::MigrationProbe::Deferred { folder: crate::migrate::SpecFolder::Openspec, reason, worktrees }
}

#[test]
fn probe_reports_a_pending_migration_even_when_the_skills_are_current() {
    // Scenario「技能檔不過期也提示遷移」：技能檔是現版，遷移狀態照樣回報。
    let root = legacy_workspace("probe-migration-pending");
    commit_all(&root);
    let before = project_snapshot(&root);

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Current, "{probe:?}");
    assert_eq!(probe.migration, pending(2), "{probe:?}");
    // Scenario「app 升版不在背景遷移」：探測只讀，openspec/ 原封不動。
    assert_eq!(project_snapshot(&root), before, "探測不得移動或寫入任何檔案");
    assert!(root.exists("openspec/changes/demo/.openspec.yaml"));
}

#[test]
fn probe_reports_the_migration_beside_stale_skills() {
    let root = legacy_workspace("probe-migration-stale");
    set_skill_version(&root, Tool::Claude, "v0.9.0");

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Stale, "{probe:?}");
    assert_eq!(probe.migration, pending(2), "{probe:?}");
}

#[test]
fn probe_defers_the_migration_while_other_worktrees_exist() {
    // Scenario「暫緩時只更新技能檔」：主目錄看到其他 worktree 的路徑；
    // linked worktree 本身回報 linkedWorktree。
    let root = legacy_workspace("probe-migration-worktrees");
    commit_all(&root);
    let wt = root.at("wt");
    git(&root, &["worktree", "add", "-q", wt.to_str().unwrap(), "-b", "side"]);
    let listed: Vec<String> =
        crate::testkit::listed_worktrees(&root.dir).iter().map(|p| p.display().to_string()).collect();
    assert_eq!(listed.len(), 1);

    assert_eq!(probe_assets(&root.dir).migration, deferred(crate::migrate::DeferCode::OtherWorktrees, listed));
    assert_eq!(probe_assets(&wt).migration, deferred(crate::migrate::DeferCode::LinkedWorktree, Vec::new()));
}

#[test]
fn probe_defers_the_migration_while_openspec_has_uncommitted_changes() {
    let root = legacy_workspace("probe-migration-dirty");
    commit_all(&root);
    root.write("openspec/changes/demo/tasks.md", "- [ ] 1.1 x\n");

    assert_eq!(probe_assets(&root.dir).migration, deferred(crate::migrate::DeferCode::Uncommitted, Vec::new()));
}

#[test]
fn probe_reports_no_migration_for_the_new_layout_an_explicit_spec_dir_or_remote_mode() {
    let fresh = TempRoot::new("probe-migration-new-layout");
    init(&fresh.dir, &[Tool::Claude], false, DEFAULT_SPEC_DIR).unwrap();
    assert_eq!(probe_assets(&fresh.dir).migration, crate::migrate::MigrationProbe::None, "new layout");

    // init 以非預設資料夾初始化會寫入 `spec_dir: openspec`。
    let explicit = TempRoot::new("probe-migration-explicit");
    init(&explicit.dir, &[Tool::Claude], false, "openspec").unwrap();
    explicit.write("openspec/changes/demo/.openspec.yaml", "schema: spec-driven\n");
    assert_eq!(probe_assets(&explicit.dir).migration, crate::migrate::MigrationProbe::None, "explicit spec_dir");

    let remote = TempRoot::new("probe-migration-remote");
    seed_remote_workspace(&remote, &[Tool::Claude]);
    remote.write("openspec/changes/demo/.openspec.yaml", "schema: spec-driven\n");
    assert_eq!(probe_assets(&remote.dir).migration, crate::migrate::MigrationProbe::None, "remote");
}

#[test]
fn sync_skills_regenerates_without_moving_the_spec_folder() {
    // spec「技能足跡同步不遷移規格資料夾」（desktop-spec-dir-migration D6）：worktree 政策
    // 切換的技能同步不遷移；降級守門照舊。
    let root = legacy_workspace("sync-skills-no-migration");
    commit_all(&root);
    std::fs::remove_dir_all(root.at(".claude/skills/speclink-propose")).unwrap();

    let outcome = sync_skills(&root.dir).expect("sync succeeds");
    assert_eq!(outcome.migration, crate::migrate::MigrationPlan::Nothing);
    assert!(root.exists("openspec/changes/demo/.openspec.yaml"), "openspec/ 原封不動");
    assert!(!root.exists("speclink"));
    assert!(root.exists(".claude/skills/speclink-propose/SKILL.md"), "技能檔照常再生");

    set_skill_version(&root, Tool::Claude, &ahead_of_current());
    assert!(sync_skills(&root.dir).is_err(), "工作區領先引擎時照樣拒絕");
}

#[test]
fn probe_serializes_the_migration_in_camel_case() {
    let root = legacy_workspace("probe-migration-json");
    let value = serde_json::to_value(probe_assets(&root.dir)).unwrap();
    assert_eq!(
        value["migration"],
        serde_json::json!({ "state": "pending", "folder": "openspec", "metaFiles": 2 })
    );

    let value = serde_json::to_value(deferred(crate::migrate::DeferCode::OtherWorktrees, vec!["/w".into()])).unwrap();
    assert_eq!(
        value,
        serde_json::json!({ "state": "deferred", "folder": "openspec", "reason": "otherWorktrees", "worktrees": ["/w"] })
    );
    assert_eq!(
        serde_json::to_value(crate::migrate::MigrationProbe::None).unwrap(),
        serde_json::json!({ "state": "none" })
    );
}

// --- worktree 政策閘：生成集合隨 openspec/config.yaml 的 worktree 檔值 ---
// Spec requirement「worktree 技能的政策條件式生成」。

/// 兩顆受閘控技能於某工具 skills 目錄下的相對路徑。
fn worktree_skill_dirs(tool: Tool) -> [String; 2] {
    [
        format!("{}/speclink-apply-with-worktree", tool.skills_dir()),
        format!("{}/speclink-worktree-merge", tool.skills_dir()),
    ]
}

/// 覆寫 workflow config 的 worktree 政策（其餘欄位不留，測試只關心這一鍵）。
fn set_worktree_policy(root: &TempRoot, on: bool) {
    root.write("openspec/config.yaml", &format!("schema: spec-driven\nworktree: {on}\n"));
}

#[test]
fn generation_omits_worktree_skills_when_the_policy_key_is_absent() {
    // Scenario「政策關閉時生成集合不含 worktree 技能」的鍵缺席分支：init 範本
    // 只留註解示例，等同未設＝關。
    let root = TempRoot::new("gate-absent");
    init(&root.dir, &[Tool::Claude], true, "openspec").unwrap();

    for dir in worktree_skill_dirs(Tool::Claude) {
        assert!(!root.exists(&dir), "政策未設時不得生成 {dir}");
    }
    // 其餘技能照常生成。
    assert!(root.exists(".claude/skills/speclink-apply"), "非閘控技能須照常生成");
}

#[test]
fn generation_omits_worktree_skills_when_the_policy_is_false() {
    let root = TempRoot::new("gate-false");
    init(&root.dir, &[Tool::Claude], true, "openspec").unwrap();
    set_worktree_policy(&root, false);

    update(&root.dir, false).unwrap();

    for dir in worktree_skill_dirs(Tool::Claude) {
        assert!(!root.exists(&dir), "政策為 false 時不得生成 {dir}");
    }
    assert!(root.exists(".claude/skills/speclink-apply"));
}

#[test]
fn generation_includes_worktree_skills_when_the_policy_is_on() {
    // Scenario「政策開啟時注入兩顆技能」。
    let root = TempRoot::new("gate-on");
    init(&root.dir, &[Tool::Claude], true, "openspec").unwrap();
    set_worktree_policy(&root, true);

    update(&root.dir, false).unwrap();

    for dir in worktree_skill_dirs(Tool::Claude) {
        assert!(root.exists(&format!("{dir}/SKILL.md")), "政策為 true 時須生成 {dir}");
    }
}

#[test]
fn the_gate_applies_to_codex_and_custom_descriptors_alike() {
    // 需求句「此過濾對 claude、codex 與自訂描述子工具一視同仁」。
    let root = TempRoot::new("gate-all-targets");
    init(&root.dir, &[Tool::Claude, Tool::Codex], true, "openspec").unwrap();
    // init 以非預設資料夾初始化時會寫明 spec_dir；改寫時保留它，免得變成待遷移的舊佈局。
    root.write(
        ".speclink.yaml",
        &format!("spec_dir: openspec\ntools:\n  - claude\n  - codex\n{CUSTOM_DESCRIPTOR}"),
    );

    update(&root.dir, false).unwrap();
    for dir in worktree_skill_dirs(Tool::Codex) {
        assert!(!root.exists(&dir), "政策關閉時 codex 不得生成 {dir}");
    }
    assert!(!root.exists(".wad/skills/speclink-apply-with-worktree"), "描述子亦受閘控");
    assert!(root.exists(".wad/skills/speclink-apply"), "描述子的非閘控技能照常生成");

    set_worktree_policy(&root, true);
    update(&root.dir, false).unwrap();
    for dir in worktree_skill_dirs(Tool::Codex) {
        assert!(root.exists(&dir), "政策開啟時 codex 須生成 {dir}");
    }
    assert!(root.exists(".wad/skills/speclink-apply-with-worktree/SKILL.md"));
}

#[test]
fn an_unparseable_workflow_config_keeps_the_worktree_skills() {
    // 刪除是不可逆方向：政策讀不出來時（使用者手改壞了 config.yaml）一律保留
    // 技能，由技能內的執行期政策檢查兜底，絕不以「讀不到＝關」為由清掉檔案。
    let root = TempRoot::new("gate-broken-config");
    init(&root.dir, &[Tool::Claude], true, "openspec").unwrap();
    set_worktree_policy(&root, true);
    update(&root.dir, false).unwrap();

    root.write("openspec/config.yaml", "schema: [unterminated\n");
    update(&root.dir, false).unwrap();

    for dir in worktree_skill_dirs(Tool::Claude) {
        assert!(root.exists(&dir), "政策文件壞掉時不得清掉 {dir}");
    }
}

// --- update 清除孤兒技能目錄 ---
// Spec requirement: workspace-tools「update 清除孤兒技能目錄」——speclink- 前綴
// 且不在本次應生成集合的目錄於 update 時清除；非前綴目錄不動。

#[test]
fn update_prunes_renamed_skill_directory() {
    let root = TempRoot::new("prune-renamed");
    init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();
    // 舊版生成的目錄：registry 已無此技能名（onboard → baseline 改名遷移）。
    root.write(".claude/skills/speclink-onboard/SKILL.md", "old\n");
    root.write(".agents/skills/speclink-onboard/SKILL.md", "old\n");
    update(&root.dir, false).unwrap();
    assert!(!root.exists(".claude/skills/speclink-onboard"), "舊目錄須被清除");
    assert!(!root.exists(".agents/skills/speclink-onboard"), "舊目錄須被清除");
    assert!(root.exists(".claude/skills/speclink-baseline/SKILL.md"));
    assert!(root.exists(".agents/skills/speclink-baseline/SKILL.md"));
}

#[test]
fn update_keeps_user_skill_directories_without_prefix() {
    let root = TempRoot::new("prune-user-dir");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    let content = "user skill\n";
    root.write(".claude/skills/conventional-commit/SKILL.md", content);
    update(&root.dir, false).unwrap();
    assert_eq!(
        root.read(".claude/skills/conventional-commit/SKILL.md"),
        content,
        "非 speclink- 前綴的使用者技能不受清理影響"
    );
}

#[test]
fn update_prunes_prefixed_directories_not_in_registry() {
    let root = TempRoot::new("prune-prefixed");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write(".claude/skills/speclink-myown/SKILL.md", "mine\n");
    update(&root.dir, false).unwrap();
    // speclink- 前綴保留給生成物：非 registry 的前綴目錄一律清除。
    assert!(!root.exists(".claude/skills/speclink-myown"));
}

fn app_config(yaml: &str) -> crate::config::AppConfig {
    serde_yaml::from_str(yaml).expect("app config parses")
}

/// 工具選集是 `.speclink.yaml` tools 清單的唯一解析點：內建名去重、順序固定
/// claude → codex，沒有清單時才是 legacy 回退。
#[test]
fn tool_selection_resolves_builtins_without_duplicates() {
    let root = TempRoot::new("tool-selection-builtins");
    let sel = ToolSelection::resolve(&root.dir, &app_config("tools: [claude, codex, claude]\n"));
    assert_eq!(sel.builtins, vec![Tool::Claude, Tool::Codex]);
    assert!(!sel.legacy_fallback, "a non-empty list is not the legacy path");
    assert!(sel.customs.is_empty());
    assert!(sel.notes.is_empty());
    assert_eq!(sel.descriptor_error, None);
}

/// 合法描述子與內建名混在同一份清單：兩邊都解析出來，沒有錯誤。
#[test]
fn tool_selection_carries_a_valid_descriptor_beside_the_builtins() {
    let root = TempRoot::new("tool-selection-descriptor");
    let sel = ToolSelection::resolve(
        &root.dir,
        &app_config(&format!("tools:\n  - claude\n{CUSTOM_DESCRIPTOR}")),
    );
    assert_eq!(sel.builtins, vec![Tool::Claude]);
    assert_eq!(sel.customs.len(), 1, "the descriptor resolves: {:?}", sel.customs);
    assert_eq!(sel.customs[0].name, "wad-harness");
    assert_eq!(sel.customs[0].skills_dir, ".wad/skills");
    assert_eq!(sel.descriptor_error, None);
}

/// 壞描述子不在解析時就變成 `Err`：錯誤留在值上由消費端裁決（update 轉錯誤、
/// probe 忽略），內建名照樣解析出來。
#[test]
fn tool_selection_defers_a_bad_descriptor_to_its_consumer() {
    let root = TempRoot::new("tool-selection-bad-descriptor");
    let sel = ToolSelection::resolve(
        &root.dir,
        &app_config("tools:\n  - codex\n  - name: broken-harness\n"),
    );
    assert_eq!(sel.builtins, vec![Tool::Codex], "the builtins still resolve");
    assert!(sel.customs.is_empty(), "an invalid descriptor never reaches the filesystem");
    let message = sel.descriptor_error.expect("the descriptor error is carried on the value");
    assert_eq!(message, "tool descriptor: missing required field 'skills_dir'");
}

/// 空 tools 清單＝legacy 回退：只有 `.claude` 目錄存在時才把 Claude 算進選集。
#[test]
fn tool_selection_falls_back_to_the_claude_footprint_for_an_empty_list() {
    let root = TempRoot::new("tool-selection-empty");
    let app = app_config("tools: []\n");
    let sel = ToolSelection::resolve(&root.dir, &app);
    assert!(sel.builtins.is_empty(), "no .claude directory means nothing to regenerate");
    assert!(sel.legacy_fallback);

    std::fs::create_dir_all(root.at(".claude")).unwrap();
    let sel = ToolSelection::resolve(&root.dir, &app);
    assert_eq!(sel.builtins, vec![Tool::Claude]);
    assert!(sel.legacy_fallback);
}

/// 未知內建名只是警告：訊息字面與今天 `update` 的一模一樣。
#[test]
fn tool_selection_notes_an_unknown_builtin_name() {
    let root = TempRoot::new("tool-selection-unknown");
    let sel = ToolSelection::resolve(&root.dir, &app_config("tools: [claude, windsurf]\n"));
    assert_eq!(sel.builtins, vec![Tool::Claude]);
    assert_eq!(
        sel.notes,
        vec![
            "unknown tool 'windsurf' in .speclink.yaml tools list (supported: claude, codex, copilot, pi, cursor)"
                .to_string()
        ]
    );
    assert!(!sel.legacy_fallback);
}

/// `builtins_only` 是 init／reconcile 的記憶體入口：選集逐字帶過，沒有描述子、
/// 沒有回退、沒有警告。
#[test]
fn tool_selection_builtins_only_takes_the_selection_verbatim() {
    let sel = ToolSelection::builtins_only(&[Tool::Codex]);
    assert_eq!(sel.builtins, vec![Tool::Codex]);
    assert!(sel.customs.is_empty());
    assert!(sel.notes.is_empty());
    assert!(!sel.legacy_fallback);
    assert_eq!(sel.descriptor_error, None);
}

/// 與 `CUSTOM_DESCRIPTOR` 同一個描述子的已驗證形式。
fn custom_target() -> CustomTool {
    CustomTool {
        name: "wad-harness".to_string(),
        skills_dir: ".wad/skills".to_string(),
        instructions_file: Some("WAD.md".to_string()),
        invocation: crate::config::Invocation::Cli,
    }
}

fn dir_names(set: &[(String, String)]) -> Vec<String> {
    set.iter().map(|(dir, _)| dir.clone()).collect()
}

/// registry 依 `non_claude` 過濾後的 `speclink-<name>` 目錄名（順序同 registry）。
fn registry_dirs(non_claude: bool) -> Vec<String> {
    skills::registry()
        .iter()
        .filter(|s| !(non_claude && s.claude_only))
        .map(|s| format!("speclink-{}", s.name))
        .collect()
}

/// 受管技能集合只有一個擁有者：Claude 目標拿 registry 全集，非 Claude 目標拿
/// 非 Claude 子集，每一筆內容逐字等於同參數的 render。
#[test]
fn managed_skills_covers_the_registry_per_target() {
    let custom = custom_target();
    let claude = managed_skills(skills::RenderTarget::Builtin(Tool::Claude), true, "openspec", SpecLocation::InCheckout);
    let codex = managed_skills(skills::RenderTarget::Builtin(Tool::Codex), true, "openspec", SpecLocation::InCheckout);
    let neutral = managed_skills(skills::RenderTarget::Custom(&custom), true, "openspec", SpecLocation::InCheckout);

    assert_eq!(dir_names(&claude), registry_dirs(false));
    assert_eq!(dir_names(&codex), registry_dirs(true));
    assert_eq!(dir_names(&neutral), registry_dirs(true));

    let registry = skills::registry();
    for (target, set) in [
        (skills::RenderTarget::Builtin(Tool::Claude), &claude),
        (skills::RenderTarget::Builtin(Tool::Codex), &codex),
        (skills::RenderTarget::Custom(&custom), &neutral),
    ] {
        // 集合裡的每一筆都比內容——不是「找得到才比」。
        for (dir, content) in set {
            let skill = registry
                .iter()
                .find(|s| format!("speclink-{}", s.name) == *dir)
                .unwrap_or_else(|| panic!("{dir} 不在 registry"));
            assert_eq!(
                content,
                &skills::render_skill_file_for(target, skill, "openspec", SpecLocation::InCheckout),
                "{dir} 的內容必須等於同參數的 render"
            );
        }
    }
}

/// worktree 政策關閉時，三個目標的受管集合都不含兩顆 worktree 技能。
#[test]
fn managed_skills_drops_the_gated_skills_when_the_policy_is_off() {
    let custom = custom_target();
    for set in [
        managed_skills(skills::RenderTarget::Builtin(Tool::Claude), false, "openspec", SpecLocation::InCheckout),
        managed_skills(skills::RenderTarget::Builtin(Tool::Codex), false, "openspec", SpecLocation::InCheckout),
        managed_skills(skills::RenderTarget::Custom(&custom), false, "openspec", SpecLocation::InCheckout),
    ] {
        for gated in ["speclink-apply-with-worktree", "speclink-worktree-merge"] {
            assert!(
                !set.iter().any(|(dir, _)| dir == gated),
                "政策關閉時受管集合不得含 {gated}"
            );
        }
    }
}

/// 計畫的每個 target 帶自己的 skills_root；未選中的內建進 `deselected_builtins`。
#[test]
fn sync_plan_builds_one_target_per_selected_tool() {
    let root = TempRoot::new("sync-plan-targets");
    let plan =
        SyncPlan::resolve(&root.dir, ToolSelection::builtins_only(&[Tool::Codex]), "openspec", SpecLocation::InCheckout);
    assert_eq!(plan.targets.len(), 1, "只選 codex 就只有一個 target");
    assert_eq!(plan.targets[0].label, "codex");
    assert_eq!(plan.targets[0].skills_root, root.at(".agents/skills"));
    assert_eq!(
        plan.deselected_builtins,
        vec![Tool::Claude, Tool::Copilot, Tool::Pi, Tool::Cursor]
    );
}

/// legacy 回退（沒有 tools 清單）不下架任何內建工具。
#[test]
fn sync_plan_prunes_nothing_on_the_legacy_fallback() {
    let root = TempRoot::new("sync-plan-legacy");
    std::fs::create_dir_all(root.at(".claude")).unwrap();
    let selection = ToolSelection::resolve(&root.dir, &app_config("tools: []\n"));
    let plan = SyncPlan::resolve(&root.dir, selection, "openspec", SpecLocation::InCheckout);
    assert_eq!(plan.targets.len(), 1);
    assert_eq!(plan.targets[0].label, "claude");
    assert!(plan.deselected_builtins.is_empty(), "沒有清單就沒有「下架」這回事");
}

/// 描述子各自成為一個 target，skills_root 就是描述子宣告的目錄。
#[test]
fn sync_plan_adds_a_target_for_each_descriptor() {
    let root = TempRoot::new("sync-plan-descriptor");
    let selection = ToolSelection::resolve(
        &root.dir,
        &app_config(&format!("tools:\n  - claude\n{CUSTOM_DESCRIPTOR}")),
    );
    let plan = SyncPlan::resolve(&root.dir, selection, "openspec", SpecLocation::InCheckout);
    assert_eq!(plan.targets.len(), 2);
    assert_eq!(plan.targets[0].label, "claude");
    assert_eq!(plan.targets[1].label, "wad-harness");
    assert_eq!(plan.targets[1].skills_root, root.at(".wad/skills"));
    assert_eq!(
        plan.deselected_builtins,
        vec![Tool::Codex, Tool::Copilot, Tool::Pi, Tool::Cursor]
    );
}

/// 守門的檢查面就是 targets 的 skills_root 集合：只有描述子目錄領先版本時
/// 一樣被拒，訊息含工作區與引擎兩個版號。
#[test]
fn sync_plan_guard_checks_every_targets_skills_root() {
    let root = TempRoot::new("sync-plan-guard");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write(".speclink.yaml", &format!("tools:\n{CUSTOM_DESCRIPTOR}"));
    update(&root.dir, false).expect("先生成描述子受管檔");
    let ahead = ahead_of_current();
    crate::testkit::set_skill_version(&root.at(".wad/skills"), &ahead);

    let app = crate::config::AppConfig::load(&root.at(".speclink.yaml")).expect("config parses");
    let plan = SyncPlan::resolve(&root.dir, ToolSelection::resolve(&root.dir, &app), "openspec", SpecLocation::InCheckout);
    assert_eq!(
        plan.targets.iter().map(|t| t.skills_root.clone()).collect::<Vec<PathBuf>>(),
        vec![root.at(".wad/skills")],
        "守門的檢查面等於 targets 的 skills_root"
    );

    let message = plan.guard().expect_err("只有描述子目錄領先也必須被拒").to_string();
    assert!(message.contains(&ahead), "訊息須含工作區版號：{message}");
    assert!(message.contains(ASSET_VERSION), "訊息須含引擎版號：{message}");
}

/// `init --force` 會把 `openspec/config.yaml` 寫回範本（worktree 政策關閉）：上一次
/// 政策開啟留下的兩顆 worktree 技能目錄必須跟著消失（舊 `skip_gated_skill` 的行為），
/// 其餘技能照常在。
#[test]
fn init_force_removes_gated_skill_directories_the_reset_policy_no_longer_allows() {
    let root = TempRoot::new("init-force-gated");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    set_worktree_policy(&root, true);
    update(&root.dir, false).unwrap();
    for dir in worktree_skill_dirs(Tool::Claude) {
        assert!(root.exists(&format!("{dir}/SKILL.md")), "前置：政策開啟時 {dir} 存在");
    }

    init(&root.dir, &[Tool::Claude], true, "openspec").unwrap();

    assert!(
        !root.read("openspec/config.yaml").contains("\nworktree: true"),
        "--force 把 config.yaml 寫回範本"
    );
    for dir in worktree_skill_dirs(Tool::Claude) {
        assert!(!root.exists(&dir), "政策被重置為關閉後 {dir} 不得留下");
    }
    assert!(root.exists(".claude/skills/speclink-apply/SKILL.md"));
}

// --- 內建 copilot（change: add-copilot-tool）---
// Spec requirement「Copilot 渲染目標」與「worktree 技能的政策條件式生成」的 copilot
// 分支：技能落在 .github/skills、子集與內容沿用 codex，只有叫用前綴與 agent 名不同。

#[test]
fn copilot_generates_the_codex_subset_under_github_skills() {
    // Scenario「Copilot 與 Codex 生成集合相同」：非 Claude 子集，政策關閉時不含兩顆
    // worktree 技能。
    let root = TempRoot::new("copilot-subset");
    init(&root.dir, &[Tool::Copilot], false, "openspec").unwrap();

    assert_eq!(Tool::Copilot.skills_dir(), ".github/skills");
    let generated = skill_dirs(&root, ".github/skills");
    assert_eq!(
        generated,
        expected_dirs(Tool::Codex),
        "copilot 的生成集合須等於 codex 的"
    );
    assert!(
        !generated.contains(&"speclink-analyze".to_string()),
        "analyze 只給 claude"
    );
    for dir in worktree_skill_dirs(Tool::Copilot) {
        assert!(!root.exists(&dir), "政策關閉時不得生成 {dir}");
    }
}

#[test]
fn copilot_follows_the_worktree_policy_like_the_other_targets() {
    // Requirement「worktree 技能的政策條件式生成」對 copilot 一視同仁：開啟即生成、
    // 改回關閉後再生即清理。
    let root = TempRoot::new("copilot-worktree");
    init(&root.dir, &[Tool::Copilot], false, "openspec").unwrap();

    set_worktree_policy(&root, true);
    update(&root.dir, false).unwrap();
    for dir in worktree_skill_dirs(Tool::Copilot) {
        assert!(root.exists(&dir), "政策開啟時須生成 {dir}");
    }

    set_worktree_policy(&root, false);
    update(&root.dir, false).unwrap();
    for dir in worktree_skill_dirs(Tool::Copilot) {
        assert!(!root.exists(&dir), "政策改回關閉後須清除 {dir}");
    }
}

#[test]
fn copilot_skill_has_neutral_frontmatter_and_slash_references() {
    // Scenario「選取 Copilot 生成正確的技能檔」。
    let root = TempRoot::new("copilot-render");
    init(&root.dir, &[Tool::Copilot], false, "openspec").unwrap();

    let propose = root.read(&propose_skill(Tool::Copilot));
    assert_eq!(
        frontmatter_keys(&propose),
        ["name", "description", "license", "compatibility", "metadata"]
    );
    assert!(propose.contains("name: speclink-propose\n"), "{propose}");
    assert!(
        propose.contains("/speclink-apply"),
        "技能引用須寫成 /speclink-<name>"
    );
    assert!(
        !propose.contains("$speclink-"),
        "不得出現 Codex 的 $ 叫用語法"
    );
    assert!(
        propose.contains("--agent copilot"),
        "建立變更須記錄 agent 為 copilot"
    );

    // fork 與唯讀技能在 claude 版才帶專用欄位與前言；copilot 版一律不帶。
    for skill in ["drift", "trace", "discuss", "verify"] {
        let text = root.read(&format!(".github/skills/speclink-{skill}/SKILL.md"));
        for claude_only in [
            "context: fork",
            "agent: Explore",
            "disallowedTools",
            "## Claude fork context",
        ] {
            assert!(
                !text.contains(claude_only),
                "speclink-{skill} 不得含 {claude_only}"
            );
        }
    }
}

/// 各內建工具的技能叫用前綴（規格「內建工具的渲染差異」表的「技能引用寫法」欄）。
/// 測試自帶一份：拿來比對的期望值不從被測的程式取。
fn invocation_prefix(tool: Tool) -> &'static str {
    match tool {
        Tool::Claude | Tool::Copilot | Tool::Cursor => "/speclink-",
        Tool::Codex => "$speclink-",
        Tool::Pi => "/skill:speclink-",
    }
}

/// 把一個內建工具的技能內文換成另一個工具的：叫用前綴，加上 `{{TOOL}}` 代換值出現的
/// 每個位置——`--agent` 的值、子代理模型的格子路徑與寫入指令範例。只換這些位置，其他
/// 差異照樣失敗（例如 `~/.codex/config.toml` 這類寫死的路徑不能跟著換）。
fn swap_tool(text: &str, from: Tool, to: Tool) -> String {
    let (from_tool, to_tool) = (from.name(), to.name());
    let mut out = text
        .replace(invocation_prefix(from), invocation_prefix(to))
        .replace(&format!("--agent {from_tool}"), &format!("--agent {to_tool}"));
    for prefix in ["subagentModel", "subagent_model"] {
        for station in ["<station>", "review", "verify"] {
            out = out.replace(
                &format!("{prefix}.{station}.{from_tool}"),
                &format!("{prefix}.{station}.{to_tool}"),
            );
        }
    }
    out.replace(
        &format!("subagent-model <station> {from_tool}"),
        &format!("subagent-model <station> {to_tool}"),
    )
}

#[test]
fn copilot_skills_differ_from_codex_only_by_prefix_and_agent_name() {
    // Requirement「Copilot 渲染目標」：除叫用前綴與 {{TOOL}} 代換值以外逐字相同。
    let root = TempRoot::new("copilot-vs-codex");
    init(&root.dir, &[Tool::Codex, Tool::Copilot], false, "openspec").unwrap();

    for dir in expected_dirs(Tool::Codex) {
        let codex = root.read(&format!(".agents/skills/{dir}/SKILL.md"));
        let copilot = root.read(&format!(".github/skills/{dir}/SKILL.md"));
        assert_eq!(copilot, swap_tool(&codex, Tool::Codex, Tool::Copilot), "{dir}");
    }
}

#[test]
fn copilot_writes_no_instruction_file() {
    // Scenario「選取 Copilot 不產生指令檔」。
    let root = TempRoot::new("copilot-no-instructions");
    init(&root.dir, &[Tool::Copilot], false, "openspec").unwrap();

    for file in ["AGENTS.md", "CLAUDE.md", ".github/copilot-instructions.md"] {
        assert!(!root.exists(file), "{file} 不得生成");
    }
    let entries: Vec<String> = std::fs::read_dir(root.at(".github"))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(entries, ["skills"], ".github/ 下只能有 skills/");
}

#[test]
fn footprint_detection_never_infers_copilot() {
    // Scenario「footprint 偵測不推斷 Copilot」：多數 GitHub repo 都有 .github/，
    // 它不代表使用者要 Copilot；legacy 回退的 update 也只依 .claude/ 再生。
    let root = TempRoot::new("copilot-footprint");
    root.write(".github/workflows/ci.yml", "on: push\n");
    std::fs::create_dir_all(root.at(".github/skills/my-skill")).unwrap();
    std::fs::create_dir_all(root.at(".claude")).unwrap();

    assert_eq!(detect_footprint_tools(&root.dir), vec![Tool::Claude]);

    root.write(".speclink.yaml", "# Speclink application config\n");
    update(&root.dir, false).expect("legacy update");
    assert!(
        root.exists(&propose_skill(Tool::Claude)),
        "claude 技能照常再生"
    );
    assert!(
        skill_dirs(&root, ".github/skills").is_empty(),
        "不得生成任何 copilot 技能"
    );
    assert_eq!(root.read(".github/workflows/ci.yml"), "on: push\n");
}

#[test]
fn deselecting_copilot_removes_only_speclink_skill_dirs() {
    // Scenario「取消 Copilot 只移除 speclink 技能目錄」（built-in tools 權威收斂）。
    let root = TempRoot::new("copilot-deselect");
    init(&root.dir, &[Tool::Claude, Tool::Copilot], false, "openspec").unwrap();
    root.write(
        ".github/skills/my-skill/SKILL.md",
        "---\nname: my-skill\n---\n",
    );
    root.write(".github/workflows/ci.yml", "on: push\n");
    root.write(".github/copilot-instructions.md", "團隊自己的指示\n");

    reconcile_builtin_tools(&root.dir, &[Tool::Claude]).expect("reconcile succeeds");

    assert!(
        skill_dirs(&root, ".github/skills").is_empty(),
        "speclink- 目錄須全數移除"
    );
    assert_eq!(
        root.read(".github/skills/my-skill/SKILL.md"),
        "---\nname: my-skill\n---\n"
    );
    assert_eq!(root.read(".github/workflows/ci.yml"), "on: push\n");
    assert_eq!(
        root.read(".github/copilot-instructions.md"),
        "團隊自己的指示\n"
    );
    assert!(
        root.exists(&propose_skill(Tool::Claude)),
        "claude 技能照常更新"
    );
    assert_eq!(builtin_names(&root), ["claude"]);
}

#[test]
fn deselecting_copilot_drops_the_emptied_skills_dir_but_keeps_github() {
    // 只剩 speclink 技能的 .github/skills/ 變空即移除；.github/ 還有 workflows 就留著。
    let root = TempRoot::new("copilot-deselect-empty");
    init(&root.dir, &[Tool::Claude, Tool::Copilot], false, "openspec").unwrap();
    root.write(".github/workflows/ci.yml", "on: push\n");

    reconcile_builtin_tools(&root.dir, &[Tool::Claude]).expect("reconcile succeeds");

    assert!(
        !root.exists(".github/skills"),
        "變空的 .github/skills/ 須移除"
    );
    assert_eq!(root.read(".github/workflows/ci.yml"), "on: push\n");
}

#[cfg(unix)]
#[test]
fn update_never_prunes_through_a_symlinked_github_skills_dir() {
    // 沒選 copilot 的 update 會清 .github/skills；它若是指向 .claude/skills 的
    // symlink，清理不得穿過去刪掉 claude 剛寫好的技能。
    let root = TempRoot::new("copilot-symlink");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    std::fs::create_dir_all(root.at(".github")).unwrap();
    std::os::unix::fs::symlink("../.claude/skills", root.at(".github/skills")).unwrap();

    let outcome = update(&root.dir, false).expect("update succeeds");

    assert!(
        root.exists(&propose_skill(Tool::Claude)),
        "claude 技能不得被刪"
    );
    let link = std::fs::symlink_metadata(root.at(".github/skills")).unwrap();
    assert!(link.file_type().is_symlink(), "symlink 本身留著");
    assert!(
        !outcome.pruned.iter().any(|p| p == "copilot"),
        "沒有清掉任何東西就不回報 copilot：{:?}",
        outcome.pruned
    );
}

#[test]
fn update_keeps_a_preexisting_empty_github_skills_dir() {
    // 從沒選過 copilot：本來就空的 .github/skills/ 不是這次清空的，不得被收掉。
    let root = TempRoot::new("copilot-empty-kept");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    std::fs::create_dir_all(root.at(".github/skills")).unwrap();

    update(&root.dir, false).expect("update succeeds");

    assert!(
        root.exists(".github/skills"),
        "原本就空的 .github/skills/ 須保留"
    );
}

#[test]
fn update_prunes_an_orphan_skill_dir_under_github_skills() {
    // Requirement「update 清除孤兒技能目錄」：copilot 與 codex 同為非 Claude 子集，
    // 改名留下的舊目錄一樣被清。
    let root = TempRoot::new("copilot-orphan");
    init(&root.dir, &[Tool::Copilot], false, "openspec").unwrap();
    root.write(".github/skills/speclink-onboard/SKILL.md", "old\n");

    update(&root.dir, false).expect("update succeeds");

    assert!(
        !root.exists(".github/skills/speclink-onboard"),
        "舊目錄須被清除"
    );
    assert!(root.exists(".github/skills/speclink-baseline/SKILL.md"));
}

#[test]
fn skill_probe_reports_missing_copilot_skills() {
    // Requirement「技能檔過期探測」：逐工具資訊的內建名含 copilot，缺失的差異清單
    // 落在 .github/skills/ 底下。
    let root = TempRoot::new("copilot-probe-missing");
    init(&root.dir, &[Tool::Claude, Tool::Copilot], false, "openspec").unwrap();
    std::fs::remove_dir_all(root.at(Tool::Copilot.skills_dir())).unwrap();

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Missing, "{probe:?}");
    let copilot = probe
        .tools
        .iter()
        .find(|t| t.tool == "copilot")
        .expect("copilot 在列");
    assert!(
        copilot.missing && copilot.workspace_version.is_none(),
        "{copilot:?}"
    );
    assert!(!probe.differing_files.is_empty());
    for file in &probe.differing_files {
        assert!(file.starts_with(".github/skills/speclink-"), "{file}");
        assert!(file.ends_with("/SKILL.md"), "{file}");
    }
}

#[test]
fn skill_probe_reports_stale_copilot_skills() {
    let root = TempRoot::new("copilot-probe-stale");
    init(&root.dir, &[Tool::Copilot], false, "openspec").unwrap();
    set_skill_version(&root, Tool::Copilot, "v0.9.0");

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Stale, "{probe:?}");
    assert_eq!(probe.tools.len(), 1);
    assert_eq!(probe.tools[0].tool, "copilot");
    assert!(probe.tools[0].stale);
    assert!(probe
        .differing_files
        .contains(&propose_skill(Tool::Copilot)));
}

// --- 內建 pi 與 cursor（change: add-pi-cursor-tools）---
// Spec requirement「Pi 渲染目標」「Cursor 渲染目標」與「worktree 技能的政策條件式生成」
// 的 pi、cursor 分支：技能落在 .pi/skills、.cursor/skills，子集與內容沿用 codex，
// 只有叫用前綴與 agent 名不同。

/// 這次新增的兩個內建工具。
const PI_AND_CURSOR: [Tool; 2] = [Tool::Pi, Tool::Cursor];

/// 一個專案根相對目錄下的項目名（排序後）。
fn entry_names(root: &TempRoot, rel: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(root.at(rel))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
}

/// frontmatter 的頂層鍵（依出現順序）。
fn frontmatter_keys(skill_file: &str) -> Vec<String> {
    skill_file
        .split("---\n")
        .nth(1)
        .expect("frontmatter block")
        .lines()
        .filter(|l| !l.starts_with(' '))
        .filter_map(|l| l.split(':').next())
        .map(str::to_string)
        .collect()
}

#[test]
fn pi_and_cursor_generate_the_codex_subset_under_their_own_skills_dir() {
    // Scenario「Pi 與 Codex 並存時各寫各的目錄」「Cursor 與 Codex 生成集合相同」的
    // 集合面：非 Claude 子集，政策關閉時不含兩顆 worktree 技能。
    assert_eq!(Tool::Pi.skills_dir(), ".pi/skills");
    assert_eq!(Tool::Cursor.skills_dir(), ".cursor/skills");
    for tool in PI_AND_CURSOR {
        let root = TempRoot::new(&format!("{}-subset", tool.name()));
        init(&root.dir, &[tool], false, "openspec").unwrap();

        let generated = skill_dirs(&root, tool.skills_dir());
        assert_eq!(generated, expected_dirs(Tool::Codex), "{tool:?} 的生成集合須等於 codex 的");
        assert!(!generated.contains(&"speclink-analyze".to_string()), "analyze 只給 claude");
        for dir in worktree_skill_dirs(tool) {
            assert!(!root.exists(&dir), "政策關閉時不得生成 {dir}");
        }
    }
}

#[test]
fn pi_and_cursor_follow_the_worktree_policy_like_the_other_targets() {
    // Requirement「worktree 技能的政策條件式生成」對 pi、cursor 一視同仁。
    for tool in PI_AND_CURSOR {
        let root = TempRoot::new(&format!("{}-worktree", tool.name()));
        init(&root.dir, &[tool], false, "openspec").unwrap();

        set_worktree_policy(&root, true);
        update(&root.dir, false).unwrap();
        for dir in worktree_skill_dirs(tool) {
            assert!(root.exists(&format!("{dir}/SKILL.md")), "政策開啟時須生成 {dir}");
        }

        set_worktree_policy(&root, false);
        update(&root.dir, false).unwrap();
        for dir in worktree_skill_dirs(tool) {
            assert!(!root.exists(&dir), "政策改回關閉後須清除 {dir}");
        }
    }
}

#[test]
fn pi_and_cursor_skills_have_neutral_frontmatter_and_their_own_references() {
    // Scenario「選取 Pi 生成正確的技能檔」「選取 Cursor 生成正確的技能檔」：name 與目錄名
    // 相同；pi 的技能檔連斜線形式的引用都不得有（`/skill:speclink-` 不含 `/speclink-`）。
    let rows: [(Tool, &str, &[&str]); 2] = [
        (Tool::Pi, "/skill:speclink-apply", &["/speclink-"]),
        (Tool::Cursor, "/speclink-apply", &[]),
    ];
    for (tool, reference, extra_forbidden) in rows {
        let root = TempRoot::new(&format!("{}-render", tool.name()));
        init(&root.dir, &[tool], false, "openspec").unwrap();

        let propose = root.read(&propose_skill(tool));
        assert_eq!(
            frontmatter_keys(&propose),
            ["name", "description", "license", "compatibility", "metadata"],
            "{tool:?}"
        );
        assert!(propose.contains("name: speclink-propose\n"), "{tool:?}: {propose}");
        assert!(propose.contains(reference), "{tool:?}: 技能引用須寫成 {reference}");
        assert!(
            propose.contains(&format!("--agent {}", tool.name())),
            "{tool:?}: 建立變更須記錄 agent 為 {}",
            tool.name()
        );

        let forbidden = ["$speclink-", "## Claude fork context", "context: fork", "agent: Explore", "disallowedTools"];
        for dir in expected_dirs(tool) {
            let text = root.read(&format!("{}/{dir}/SKILL.md", tool.skills_dir()));
            for needle in forbidden.iter().chain(extra_forbidden) {
                assert!(!text.contains(needle), "{tool:?} {dir} 不得含 {needle}");
            }
        }
    }
}

#[test]
fn pi_and_cursor_skills_differ_from_codex_only_by_prefix_and_tool_name() {
    // Requirement「Pi 渲染目標」「Cursor 渲染目標」：除叫用前綴與 {{TOOL}} 代換值以外，
    // 與同一技能的 codex 內容逐字相同；同時選取時各寫各的目錄、不合併。
    for tool in PI_AND_CURSOR {
        let root = TempRoot::new(&format!("{}-vs-codex", tool.name()));
        init(&root.dir, &[Tool::Codex, tool], false, "openspec").unwrap();

        assert_eq!(
            skill_dirs(&root, tool.skills_dir()),
            skill_dirs(&root, ".agents/skills"),
            "{tool:?} 與 codex 的目錄名集合須相同"
        );
        for dir in expected_dirs(Tool::Codex) {
            let codex = root.read(&format!(".agents/skills/{dir}/SKILL.md"));
            let own = root.read(&format!("{}/{dir}/SKILL.md", tool.skills_dir()));
            assert_eq!(swap_tool(&own, tool, Tool::Codex), codex, "{tool:?} {dir}");
        }
    }
}

#[test]
fn builtin_render_differences_match_the_spec_table() {
    // Example「內建工具的渲染差異」逐列：skills 目錄、技能引用寫法、--agent 值、
    // Claude 專用 frontmatter、是否含 analyze。
    let rows: [(Tool, &str, &str, &str, bool); 5] = [
        (Tool::Claude, ".claude/skills", "/speclink-apply", "claude", true),
        (Tool::Codex, ".agents/skills", "$speclink-apply", "codex", false),
        (Tool::Copilot, ".github/skills", "/speclink-apply", "copilot", false),
        (Tool::Pi, ".pi/skills", "/skill:speclink-apply", "pi", false),
        (Tool::Cursor, ".cursor/skills", "/speclink-apply", "cursor", false),
    ];
    assert_eq!(Tool::ALL.to_vec(), rows.map(|r| r.0).to_vec(), "內建工具的固定順序");
    for (tool, dir, reference, agent, claude) in rows {
        let root = TempRoot::new(&format!("render-table-{agent}"));
        init(&root.dir, &[tool], false, "openspec").unwrap();

        assert_eq!(tool.skills_dir(), dir);
        let propose = root.read(&format!("{dir}/speclink-propose/SKILL.md"));
        assert!(propose.contains(reference), "{agent}: 技能引用須寫成 {reference}");
        assert!(propose.contains(&format!("--agent {agent}")), "{agent}: --agent 值");
        assert_eq!(root.exists(&format!("{dir}/speclink-analyze")), claude, "{agent}: analyze");
        let drift = root.read(&format!("{dir}/speclink-drift/SKILL.md"));
        assert_eq!(drift.contains("context: fork\nagent: Explore\n"), claude, "{agent}: fork 欄位");
        assert_eq!(drift.contains("disallowedTools"), claude, "{agent}: 唯讀欄位");
    }
}

#[test]
fn pi_and_cursor_write_no_instruction_file_and_nothing_beside_skills() {
    // Requirement「Pi 渲染目標」「Cursor 渲染目標」：不產生指令檔，`.pi/`、`.cursor/`
    // 下只有 skills/（沒有 .cursor/rules/、.cursor/agents/）。
    for tool in PI_AND_CURSOR {
        let root = TempRoot::new(&format!("{}-no-instructions", tool.name()));
        init(&root.dir, &[tool], false, "openspec").unwrap();

        for file in ["AGENTS.md", "CLAUDE.md"] {
            assert!(!root.exists(file), "{tool:?}: {file} 不得生成");
        }
        let own_dir = tool.skills_dir().trim_end_matches("/skills");
        assert_eq!(entry_names(&root, own_dir), ["skills"], "{own_dir}/ 下只能有 skills/");
        assert_eq!(instructions_path(tool), None, "{tool:?} 沒有指令檔");
    }
}

#[test]
fn footprint_detection_never_infers_pi_or_cursor() {
    // Scenario「footprint 偵測不推斷 Pi」「footprint 偵測不推斷 Cursor」：`.pi/`、
    // `.cursor/` 存在不代表使用者要這個工具；legacy 回退的 update 只依 .claude/ 再生。
    for (tool, user_file, content) in [
        (Tool::Pi, ".pi/settings.json", "{\"theme\": \"dark\"}\n"),
        (Tool::Cursor, ".cursor/rules/style.mdc", "---\nalwaysApply: true\n---\n"),
    ] {
        let root = TempRoot::new(&format!("{}-footprint", tool.name()));
        root.write(user_file, content);
        // skills 目錄本身存在（放著使用者自己的技能）也不算 footprint。
        let user_skill = format!("{}/my-skill/SKILL.md", tool.skills_dir());
        root.write(&user_skill, "---\nname: my-skill\n---\n");
        std::fs::create_dir_all(root.at(".claude")).unwrap();

        assert_eq!(detect_footprint_tools(&root.dir), vec![Tool::Claude]);

        root.write(".speclink.yaml", "# Speclink application config\n");
        update(&root.dir, false).expect("legacy update");
        assert!(root.exists(&propose_skill(Tool::Claude)), "claude 技能照常再生");
        assert!(
            skill_dirs(&root, tool.skills_dir()).is_empty(),
            "{} 下不得生成 speclink 技能",
            tool.skills_dir()
        );
        assert_eq!(root.read(user_file), content, "{user_file} 須位元級不變");
        assert_eq!(root.read(&user_skill), "---\nname: my-skill\n---\n", "{user_skill} 須位元級不變");
    }
}

#[test]
fn deselecting_cursor_removes_only_speclink_skill_dirs() {
    // Scenario「取消 Cursor 只移除 speclink 技能目錄」（built-in tools 權威收斂）。
    let root = TempRoot::new("cursor-deselect");
    init(&root.dir, &[Tool::Claude, Tool::Cursor], false, "openspec").unwrap();
    root.write(".cursor/skills/my-skill/SKILL.md", "---\nname: my-skill\n---\n");
    root.write(".cursor/rules/style.mdc", "---\nalwaysApply: true\n---\n");

    reconcile_builtin_tools(&root.dir, &[Tool::Claude]).expect("reconcile succeeds");

    assert!(skill_dirs(&root, ".cursor/skills").is_empty(), "speclink- 目錄須全數移除");
    assert_eq!(root.read(".cursor/skills/my-skill/SKILL.md"), "---\nname: my-skill\n---\n");
    assert_eq!(root.read(".cursor/rules/style.mdc"), "---\nalwaysApply: true\n---\n");
    assert_eq!(entry_names(&root, ".cursor"), ["rules", "skills"], ".cursor/ 下不得多出別的項目");
    assert!(root.exists(&propose_skill(Tool::Claude)), "claude 技能照常更新");
    assert_eq!(builtin_names(&root), ["claude"]);
}

#[test]
fn deselecting_pi_drops_the_emptied_skills_dir_and_its_parent() {
    // Scenario「取消 Pi 後變空的目錄一併移除」：`.pi/` 下只有生成物時整個收掉。
    let root = TempRoot::new("pi-deselect-empty");
    init(&root.dir, &[Tool::Codex, Tool::Pi], false, "openspec").unwrap();
    assert!(root.exists(&propose_skill(Tool::Pi)), "前置：pi 技能已生成");

    reconcile_builtin_tools(&root.dir, &[Tool::Codex]).expect("reconcile succeeds");

    assert!(!root.exists(".pi/skills"), "變空的 .pi/skills/ 須移除");
    assert!(!root.exists(".pi"), "變空的 .pi/ 須移除");
    assert_eq!(skill_dirs(&root, ".agents/skills"), expected_dirs(Tool::Codex), "codex 技能照常更新");
    assert_eq!(builtin_names(&root), ["codex"]);
}

#[test]
fn deselecting_pi_keeps_the_users_own_files_under_pi() {
    // Implementation Contract：取消勾選不動 `.pi/` 下其他內容（pi 的 settings.json）。
    let root = TempRoot::new("pi-deselect-settings");
    init(&root.dir, &[Tool::Codex, Tool::Pi], false, "openspec").unwrap();
    root.write(".pi/settings.json", "{\"theme\": \"dark\"}\n");

    reconcile_builtin_tools(&root.dir, &[Tool::Codex]).expect("reconcile succeeds");

    assert!(!root.exists(".pi/skills"), "變空的 .pi/skills/ 須移除");
    assert_eq!(root.read(".pi/settings.json"), "{\"theme\": \"dark\"}\n");
}

#[test]
fn update_prunes_an_orphan_skill_dir_under_pi_skills() {
    // Requirement「update 清除孤兒技能目錄」：pi 與 codex 同為非 Claude 子集。
    let root = TempRoot::new("pi-orphan");
    init(&root.dir, &[Tool::Pi], false, "openspec").unwrap();
    root.write(".pi/skills/speclink-onboard/SKILL.md", "old\n");

    update(&root.dir, false).expect("update succeeds");

    assert!(!root.exists(".pi/skills/speclink-onboard"), "舊目錄須被清除");
    assert!(root.exists(".pi/skills/speclink-baseline/SKILL.md"));
}

#[test]
fn skill_probe_reports_missing_pi_skills() {
    // Scenario「Pi 技能目錄缺少判缺失」：差異清單只落在 .pi/skills/ 底下。
    let root = TempRoot::new("pi-probe-missing");
    init(&root.dir, &[Tool::Codex, Tool::Pi], false, "openspec").unwrap();
    std::fs::remove_dir_all(root.at(".pi/skills")).unwrap();

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Missing, "{probe:?}");
    let pi = probe.tools.iter().find(|t| t.tool == "pi").expect("pi 在列");
    assert!(pi.missing && pi.workspace_version.is_none(), "{pi:?}");
    assert!(!probe.differing_files.is_empty());
    for file in &probe.differing_files {
        assert!(file.starts_with(".pi/skills/speclink-"), "{file}");
        assert!(file.ends_with("/SKILL.md"), "{file}");
    }
}

#[cfg(unix)]
#[test]
fn update_never_prunes_through_a_symlinked_cursor_or_pi_skills_dir() {
    // Requirement「built-in tools 權威收斂」：未選取 built-in 的 skills 目錄是 symlink 時
    // 不得穿過去清，否則會刪掉另一個工具剛寫好的技能。這裡只鎖定「不穿過連結清理」；
    // 已勾選工具共用目錄的連結在同步前就被拒（見下方的 shared-dir 測試）。
    for (tool, own_dir, selected) in [
        (Tool::Cursor, ".cursor", Tool::Claude),
        (Tool::Pi, ".pi", Tool::Codex),
    ] {
        let root = TempRoot::new(&format!("{}-symlink", tool.name()));
        init(&root.dir, &[selected], false, "openspec").unwrap();
        std::fs::create_dir_all(root.at(own_dir)).unwrap();
        let target = format!("../{}", selected.skills_dir());
        std::os::unix::fs::symlink(&target, root.at(tool.skills_dir())).unwrap();

        let outcome = update(&root.dir, false).expect("update succeeds");

        assert!(root.exists(&propose_skill(selected)), "{selected:?} 技能不得被刪");
        let link = std::fs::symlink_metadata(root.at(tool.skills_dir())).unwrap();
        assert!(link.file_type().is_symlink(), "symlink 本身留著");
        assert!(
            !outcome.pruned.iter().any(|p| p == tool.name()),
            "沒有清掉任何東西就不回報 {}：{:?}",
            tool.name(),
            outcome.pruned
        );
    }
}

// --- 經 symlink 共用的 skills 目錄（品質關卡第 1 輪；決策「同步前拒絕經 symlink 共用的
// skills 目錄」）。Requirement「built-in tools 權威收斂」的四個 symlink 情境。

#[cfg(unix)]
fn link(root: &TempRoot, target: impl AsRef<Path>, at: &str) {
    let path = root.at(at);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(target, path).unwrap();
}

/// 拒絕的訊息是一行，且點名兩個目錄與兩個工具。
#[cfg(unix)]
fn assert_shared_dir_refusal(err: anyhow::Error, dirs: [&str; 2], tools: [&str; 2]) {
    let message = err.to_string();
    assert_eq!(message.lines().count(), 1, "一行錯誤：{message}");
    for needle in dirs.iter().chain(tools.iter()) {
        assert!(message.contains(needle), "訊息須含 {needle}：{message}");
    }
}

#[cfg(unix)]
#[test]
fn sync_refuses_a_selected_skills_dir_that_resolves_to_another_selected_tools_dir() {
    // Scenario「已勾選工具的 skills 目錄經 symlink 指到另一個工具的目錄時拒絕」：
    // --allow-downgrade 也不能越過。
    for (allow_downgrade, tag) in [(false, "plain"), (true, "allow-downgrade")] {
        let root = TempRoot::new(&format!("shared-dir-cursor-{tag}"));
        init(&root.dir, &[Tool::Claude], false, DEFAULT_SPEC_DIR).unwrap();
        link(&root, "../.claude/skills", ".cursor/skills");
        root.write(".speclink.yaml", "tools: [claude, cursor]\n");
        let before = snapshot(&root);

        let err = update(&root.dir, allow_downgrade).expect_err("共用目錄必須被拒");

        assert_shared_dir_refusal(err, [".cursor/skills", ".claude/skills"], ["cursor", "claude"]);
        assert_eq!(snapshot(&root), before, "{tag}: 拒絕＝零寫入");
    }
}

#[cfg(unix)]
#[test]
fn sync_refuses_a_pi_skills_dir_linked_to_the_codex_dir() {
    // 同一條規則套在 pi 連到 .agents/skills。
    let root = TempRoot::new("shared-dir-pi");
    init(&root.dir, &[Tool::Codex], false, DEFAULT_SPEC_DIR).unwrap();
    link(&root, "../.agents/skills", ".pi/skills");
    root.write(".speclink.yaml", "tools: [codex, pi]\n");
    let before = snapshot(&root);

    let err = update(&root.dir, false).expect_err("共用目錄必須被拒");

    assert_shared_dir_refusal(err, [".pi/skills", ".agents/skills"], ["pi", "codex"]);
    assert_eq!(snapshot(&root), before, "拒絕＝零寫入");
}

#[cfg(unix)]
#[test]
fn reconcile_refuses_a_link_into_a_deselected_tools_dir() {
    // Scenario「指到未選取工具目錄的 symlink 同樣被拒」：不會先寫入 Cursor 版、再把它當成
    // 未選取的 claude 清掉；.speclink.yaml 也不得先被改寫。
    let root = TempRoot::new("shared-dir-deselected");
    init(&root.dir, &[Tool::Claude], false, DEFAULT_SPEC_DIR).unwrap();
    link(&root, "../.claude/skills", ".cursor/skills");
    let before = snapshot(&root);

    let err = reconcile_builtin_tools(&root.dir, &[Tool::Cursor]).expect_err("必須被拒");

    assert_shared_dir_refusal(err, [".cursor/skills", ".claude/skills"], ["cursor", "claude"]);
    assert_eq!(snapshot(&root), before, "拒絕＝零寫入");
}

#[cfg(unix)]
#[test]
fn a_link_to_a_folder_outside_the_project_still_syncs() {
    // Scenario「指到其他位置的 symlink 照常同步」：只擋兩個工具共用同一個目錄。
    let shared = TempRoot::new("shared-dir-outside-target");
    let root = TempRoot::new("shared-dir-outside");
    link(&root, &shared.dir, ".claude");

    init(&root.dir, &[Tool::Claude], false, DEFAULT_SPEC_DIR).expect("init 照常成功");
    update(&root.dir, false).expect("update 照常成功");

    assert!(shared.exists("skills/speclink-propose/SKILL.md"), "技能寫進共用資料夾");
}

#[cfg(unix)]
#[test]
fn update_never_prunes_through_a_symlinked_parent_of_a_deselected_skills_dir() {
    // Scenario「取消勾選工具的上層目錄是 symlink 時不清理」：.cursor 連到 .claude，
    // .cursor/skills 就是 .claude/skills；清理 cursor 不得刪掉剛寫好的 Claude 技能。
    let root = TempRoot::new("shared-dir-parent");
    init(&root.dir, &[Tool::Claude], false, DEFAULT_SPEC_DIR).unwrap();
    link(&root, ".claude", ".cursor");

    let outcome = update(&root.dir, false).expect("update succeeds");

    assert_eq!(skill_dirs(&root, ".claude/skills"), expected_dirs(Tool::Claude), "Claude 技能全在");
    let meta = std::fs::symlink_metadata(root.at(".cursor")).unwrap();
    assert!(meta.file_type().is_symlink(), ".cursor symlink 本身保留");
    assert!(!outcome.pruned.iter().any(|p| p == "cursor"), "{:?}", outcome.pruned);
}

#[cfg(unix)]
#[test]
fn sync_refuses_a_link_whose_target_does_not_exist_yet() {
    // 連結目標還不存在（先建好連結再同步，或刪掉 .claude/skills 想重新產生）：同步會先建出
    // .claude/skills、連結隨即生效，所以解析時要順著懸空連結走，照樣在寫入前拒絕。
    let root = TempRoot::new("shared-dir-dangling");
    init(&root.dir, &[Tool::Claude], false, DEFAULT_SPEC_DIR).unwrap();
    std::fs::remove_dir_all(root.at(".claude")).unwrap();
    link(&root, "../.claude/skills", ".cursor/skills");
    root.write(".speclink.yaml", "tools: [claude, cursor]\n");
    let before = snapshot(&root);

    let err = update(&root.dir, false).expect_err("懸空連結也必須被拒");

    assert_shared_dir_refusal(err, [".cursor/skills", ".claude/skills"], ["cursor", "claude"]);
    assert_eq!(snapshot(&root), before, "拒絕＝零寫入");
}

#[cfg(unix)]
#[test]
fn reconcile_refuses_a_dangling_link_into_a_deselected_tools_dir_without_writing() {
    // 只勾 cursor、連結指向還不存在的 .claude/skills：.speclink.yaml 也不得先被改寫。
    let root = TempRoot::new("shared-dir-dangling-deselected");
    init(&root.dir, &[Tool::Claude], false, DEFAULT_SPEC_DIR).unwrap();
    std::fs::remove_dir_all(root.at(".claude")).unwrap();
    link(&root, "../.claude/skills", ".cursor/skills");
    let before = snapshot(&root);

    let err = reconcile_builtin_tools(&root.dir, &[Tool::Cursor]).expect_err("必須被拒");

    assert_shared_dir_refusal(err, [".cursor/skills", ".claude/skills"], ["cursor", "claude"]);
    assert_eq!(snapshot(&root), before, "拒絕＝零寫入");
}

#[cfg(unix)]
#[test]
fn a_loop_of_links_does_not_hang_the_sync() {
    // 連結迴圈（a→b、b→a）：解析有層數上限，同步不會卡住；兩個目錄不是同一個工具的目錄，
    // 檢查放行，寫入時才由檔案系統回報錯誤。
    let root = TempRoot::new("shared-dir-loop");
    init(&root.dir, &[Tool::Claude], false, DEFAULT_SPEC_DIR).unwrap();
    link(&root, "b", ".cursor/a");
    link(&root, "a", ".cursor/b");
    link(&root, "a", ".cursor/skills");
    root.write(".speclink.yaml", "tools: [claude, cursor]\n");

    assert!(update(&root.dir, false).is_err(), "寫進連結迴圈必須失敗，而不是卡住");
}

// --- 渲染的軸：規格在不在這個 checkout（spec workspace-tools「技能依規格是否在 checkout 渲染」） ---

/// `skills_dir` 下的受管技能檔逐份等於以這個軸渲染的內容（worktree 政策關閉的集合）。
fn assert_skills_rendered_as(
    root: &TempRoot,
    target: skills::RenderTarget,
    skills_dir: &str,
    location: SpecLocation,
) {
    // 兩個軸渲染出來的內容不同，下面的比對才分得出寫進去的是哪一個。
    assert_ne!(
        managed_skills(target, false, DEFAULT_SPEC_DIR, SpecLocation::InCheckout),
        managed_skills(target, false, DEFAULT_SPEC_DIR, SpecLocation::Elsewhere),
        "兩個軸的受管集合不該相同"
    );
    for (dir, expected) in managed_skills(target, false, DEFAULT_SPEC_DIR, location) {
        assert_eq!(
            eol_normalized(&root.read(&format!("{skills_dir}/{dir}/SKILL.md"))),
            eol_normalized(&expected),
            "{skills_dir}/{dir}（{location:?}）"
        );
    }
}

const CLAUDE_TARGET: skills::RenderTarget = skills::RenderTarget::Builtin(Tool::Claude);

#[test]
fn init_renders_skills_with_the_spec_in_the_checkout() {
    let root = TempRoot::new("axis-init-local");
    init(&root.dir, &[Tool::Claude], false, DEFAULT_SPEC_DIR).unwrap();
    assert_skills_rendered_as(&root, CLAUDE_TARGET, ".claude/skills", SpecLocation::InCheckout);
}

#[test]
fn init_remote_renders_skills_with_the_spec_not_in_the_checkout() {
    let root = TempRoot::new("axis-init-remote");
    init_remote(&root.dir, &[Tool::Claude], false, REMOTE_URL, None).unwrap();
    assert_skills_rendered_as(&root, CLAUDE_TARGET, ".claude/skills", SpecLocation::Elsewhere);
}

/// `update` 與各同步函式看 `.speclink.yaml` 的 remote 區段，不看規格資料夾在不在磁碟上：
/// 本機專案是「在」；remote 專案留著舊的規格資料夾，仍然是「不在」
/// （Scenario「殘留的規格資料夾不影響判斷」）。
#[test]
fn update_renders_skills_by_the_remote_section_not_by_the_spec_folder() {
    let local = TempRoot::new("axis-update-local");
    init(&local.dir, &[Tool::Claude], false, DEFAULT_SPEC_DIR).unwrap();
    update(&local.dir, false).unwrap();
    assert_skills_rendered_as(&local, CLAUDE_TARGET, ".claude/skills", SpecLocation::InCheckout);

    let remote = TempRoot::new("axis-update-remote");
    remote.write("speclink/changes/leftover/proposal.md", "## Why\n");
    seed_remote_workspace(&remote, &[Tool::Claude]);
    let custom = custom_target();
    let assert_not_in_checkout = |entry: &str| {
        assert!(remote.exists("speclink/changes/leftover/proposal.md"), "{entry}：殘留的資料夾還在");
        assert_skills_rendered_as(&remote, CLAUDE_TARGET, ".claude/skills", SpecLocation::Elsewhere);
        assert_skills_rendered_as(&remote, skills::RenderTarget::Custom(&custom), ".wad/skills", SpecLocation::Elsewhere);
    };
    assert_not_in_checkout("update");
    sync_skills(&remote.dir).unwrap();
    assert_not_in_checkout("sync_skills");
    reconcile_builtin_tools(&remote.dir, &[Tool::Claude]).unwrap();
    assert_not_in_checkout("reconcile_builtin_tools");
}

/// Scenario「探測用相同的基準」：remote 專案剛更新後是現版；之後只有一份技能檔變舊時，
/// 被列為不同的就只有那一份——比對基準和 `update` 用同一個軸。
#[test]
fn probing_skills_after_a_remote_update_uses_the_same_base() {
    let root = TempRoot::new("axis-probe-remote");
    seed_remote_workspace(&root, &[Tool::Claude]);

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Current);
    assert_eq!(probe.differing_files, Vec::<String>::new());

    let aged = ".claude/skills/speclink-commit/SKILL.md";
    root.write(aged, &root.read(aged).replace(ASSET_VERSION, "v0.0.1"));
    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Stale);
    assert_eq!(probe.differing_files, vec![aged.to_string()]);
}
