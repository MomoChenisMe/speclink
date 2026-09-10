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
/// 第二份描述子夾具：規格「技能檔過期探測」Example 的字面值（name cursor、
/// skills_dir .cursor/skills），且不帶已棄用的 instructions_file——探測測試要的是
/// 乾淨的描述子，不是剝除路徑。
const CURSOR_DESCRIPTOR: &str = "  - name: cursor\n    skills_dir: .cursor/skills\n";
const REMOTE_URL: &str = "https://team.example.test/api/speclink/v1/projects/acme";
const CLAUDE_USER_TEXT: &str = "使用者寫在 CLAUDE.md 的段落";
const CODEX_USER_TEXT: &str = "使用者寫在 AGENTS.md 的段落";

fn instructions_file(tool: Tool) -> &'static str {
    instructions_path(tool)
}

fn user_text(tool: Tool) -> &'static str {
    match tool {
        Tool::Claude => CLAUDE_USER_TEXT,
        Tool::Codex => CODEX_USER_TEXT,
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
    fn walk(dir: &Path, prefix: &str, out: &mut Vec<(String, Vec<u8>)>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            let rel = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };
            if entry.path().is_dir() {
                out.push((format!("{rel}/"), Vec::new()));
                walk(&entry.path(), &rel, out);
            } else {
                out.push((rel, std::fs::read(entry.path()).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(&root.dir, "", &mut out);
    out
}

/// Spec example「built-in 選集轉換」逐列：轉換後 built-in 集合等於請求，
/// custom descriptor／未知頂層鍵／remote 原值保留，marker 不重複，使用者文字不動。
#[test]
fn reconcile_converts_builtin_selection_row_by_row() {
    let rows: [(&[Tool], &[Tool]); 3] = [
        (&[Tool::Claude], &[Tool::Codex]),
        (&[Tool::Claude, Tool::Codex], &[Tool::Claude]),
        (&[Tool::Codex], &[Tool::Claude, Tool::Codex]),
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

        for tool in [Tool::Claude, Tool::Codex] {
            let md = instructions_file(tool);
            let text = root.read(md);
            let skill = propose_skill(tool);
            if to.contains(&tool) {
                assert!(root.exists(&skill), "row {i}: {skill} 應被補齊");
            } else {
                assert!(!root.exists(&skill), "row {i}: {skill} 應被清理");
            }
            // 指令檔已退出受管：不論選取與否都只剩使用者自己的內容。
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
// Spec requirement:「工作區補齊入口」（desktop-enable-speclink-prompt）——
// 冪等補骨架缺件、寫 tools、生成受管檔，既有 openspec/ 內容零觸碰。

const CUSTOM_WORKFLOW_CONFIG: &str =
    "schema: spec-driven\nlocale: tw\nrules:\n  proposal:\n    - 保持提案精簡\n";

/// 未啟用目錄：openspec/ 內有規格文件、變更、討論與自訂 config.yaml，
/// 但專案根無 .speclink.yaml。
fn seed_unadopted(root: &TempRoot) {
    root.write("openspec/specs/auth/spec.md", "## Purpose\n既有規格文件。\n");
    root.write("openspec/changes/add-auth/proposal.md", "## Why\n既有變更。\n");
    root.write("openspec/discussions/auth-scope.md", "# Discussion\n既有討論。\n");
    root.write("openspec/config.yaml", CUSTOM_WORKFLOW_CONFIG);
}

/// Spec Scenario「補齊工作區檔且既有內容零觸碰」：工作區檔補齊，
/// 既有 openspec/ 文件（含自訂 config.yaml）位元級不變。
#[test]
fn adopt_fills_workspace_files_with_existing_content_untouched() {
    let root = TempRoot::new("adopt-fill");
    seed_unadopted(&root);
    let before = snapshot(&root);

    adopt(&root.dir, &[Tool::Claude]).expect("adopt succeeds");

    let app = root.read(".speclink.yaml");
    assert!(app.contains("claude"), "tools 須記錄 claude：{app}");
    assert!(!root.exists("CLAUDE.md"), "工作區補齊不得產生指令檔");
    assert!(root.exists(".claude/skills/speclink-propose/SKILL.md"));

    let after = snapshot(&root);
    for entry in before.iter().filter(|(rel, _)| !rel.ends_with('/')) {
        assert!(after.contains(entry), "既有文件必須位元級不變：{}", entry.0);
    }
    assert_eq!(root.read("openspec/config.yaml"), CUSTOM_WORKFLOW_CONFIG);
}

/// Spec Scenario「骨架缺件補齊」：缺 specs/ 與 config.yaml 時補齊目錄與範本。
#[test]
fn adopt_backfills_missing_skeleton() {
    let root = TempRoot::new("adopt-skeleton");
    root.write("openspec/changes/add-auth/proposal.md", "## Why\n既有變更。\n");

    adopt(&root.dir, &[Tool::Claude]).expect("adopt succeeds");

    assert!(root.at("openspec/specs").is_dir());
    assert!(root.at("openspec/changes/archive").is_dir());
    assert_eq!(root.read("openspec/config.yaml"), WORKFLOW_CONFIG_TEMPLATE);
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
        managed_skills(skills::RenderTarget::Builtin(tool), false, "openspec")
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
    root.write(".speclink.yaml", &format!("tools:\n  - claude\n{CURSOR_DESCRIPTOR}"));
    update(&root.dir, false).expect("先把兩邊的受管檔生成到現版");
    root
}

#[test]
fn probe_reports_missing_for_a_descriptor_without_skills() {
    // Scenario「描述子技能檔缺失判缺失」：描述子的技能檔與內建技能檔同為受管檔，
    // 整組不在即缺失，差異清單以描述子的 skills_dir 起頭。
    let root = workspace_with_descriptor("probe-descriptor-missing");
    std::fs::remove_dir_all(root.at(".cursor/skills")).unwrap();

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Missing, "{probe:?}");
    let cursor = probe
        .tools
        .iter()
        .find(|t| t.tool == "cursor")
        .expect("逐工具資訊須含描述子的 name");
    assert!(cursor.missing, "{cursor:?}");
    assert_eq!(cursor.workspace_version, None, "{cursor:?}");
    let claude = probe.tools.iter().find(|t| t.tool == "claude").unwrap();
    assert!(!claude.missing && !claude.stale && !claude.newer, "{claude:?}");
    assert!(
        !probe.differing_files.is_empty(),
        "缺失須列出描述子的受管檔"
    );
    for path in &probe.differing_files {
        assert!(
            path.starts_with(".cursor/skills/speclink-") && path.ends_with("/SKILL.md"),
            "差異清單只該含描述子技能檔：{path}"
        );
    }
    // for_codex 子集＋worktree 政策關閉：兩顆 worktree 技能不在預期生成集合內。
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
        ".cursor/skills/speclink-propose/SKILL.md",
    ] {
        root.write(skill, &root.read(skill).replace(ASSET_VERSION, "v0.9.0"));
    }

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Stale, "{probe:?}");
    let cursor = probe.tools.iter().find(|t| t.tool == "cursor").unwrap();
    assert!(cursor.stale && !cursor.newer && !cursor.missing, "{cursor:?}");
    for expected in [
        ".claude/skills/speclink-propose/SKILL.md",
        ".cursor/skills/speclink-propose/SKILL.md",
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
        &format!("tools:\n  - claude\n  - name: broken-harness\n{CURSOR_DESCRIPTOR}"),
    );
    std::fs::remove_dir_all(root.at(".cursor/skills")).unwrap();

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Missing, "{probe:?}");
    let cursor = probe.tools.iter().find(|t| t.tool == "cursor").expect("cursor 仍在判定面");
    assert!(cursor.missing, "{cursor:?}");
}

#[test]
fn probe_normalizes_a_descriptor_skills_dir_with_a_trailing_slash() {
    // 描述子的 skills_dir 是使用者手寫的字串：結尾斜線不得在差異清單裡變成 `//`。
    let root = TempRoot::new("probe-descriptor-trailing-slash");
    init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
    root.write(
        ".speclink.yaml",
        "tools:\n  - claude\n  - name: cursor\n    skills_dir: .cursor/skills/\n",
    );
    update(&root.dir, false).expect("結尾斜線的 skills_dir 合法，先生成到現版");
    std::fs::remove_dir_all(root.at(".cursor/skills")).unwrap();

    let probe = probe_assets(&root.dir);
    assert_eq!(probe.status, AssetStatus::Missing, "{probe:?}");
    assert!(!probe.differing_files.is_empty());
    for path in &probe.differing_files {
        assert!(
            path.starts_with(".cursor/skills/speclink-") && !path.contains("//"),
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
    root.write(".speclink.yaml", "tools:\n  - claude\n  - name: cursor\n");

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
    root.write(
        ".speclink.yaml",
        &format!("tools:\n  - claude\n  - codex\n{CUSTOM_DESCRIPTOR}"),
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
    let sel = ToolSelection::resolve(&root.dir, &app_config("tools: [claude, cursor]\n"));
    assert_eq!(sel.builtins, vec![Tool::Claude]);
    assert_eq!(
        sel.notes,
        vec![
            "unknown tool 'cursor' in .speclink.yaml tools list (supported: claude, codex)"
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

/// registry 依 `codex_subset` 過濾後的 `speclink-<name>` 目錄名（順序同 registry）。
fn registry_dirs(codex_subset: bool) -> Vec<String> {
    skills::registry()
        .iter()
        .filter(|s| !codex_subset || s.for_codex)
        .map(|s| format!("speclink-{}", s.name))
        .collect()
}

/// 受管技能集合只有一個擁有者：Claude 目標拿 registry 全集，非 Claude 目標拿
/// `for_codex` 子集，每一筆內容逐字等於同參數的 render。
#[test]
fn managed_skills_covers_the_registry_per_target() {
    let custom = custom_target();
    let claude = managed_skills(skills::RenderTarget::Builtin(Tool::Claude), true, "openspec");
    let codex = managed_skills(skills::RenderTarget::Builtin(Tool::Codex), true, "openspec");
    let neutral = managed_skills(skills::RenderTarget::Custom(&custom), true, "openspec");

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
                &skills::render_skill_file_for(target, skill, "openspec"),
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
        managed_skills(skills::RenderTarget::Builtin(Tool::Claude), false, "openspec"),
        managed_skills(skills::RenderTarget::Builtin(Tool::Codex), false, "openspec"),
        managed_skills(skills::RenderTarget::Custom(&custom), false, "openspec"),
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
        SyncPlan::resolve(&root.dir, ToolSelection::builtins_only(&[Tool::Codex]), "openspec");
    assert_eq!(plan.targets.len(), 1, "只選 codex 就只有一個 target");
    assert_eq!(plan.targets[0].label, "codex");
    assert_eq!(plan.targets[0].skills_root, root.at(".agents/skills"));
    assert_eq!(plan.deselected_builtins, vec![Tool::Claude]);
}

/// legacy 回退（沒有 tools 清單）不下架任何內建工具。
#[test]
fn sync_plan_prunes_nothing_on_the_legacy_fallback() {
    let root = TempRoot::new("sync-plan-legacy");
    std::fs::create_dir_all(root.at(".claude")).unwrap();
    let selection = ToolSelection::resolve(&root.dir, &app_config("tools: []\n"));
    let plan = SyncPlan::resolve(&root.dir, selection, "openspec");
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
    let plan = SyncPlan::resolve(&root.dir, selection, "openspec");
    assert_eq!(plan.targets.len(), 2);
    assert_eq!(plan.targets[0].label, "claude");
    assert_eq!(plan.targets[1].label, "wad-harness");
    assert_eq!(plan.targets[1].skills_root, root.at(".wad/skills"));
    assert_eq!(plan.deselected_builtins, vec![Tool::Codex]);
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
    let plan = SyncPlan::resolve(&root.dir, ToolSelection::resolve(&root.dir, &app), "openspec");
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
