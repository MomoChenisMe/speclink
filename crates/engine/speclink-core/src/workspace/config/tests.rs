
#[test]
fn set_schema_overwrites_the_existing_key_and_preserves_every_other_byte() {
    let doc = "# lead comment\nschema: spec-driven\n\nlocale: tw\ncontext: |\n  line one\n";
    let out = super::set_workflow_schema_text(Some(doc), "my-flow").expect("rewrites");
    assert_eq!(out, "# lead comment\nschema: my-flow\n\nlocale: tw\ncontext: |\n  line one\n");
}

#[test]
fn set_schema_refuses_an_unparseable_document() {
    assert!(super::set_workflow_schema_text(Some(": not yaml : [\n"), "my-flow").is_err());
}
use super::*;

fn app(yaml: &str) -> AppConfig {
    serde_yaml::from_str(yaml).expect("app yaml")
}

fn wf(yaml: &str) -> WorkflowConfig {
    serde_yaml::from_str(yaml).expect("wf yaml")
}

fn env_of(pairs: &[(&str, &str)]) -> EnvOverrides {
    EnvOverrides::from_lookup(|key| {
        pairs
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.to_string())
    })
}

const NO_ENV: &[(&str, &str)] = &[];

// --- 注入形 lookup 是唯一 env 來源：process env 不可見 ---

#[test]
fn from_lookup_never_reads_the_process_environment() {
    std::env::set_var("SPECLINK_AUDIT", "true");
    let env = EnvOverrides::from_lookup(|_| None);
    std::env::remove_var("SPECLINK_AUDIT");
    assert_eq!(
        env.audit, None,
        "an empty injected lookup yields no overrides regardless of process env"
    );
}

// --- locale: env > config.yaml > default ---

#[test]
fn locale_env_var_wins_over_all_layers() {
    let p = resolve_policy(&env_of(&[("SPECLINK_LOCALE", "ja")]), &wf("locale: en"));
    assert_eq!(p.locale, "Japanese (日本語)");
}

#[test]
fn locale_canonical_value_applies_without_upper_layers() {
    let p = resolve_policy(&env_of(NO_ENV), &wf("locale: ja"));
    assert_eq!(p.locale, "Japanese (日本語)");
}

#[test]
fn locale_defaults_to_english() {
    let p = resolve_policy(&env_of(NO_ENV), &wf("{}"));
    assert_eq!(p.locale, "English");
}

// --- spec_locale: env > config.yaml > default ---

#[test]
fn spec_locale_env_var_wins_over_all_layers() {
    let p = resolve_policy(&env_of(&[("SPECLINK_SPEC_LOCALE", "ja")]), &wf("spec_locale: en"));
    assert_eq!(p.spec_locale.as_deref(), Some("ja"));
}

#[test]
fn spec_locale_canonical_value_applies_without_upper_layers() {
    let p = resolve_policy(&env_of(NO_ENV), &wf("spec_locale: ja"));
    assert_eq!(p.spec_locale.as_deref(), Some("ja"));
}

#[test]
fn spec_locale_defaults_to_none() {
    let p = resolve_policy(&env_of(NO_ENV), &wf("{}"));
    assert_eq!(p.spec_locale, None);
}

#[test]
fn spec_locale_auto_follows_resolved_locale() {
    // Existing "auto" semantics survive the extra env layer: auto follows the
    // locale resolved through the same three layers.
    let p = resolve_policy(&env_of(NO_ENV), &wf("locale: ja\nspec_locale: auto"));
    assert_eq!(p.spec_locale.as_deref(), Some("ja"));
    let p = resolve_policy(
        &env_of(&[("SPECLINK_LOCALE", "tw")]),
        &wf("locale: ja\nspec_locale: auto"),
    );
    assert_eq!(p.spec_locale.as_deref(), Some("tw"));
}

// --- tdd: env > config.yaml > default ---

#[test]
fn tdd_env_var_wins_over_all_layers() {
    // Spec scenario 環境變數覆寫正典值: SPECLINK_TDD=false beats both files' true.
    let p = resolve_policy(&env_of(&[("SPECLINK_TDD", "false")]), &wf("tdd: true"));
    assert!(!p.tdd);
}

#[test]
fn tdd_canonical_value_applies_without_upper_layers() {
    // Spec scenario 正典值生效: only config.yaml sets tdd: true.
    let p = resolve_policy(&env_of(NO_ENV), &wf("tdd: true"));
    assert!(p.tdd);
}

#[test]
fn tdd_defaults_to_false() {
    let p = resolve_policy(&env_of(NO_ENV), &wf("{}"));
    assert!(!p.tdd);
}

// --- audit: env > config.yaml > default ---

#[test]
fn audit_env_var_wins_over_all_layers() {
    let p = resolve_policy(&env_of(&[("SPECLINK_AUDIT", "false")]), &wf("audit: true"));
    assert!(!p.audit);
}

#[test]
fn audit_canonical_value_applies_without_upper_layers() {
    let p = resolve_policy(&env_of(NO_ENV), &wf("audit: true"));
    assert!(p.audit);
}

#[test]
fn audit_defaults_to_false() {
    let p = resolve_policy(&env_of(NO_ENV), &wf("{}"));
    assert!(!p.audit);
}

// --- worktree: env > config.yaml > default ---

#[test]
fn worktree_canonical_value_applies_without_upper_layers() {
    // Spec scenario worktree 欄位寫入與呈現 (read side): config.yaml alone turns it on.
    let p = resolve_policy(&env_of(NO_ENV), &wf("worktree: true"));
    assert!(p.worktree);
}

#[test]
fn worktree_env_var_wins_over_canonical() {
    // Spec scenario SPECLINK_WORKTREE 覆寫檔案值.
    let p = resolve_policy(
        &env_of(&[("SPECLINK_WORKTREE", "true")]),
        &wf("worktree: false"),
    );
    assert!(p.worktree);
    let p = resolve_policy(
        &env_of(&[("SPECLINK_WORKTREE", "false")]),
        &wf("worktree: true"),
    );
    assert!(!p.worktree);
}

#[test]
fn worktree_invalid_env_var_falls_to_next_layer() {
    let p = resolve_policy(
        &env_of(&[("SPECLINK_WORKTREE", "yes")]),
        &wf("worktree: true"),
    );
    assert!(p.worktree);
}

#[test]
fn worktree_defaults_to_false() {
    let p = resolve_policy(&env_of(NO_ENV), &wf("{}"));
    assert!(!p.worktree);
}

// --- boolean env vars: only true/false are accepted; anything else is unset ---

#[test]
fn invalid_bool_env_var_falls_to_next_layer() {
    // Spec scenario 非法布林環境變數落到下一層: SPECLINK_AUDIT=yes is ignored.
    let p = resolve_policy(
        &env_of(&[("SPECLINK_AUDIT", "yes")]),
        &wf("audit: true"),
    );
    assert!(p.audit);
    // Numeric truthiness is NOT a boolean here — "1"/"0" are unset too.
    let p = resolve_policy(
        &env_of(&[("SPECLINK_TDD", "1")]),
        &wf("tdd: false"),
    );
    assert!(!p.tdd);
    // An invalid env value falls to the canonical layer — the app key stays inert.
    let p = resolve_policy(
        &env_of(&[("SPECLINK_TDD", "yes")]),
        &wf("tdd: true"),
    );
    assert!(p.tdd);
}

#[test]
fn bool_env_var_is_case_insensitive_and_trimmed() {
    // A CI system exporting SPECLINK_TDD=TRUE must not silently disable the
    // override (confused-developer trap): case and surrounding whitespace are
    // normalized before matching.
    let p = resolve_policy(
        &env_of(&[("SPECLINK_TDD", " TRUE ")]),
        &wf("tdd: false"),
    );
    assert!(p.tdd);
}

#[test]
fn empty_string_env_vars_are_unset() {
    // `SPECLINK_LOCALE=` (empty) must not shadow lower layers with emptiness.
    let p = resolve_policy(
        &env_of(&[
            ("SPECLINK_LOCALE", ""),
            ("SPECLINK_SPEC_LOCALE", "  "),
            ("SPECLINK_TDD", ""),
        ]),
        &wf("locale: tw\nspec_locale: tw\ntdd: true"),
    );
    assert_eq!(p.locale, "Traditional Chinese (繁體中文)");
    assert_eq!(p.spec_locale.as_deref(), Some("tw"));
    assert!(p.tdd);
}

// --- serde compatibility ---

#[test]
fn workflow_config_parses_new_policy_fields() {
    let w = wf("tdd: true\naudit: false");
    assert_eq!(w.tdd, Some(true));
    assert_eq!(w.audit, Some(false));
}

#[test]
fn workflow_config_without_policy_fields_still_parses() {
    let w = WorkflowConfig::from_text(Some("schema: spec-driven\nlocale: tw")).expect("parses");
    assert_eq!(w.tdd, None);
    assert_eq!(w.audit, None);
    assert_eq!(w.worktree, None);
    assert_eq!(w.locale.as_deref(), Some("tw"));
}

#[test]
fn workflow_config_parses_worktree() {
    assert_eq!(wf("worktree: true").worktree, Some(true));
    assert_eq!(wf("worktree: false").worktree, Some(false));
}

// --- fail-closed loading: a present file must parse; only a MISSING file yields defaults ---

/// Throwaway dir for load() tests, removed on drop.
struct TempCfgDir {
    dir: std::path::PathBuf,
}

impl TempCfgDir {
    fn new(tag: &str) -> TempCfgDir {
        let dir = std::env::temp_dir().join(format!(
            "speclink-core-cfg-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        TempCfgDir { dir }
    }

    fn app_yaml(&self, content: &str) -> std::path::PathBuf {
        let path = self.dir.join(".speclink.yaml");
        std::fs::write(&path, content).unwrap();
        path
    }

    fn read_app_yaml(&self) -> String {
        std::fs::read_to_string(self.dir.join(".speclink.yaml")).unwrap()
    }
}

impl Drop for TempCfgDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn app_config_load_missing_file_gives_defaults() {
    let t = TempCfgDir::new("missing");
    let cfg = AppConfig::load(&t.dir.join(".speclink.yaml")).expect("missing file → defaults");
    assert!(cfg.spec_dir.is_none());
    assert!(cfg.remote.is_none());
    assert!(cfg.tools.is_empty());
}

#[test]
fn app_config_load_empty_or_comment_only_file_gives_defaults() {
    // A template-fresh or commented-out file is a NULL document — that is valid
    // YAML, not a parse failure. Fail-closed must not break it.
    for content in ["", "\n\n", "# Speclink application config\n# tools:\n"] {
        let t = TempCfgDir::new("empty");
        let cfg = AppConfig::load(&t.app_yaml(content))
            .unwrap_or_else(|e| panic!("{content:?} must give defaults, got: {e}"));
        assert!(cfg.remote.is_none(), "for {content:?}");
    }
}

#[test]
fn app_config_load_bad_yaml_is_a_config_error() {
    // P0 fail-closed: a file that EXISTS but cannot parse is an error carrying
    // the workspace-relative path and the parser's reason — never defaults.
    for bad in ["remote: [unclosed", ": not yaml : [", "remote: 42", "tools: notalist"] {
        let t = TempCfgDir::new("bad");
        let err = AppConfig::load(&t.app_yaml(bad))
            .expect_err(&format!("{bad:?} must be a config error"));
        assert_eq!(err.file, ".speclink.yaml", "for {bad:?}");
        assert!(!err.reason.is_empty(), "reason must not be empty for {bad:?}");
        let msg = err.to_string();
        assert!(msg.contains(".speclink.yaml"), "display names the file: {msg}");
        assert!(msg.contains(&err.reason), "display carries the reason: {msg}");
    }
}

#[test]
fn app_config_load_valid_file_behavior_unchanged() {
    // Successfully parsing files keep their exact behavior (unknown keys tolerated).
    let t = TempCfgDir::new("valid");
    let path = t.app_yaml("spec_dir: docs/specs\nfuture_key: ignored\ntools:\n  - claude\n");
    let cfg = AppConfig::load(&path).expect("valid file parses");
    assert_eq!(cfg.spec_dir.as_deref(), Some("docs/specs"));
    assert_eq!(cfg.tools.len(), 1);
}

#[test]
fn workflow_config_from_text_missing_gives_defaults() {
    let w = WorkflowConfig::from_text(None).expect("missing document → defaults");
    assert_eq!(w.schema, None);
    assert_eq!(w.tdd, None);
}

#[test]
fn workflow_config_from_text_empty_or_comment_only_gives_defaults() {
    for text in ["", "\n", "# workflow config\n# tdd: true\n"] {
        let w = WorkflowConfig::from_text(Some(text))
            .unwrap_or_else(|e| panic!("{text:?} must give defaults, got: {e}"));
        assert_eq!(w.tdd, None, "for {text:?}");
    }
}

#[test]
fn workflow_config_from_text_bad_yaml_is_a_config_error() {
    for bad in ["rules: [unclosed", "tdd: [true]", ": not yaml : ["] {
        let err = WorkflowConfig::from_text(Some(bad))
            .expect_err(&format!("{bad:?} must be a config error"));
        assert_eq!(err.file, "openspec/config.yaml", "for {bad:?}");
        assert!(!err.reason.is_empty(), "reason must not be empty for {bad:?}");
        let msg = err.to_string();
        assert!(msg.contains("openspec/config.yaml"), "display names the file: {msg}");
    }
}

// --- tools: dual-form entries (builtin name string | descriptor object) ---

#[test]
fn tools_list_parses_builtin_strings_and_descriptors() {
    let a = app(
        "tools:\n  - claude\n  - name: wad-harness\n    skills_dir: .wad/skills\n    instructions_file: WAD.md\n    invocation: tool-call\n",
    );
    assert_eq!(a.tools.len(), 2);
    match &a.tools[0] {
        ToolEntry::Builtin(s) => assert_eq!(s, "claude"),
        other => panic!("expected builtin string entry, got {other:?}"),
    }
    match &a.tools[1] {
        ToolEntry::Descriptor(d) => {
            assert_eq!(d.name.as_deref(), Some("wad-harness"));
            assert_eq!(d.skills_dir.as_deref(), Some(".wad/skills"));
            assert_eq!(d.instructions_file.as_deref(), Some("WAD.md"));
            assert_eq!(d.invocation.as_deref(), Some("tool-call"));
        }
        other => panic!("expected descriptor entry, got {other:?}"),
    }
}

#[test]
fn tools_list_of_plain_strings_still_parses() {
    let a = app("tools:\n  - claude\n  - codex\n");
    assert_eq!(a.tools.len(), 2);
    assert!(matches!(&a.tools[0], ToolEntry::Builtin(s) if s == "claude"));
    assert!(matches!(&a.tools[1], ToolEntry::Builtin(s) if s == "codex"));
}

// --- descriptor validation: single-line semantic errors naming the field ---

fn descriptor(name: &str, skills_dir: &str, instructions_file: &str) -> ToolDescriptor {
    ToolDescriptor {
        name: Some(name.to_string()),
        skills_dir: Some(skills_dir.to_string()),
        instructions_file: Some(instructions_file.to_string()),
        invocation: None,
    }
}

#[test]
fn descriptor_validation_accepts_valid_input_and_defaults_invocation_to_cli() {
    let v = descriptor("wad-harness", ".wad/skills", "WAD.md")
        .validate()
        .expect("valid descriptor");
    assert_eq!(v.name, "wad-harness");
    assert_eq!(v.invocation, Invocation::Cli);
    let mut d = descriptor("wad-harness", ".wad/skills", "WAD.md");
    d.invocation = Some("tool-call".to_string());
    assert_eq!(d.validate().unwrap().invocation, Invocation::ToolCall);
}

#[test]
fn descriptor_validation_rejects_builtin_name_conflict() {
    let err = descriptor("claude", ".wad/skills", "WAD.md").validate().unwrap_err();
    assert!(err.contains("name"), "must name the field: {err}");
    assert!(!err.contains('\n'), "single line: {err:?}");
}

#[test]
fn descriptor_validation_rejects_non_kebab_case_names() {
    for bad in ["Wad-Harness", "wad_harness", "-wad", "wad-", "w", &"x".repeat(51)] {
        let err = descriptor(bad, ".wad/skills", "WAD.md").validate().unwrap_err();
        assert!(err.contains("name"), "must name the field for {bad:?}: {err}");
        assert!(!err.contains('\n'), "single line: {err:?}");
    }
}

#[test]
fn descriptor_validation_rejects_paths_escaping_project_root() {
    let err = descriptor("wad-harness", "../outside/skills", "WAD.md").validate().unwrap_err();
    assert!(err.contains("skills_dir"), "must name the field: {err}");
    assert!(!err.contains('\n'), "single line: {err:?}");
    let err = descriptor("wad-harness", ".wad/skills", "../WAD.md").validate().unwrap_err();
    assert!(err.contains("instructions_file"), "must name the field: {err}");
    // Lexical dot-dot tricks and absolute paths are escapes too.
    assert!(descriptor("wad-harness", ".wad/../../x", "WAD.md").validate().is_err());
    assert!(descriptor("wad-harness", "/abs/skills", "WAD.md").validate().is_err());
    // A drive-letter path is absolute only on Windows; on unix the same string is a
    // legal (if odd) relative dir name — the containment check is lexical host-platform
    // semantics, so the rejection can only be asserted where it actually applies.
    if cfg!(windows) {
        assert!(descriptor("wad-harness", "C:\\abs\\skills", "WAD.md").validate().is_err());
    }
}

#[test]
fn descriptor_validation_normalizes_a_trailing_slash() {
    // skills_dir 是使用者手寫的字串：結尾分隔符在邊界一次削掉，下游（生成、
    // 足跡記錄、過期探測的路徑回報）就只看得到同一種形式。
    let v = descriptor("wad-harness", ".wad/skills/", "WAD.md").validate().unwrap();
    assert_eq!(v.skills_dir, ".wad/skills");
    let v = descriptor("wad-harness", ".wad/skills///", "WAD.md").validate().unwrap();
    assert_eq!(v.skills_dir, ".wad/skills", "連續分隔符一併削去");
}

#[test]
fn descriptor_validation_rejects_a_skills_dir_that_normalizes_to_nothing() {
    // 只由分隔符構成的 skills_dir 削完就是專案根本身——生成物會散進專案根、
    // 清理會掃到根目錄；在邊界擋掉，下游不必再驗一次。
    for bad in ["/", "///", "./", ".", ".wad/.."] {
        let err = descriptor("wad-harness", bad, "WAD.md").validate().unwrap_err();
        assert!(err.contains("skills_dir"), "must name the field for {bad:?}: {err}");
        assert!(!err.contains('\n'), "single line: {err:?}");
    }
}

#[test]
fn descriptor_validation_accepts_a_directory_next_to_a_builtin_one() {
    // Example 表的接受列：只有正規化後「等同」內建目錄才拒絕，相鄰的名字不受影響。
    for ok in [".claude/skills-extra", ".claudex/skills", ".agents/skills2"] {
        let v = descriptor("wad-harness", ok, "WAD.md").validate();
        assert!(v.is_ok(), "{ok} 應被接受：{v:?}");
    }
}

#[test]
fn descriptor_validation_rejects_a_builtin_skills_dir() {
    // 描述子指到內建工具的 skills 目錄：兩個 target 會對同一份 SKILL.md 各比一次
    // 期望內容（探測永遠回報過期），且生成時後手的 for_codex 子集會把前手剛寫的
    // claude 專屬技能當成孤兒刪掉。名稱衝突已擋，目錄衝突同理。
    // 等價拼法與字面拼法一樣被擋：比對走路徑正規化，不是字串相等。
    for bad in [
        ".claude/skills",
        ".agents/skills",
        ".claude/skills/",
        ".claude//skills",
        ".claude/skills/.",
        "./.claude/skills",
        ".claude/./skills",
        ".wad/../.claude/skills",
    ] {
        let err = descriptor("wad-harness", bad, "WAD.md").validate().unwrap_err();
        assert!(err.contains("skills_dir"), "must name the field for {bad:?}: {err}");
        assert!(!err.contains('\n'), "single line: {err:?}");
    }
}

#[test]
fn descriptor_validation_rejects_unknown_invocation() {
    let mut d = descriptor("wad-harness", ".wad/skills", "WAD.md");
    d.invocation = Some("http".to_string());
    let err = d.validate().unwrap_err();
    assert!(err.contains("invocation"), "must name the field: {err}");
    assert!(!err.contains('\n'), "single line: {err:?}");
}

#[test]
fn descriptor_validation_rejects_missing_required_fields() {
    let d = ToolDescriptor {
        name: Some("wad-harness".to_string()),
        skills_dir: None,
        instructions_file: Some("WAD.md".to_string()),
        invocation: None,
    };
    let err = d.validate().unwrap_err();
    assert!(err.contains("skills_dir"), "must name the missing field: {err}");
}

// --- remote section: optional connection settings (url / repo) ---

#[test]
fn remote_section_with_url_and_repo_parses_both_fields() {
    let a = app("remote:\n  url: https://team.example.com/speclink/projects/foo\n  repo: backend\n");
    let r = a.remote.as_ref().expect("remote section present");
    assert_eq!(
        r.url.as_deref(),
        Some("https://team.example.com/speclink/projects/foo")
    );
    assert_eq!(r.repo.as_deref(), Some("backend"));
}

#[test]
fn remote_section_with_only_repo_leaves_url_absent() {
    // Committed files may omit url (supplied at runtime via SPECLINK_STORE_URL).
    let a = app("remote:\n  repo: backend\n");
    let r = a.remote.as_ref().expect("remote section present");
    assert_eq!(r.url, None);
    assert_eq!(r.repo.as_deref(), Some("backend"));
}

#[test]
fn empty_remote_section_is_present_with_absent_fields() {
    // Both `remote: {}` and a bare `remote:` key mean "section present, fields
    // empty" — a bare key must not silently read as fs mode (the mode signal is
    // key presence, and missing url must fail loudly downstream, not vanish here).
    for yaml in ["remote: {}\n", "remote:\n"] {
        let a = app(yaml);
        let r = a
            .remote
            .as_ref()
            .unwrap_or_else(|| panic!("remote section present for {yaml:?}"));
        assert_eq!(r.url, None);
        assert_eq!(r.repo, None);
    }
}

#[test]
fn config_without_remote_key_parses_with_section_absent() {
    // Backward compatibility: existing .speclink.yaml files without a remote key
    // keep parsing, other fields intact, and the section reads as absent.
    let a = app("tools:\n  - claude\n  - codex\n");
    assert!(a.remote.is_none());
    assert_eq!(a.tools.len(), 2);
}

// --- existing two-layer resolvers keep their observable behavior ---

#[test]
fn resolve_locale_reads_the_canonical_config_only() {
    assert_eq!(resolve_locale(&wf("locale: ja")), "Japanese (日本語)");
    assert_eq!(resolve_locale(&wf("{}")), "English");
}

#[test]
fn resolve_spec_locale_keeps_auto_and_english_normalization() {
    assert_eq!(
        resolve_spec_locale(&wf("spec_locale: auto\nlocale: tw")).as_deref(),
        Some("tw")
    );
    assert_eq!(resolve_spec_locale(&wf("spec_locale: en")), None);
    assert_eq!(resolve_spec_locale(&wf("{}")), None);
}

// --- update_workflow_config_text: settings-page rewrite (text→text) ---

const WF_DOC: &str = "schema: spec-driven\n\nlocale: tw\ncontext: |\n  line one\n  line two\n\nrules:\n  proposal:\n    - first rule\n    - second rule\n";

#[test]
fn workflow_update_sets_all_policy_fields_and_output_parses() {
    let fields = WorkflowPolicyFields {
        locale: Some("ja".into()),
        spec_locale: Some("auto".into()),
        tdd: true,
        audit: true,
        worktree: true,
    };
    let out = update_workflow_config_text(WF_DOC, &fields, &ContextEdit::Keep, None).expect("rewrite ok");
    let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
    assert_eq!(w.locale.as_deref(), Some("ja"));
    assert_eq!(w.spec_locale.as_deref(), Some("auto"));
    assert_eq!(w.tdd, Some(true));
    assert_eq!(w.audit, Some(true));
    assert_eq!(w.worktree, Some(true));
}

#[test]
fn workflow_update_preserves_untouched_key_values_verbatim() {
    // Re-serialization may change YAML styling; the parsed VALUES of every
    // untouched key (schema, multi-line context, rules) must stay identical
    // character for character.
    let fields = WorkflowPolicyFields { tdd: true, ..Default::default() };
    let out = update_workflow_config_text(WF_DOC, &fields, &ContextEdit::Keep, None).expect("rewrite ok");
    let (orig, new) = (wf(WF_DOC), wf(&out));
    assert_eq!(new.schema, orig.schema);
    assert_eq!(new.context, orig.context);
    assert_eq!(new.rules, orig.rules);
}

#[test]
fn workflow_update_keeps_unknown_keys() {
    let doc = "schema: spec-driven\nfuture_key: keep me\n";
    let out = update_workflow_config_text(doc, &WorkflowPolicyFields::default(), &ContextEdit::Keep, None).expect("rewrite ok");
    let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
    assert_eq!(m.get("future_key").and_then(|v| v.as_str()), Some("keep me"));
}

#[test]
fn workflow_update_default_values_remove_keys() {
    let doc = "locale: tw\nspec_locale: auto\ntdd: true\naudit: true\nschema: spec-driven\n";
    let out = update_workflow_config_text(doc, &WorkflowPolicyFields::default(), &ContextEdit::Keep, None).expect("rewrite ok");
    let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
    for key in ["locale", "spec_locale", "tdd", "audit"] {
        assert!(!m.contains_key(key), "key '{key}' must be removed, got: {out}");
    }
    assert!(m.contains_key("schema"));
}

#[test]
fn workflow_update_writes_and_removes_worktree() {
    // Spec scenario worktree 欄位寫入與呈現: `set worktree true` lands `worktree: true`.
    let fields = WorkflowPolicyFields { worktree: true, ..Default::default() };
    let out = update_workflow_config_text("schema: spec-driven\n", &fields, &ContextEdit::Keep, None)
        .expect("rewrite ok");
    assert_eq!(
        WorkflowConfig::from_text(Some(&out)).expect("output parses").worktree,
        Some(true),
        "got: {out}"
    );
    // Back to default removes the key, like the other toggles.
    let out = update_workflow_config_text(&out, &WorkflowPolicyFields::default(), &ContextEdit::Keep, None)
        .expect("rewrite ok");
    let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
    assert!(!m.contains_key("worktree"), "worktree must be removed, got: {out}");
    assert!(m.contains_key("schema"));
}

#[test]
fn workflow_update_bad_yaml_is_a_loud_error() {
    // Unlike WorkflowConfig::from_text (silent defaults), rewriting a malformed
    // document must fail loudly — otherwise the GUI would destroy user content.
    for bad in ["rules: [unclosed", "just a top-level scalar"] {
        assert!(
            update_workflow_config_text(bad, &WorkflowPolicyFields::default(), &ContextEdit::Keep, None).is_err(),
            "must reject {bad:?}"
        );
    }
}

#[test]
fn workflow_update_empty_input_creates_fresh_document() {
    // Absent config.yaml: the caller hands an empty text and gets a fresh
    // parseable document containing exactly the requested fields.
    let fields = WorkflowPolicyFields { locale: Some("tw".into()), ..Default::default() };
    let out = update_workflow_config_text("", &fields, &ContextEdit::Keep, None).expect("rewrite ok");
    let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
    assert_eq!(w.locale.as_deref(), Some("tw"));
}

#[test]
fn workflow_update_matches_spec_example_table() {
    // spec「政策欄位寫入效果」Example 表——逐行對應。
    // 無 tdd 鍵 | tdd 切開啟 | 新增 tdd: true
    let fields = WorkflowPolicyFields { tdd: true, ..Default::default() };
    let out = update_workflow_config_text("schema: spec-driven\n", &fields, &ContextEdit::Keep, None).unwrap();
    assert_eq!(WorkflowConfig::from_text(Some(&out)).expect("output parses").tdd, Some(true));
    // tdd: true | tdd 切關閉 | tdd 鍵被移除（預設即 false）——唯一鍵移除後輸出為空文件
    let out = update_workflow_config_text("tdd: true\n", &WorkflowPolicyFields::default(), &ContextEdit::Keep, None).unwrap();
    assert_eq!(WorkflowConfig::from_text(Some(&out)).expect("output parses").tdd, None, "got: {out}");
    // locale: tw、含 rules | spec_locale 選 auto | 新增 spec_locale: auto，locale 與 rules 原樣保留
    let doc = "locale: tw\nrules:\n  proposal:\n    - keep\n";
    let fields = WorkflowPolicyFields {
        locale: Some("tw".into()),
        spec_locale: Some("auto".into()),
        ..Default::default()
    };
    let out = update_workflow_config_text(doc, &fields, &ContextEdit::Keep, None).unwrap();
    let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
    assert_eq!(w.spec_locale.as_deref(), Some("auto"));
    assert_eq!(w.locale.as_deref(), Some("tw"));
    assert_eq!(w.rules, wf(doc).rules);
}

// --- policy locale 值域驗證（workflow-config-locale-validation，design D1／D3）---

#[test]
fn validate_policy_locales_accepts_codes_and_unset() {
    for code in ["tw", "ja", "en"] {
        let f = WorkflowPolicyFields { locale: Some(code.into()), ..Default::default() };
        assert!(validate_policy_locales(&f).is_ok(), "locale {code} must pass");
        let f = WorkflowPolicyFields { spec_locale: Some(code.into()), ..Default::default() };
        assert!(validate_policy_locales(&f).is_ok(), "spec_locale {code} must pass");
    }
    // spec_locale 另接受 auto；locale 不接受 auto
    let f = WorkflowPolicyFields { spec_locale: Some("auto".into()), ..Default::default() };
    assert!(validate_policy_locales(&f).is_ok());
    let f = WorkflowPolicyFields { locale: Some("auto".into()), ..Default::default() };
    assert!(validate_policy_locales(&f).is_err(), "locale auto must be rejected");
    // None（未設定）恆合法
    assert!(validate_policy_locales(&WorkflowPolicyFields::default()).is_ok());
}

#[test]
fn validate_policy_locales_rejects_display_names_and_case_variants() {
    // spec「locale 值域判定」Example 表的拒絕列
    for bad in ["繁體中文", "TW", "Auto", ""] {
        let f = WorkflowPolicyFields { locale: Some(bad.into()), ..Default::default() };
        assert!(validate_policy_locales(&f).is_err(), "locale {bad:?} must be rejected");
    }
    for bad in ["繁體中文", "zh-Hant", "AUTO"] {
        let f = WorkflowPolicyFields { spec_locale: Some(bad.into()), ..Default::default() };
        assert!(validate_policy_locales(&f).is_err(), "spec_locale {bad:?} must be rejected");
    }
}

#[test]
fn validate_policy_locales_error_names_field_value_and_codes() {
    let f = WorkflowPolicyFields { locale: Some("繁體中文".into()), ..Default::default() };
    let err = validate_policy_locales(&f).unwrap_err().to_string();
    for needle in ["locale", "繁體中文", "tw", "ja", "en"] {
        assert!(err.contains(needle), "error must contain {needle:?}, got: {err}");
    }
    let f = WorkflowPolicyFields { spec_locale: Some("zh-Hant".into()), ..Default::default() };
    let err = validate_policy_locales(&f).unwrap_err().to_string();
    for needle in ["spec_locale", "zh-Hant", "auto"] {
        assert!(err.contains(needle), "error must contain {needle:?}, got: {err}");
    }
}

#[test]
fn workflow_update_rejects_invalid_locale_fields_without_output() {
    let fields = WorkflowPolicyFields { locale: Some("繁體中文".into()), ..Default::default() };
    let err = update_workflow_config_text("schema: spec-driven\n", &fields, &ContextEdit::Keep, None)
        .unwrap_err()
        .to_string();
    assert!(err.contains("locale") && err.contains("繁體中文"), "got: {err}");
    let fields = WorkflowPolicyFields { spec_locale: Some("zh-Hant".into()), ..Default::default() };
    assert!(
        update_workflow_config_text("schema: spec-driven\n", &fields, &ContextEdit::Keep, None).is_err()
    );
}

#[test]
fn workflow_update_injected_special_values_are_rejected() {
    // Sharp-edges audit（Scoundrel）：locale 含換行或 YAML 語法的注入向量，
    // 自值域驗證上線後在序列化前即被拒絕——文件結構不可能被破壞。
    // （原測試斷言 escape 後 round-trip；值域驗證使拒絕成為更強的防護。）
    for evil in ["tw\nrules: {}", "a: b", "#comment", "'quoted'", "- item"] {
        let fields = WorkflowPolicyFields { locale: Some(evil.into()), ..Default::default() };
        assert!(
            update_workflow_config_text("schema: spec-driven\n", &fields, &ContextEdit::Keep, None).is_err(),
            "injection vector must be rejected: {evil:?}"
        );
    }
}

// --- update_workflow_config_text: context 三態與 rules 整份代換（desktop-config-rules-context） ---

/// 政策欄位與 context/rules 皆不動的呼叫縮寫（多數測試只關心其中一個變更集）。
fn rewrite(
    doc: &str,
    context: &ContextEdit,
    rules: Option<&[(String, Vec<String>)]>,
) -> anyhow::Result<String> {
    update_workflow_config_text(doc, &wf_fields_of(doc), context, rules)
}

/// 從原文讀出政策欄位現值（「完整目標狀態」契約：不想動政策就得先讀再回填）。
fn wf_fields_of(doc: &str) -> WorkflowPolicyFields {
    let w = wf(doc);
    WorkflowPolicyFields {
        locale: w.locale.clone(),
        spec_locale: w.spec_locale.clone(),
        tdd: w.tdd.unwrap_or(false),
        audit: w.audit.unwrap_or(false),
        worktree: w.worktree.unwrap_or(false),
    }
}

fn section(pairs: &[(&str, &[&str])]) -> Vec<(String, Vec<String>)> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
        .collect()
}

#[test]
fn workflow_update_context_set_round_trips_value_verbatim() {
    // spec Scenario 編輯專案說明並儲存：值逐字元一致、其餘鍵原樣保留。
    let text = "第一行說明\n\n第二行：含冒號: 與 # 井號\n";
    let out = rewrite(WF_DOC, &ContextEdit::Set(text.into()), None).expect("rewrite ok");
    let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
    assert_eq!(w.context.as_deref(), Some(text));
    assert_eq!(w.schema, wf(WF_DOC).schema);
    assert_eq!(w.rules, wf(WF_DOC).rules);
    assert_eq!(w.locale, wf(WF_DOC).locale);
}

#[test]
fn workflow_update_context_three_states() {
    // 三態：Keep 不動、Set 設值、Remove 移除鍵。
    let out = rewrite(WF_DOC, &ContextEdit::Keep, None).expect("rewrite ok");
    assert_eq!(wf(&out).context, wf(WF_DOC).context, "Keep must not touch context");
    let out = rewrite(WF_DOC, &ContextEdit::Remove, None).expect("rewrite ok");
    let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
    assert!(!m.contains_key("context"), "Remove must delete the key, got: {out}");
    let out = rewrite("schema: spec-driven\n", &ContextEdit::Set("新說明".into()), None).unwrap();
    assert_eq!(wf(&out).context.as_deref(), Some("新說明"));
}

#[test]
fn workflow_update_context_blank_set_removes_key() {
    // 清空即移除鍵的語意在 core 落實：Set 空白字串視同 Remove（zero-value 安全）。
    for blank in ["", "   ", "\n\n"] {
        let out = rewrite(WF_DOC, &ContextEdit::Set(blank.into()), None).expect("rewrite ok");
        let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
        assert!(!m.contains_key("context"), "blank {blank:?} must remove the key");
    }
}

#[test]
fn workflow_update_rules_replace_preserves_entry_order() {
    // spec Example 條目對調：tasks 節「先寫失敗測試」「更新文件」→ 上移後順序對調。
    let doc = "schema: spec-driven\nrules:\n  tasks:\n    - 先寫失敗測試\n    - 更新文件\n";
    let rules = section(&[("tasks", &["更新文件", "先寫失敗測試"])]);
    let out = rewrite(doc, &ContextEdit::Keep, Some(&rules)).expect("rewrite ok");
    let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
    assert_eq!(
        w.rules.get("tasks").map(Vec::as_slice),
        Some(["更新文件".to_string(), "先寫失敗測試".to_string()].as_slice())
    );
    assert_eq!(w.schema.as_deref(), Some("spec-driven"));
}

#[test]
fn workflow_update_rules_key_removal_matches_spec_example_table() {
    // spec Example 鍵移除語意——逐行對應。
    // context: 舊說明、rules 含 tasks 兩條 | 清空專案說明 | context 移除，rules.tasks 原樣保留
    let doc = "context: 舊說明\nrules:\n  tasks:\n    - a\n    - b\n";
    let out = rewrite(doc, &ContextEdit::Remove, None).expect("rewrite ok");
    let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
    assert!(!m.contains_key("context"));
    assert_eq!(wf(&out).rules, wf(doc).rules);
    // rules 含 proposal 與 tasks 兩節 | 刪除 tasks 節全部條目 | rules 僅餘 proposal 節
    let doc = "rules:\n  proposal:\n    - p1\n  tasks:\n    - t1\n";
    let rules = section(&[("proposal", &["p1"]), ("tasks", &[])]);
    let out = rewrite(doc, &ContextEdit::Keep, Some(&rules)).expect("rewrite ok");
    let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
    assert_eq!(w.rules.get("proposal").map(Vec::as_slice), Some(["p1".to_string()].as_slice()));
    assert!(!w.rules.contains_key("tasks"), "empty section must drop its key");
    // rules 僅含 tasks 一節 | 刪除該節全部條目 | rules 鍵整個被移除
    let doc = "schema: spec-driven\nrules:\n  tasks:\n    - t1\n";
    let rules = section(&[("tasks", &[])]);
    let out = rewrite(doc, &ContextEdit::Keep, Some(&rules)).expect("rewrite ok");
    let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
    assert!(!m.contains_key("rules"), "all-empty rules must remove the key, got: {out}");
    assert!(m.contains_key("schema"));
}

#[test]
fn workflow_update_rules_entries_trimmed_and_blank_dropped() {
    // 條目存入前 trim、空字串條目滌除；滌除後空節一併移除。
    let rules = section(&[("tasks", &["  先寫失敗測試  ", "   ", ""]), ("design", &["  ", ""])]);
    let out = rewrite("{}", &ContextEdit::Keep, Some(&rules)).expect("rewrite ok");
    let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
    assert_eq!(
        w.rules.get("tasks").map(Vec::as_slice),
        Some(["先寫失敗測試".to_string()].as_slice())
    );
    assert!(!w.rules.contains_key("design"), "all-blank section must drop its key");
}

#[test]
fn workflow_update_rules_reserved_char_entries_round_trip() {
    // spec Example 保留字元條目自動加引號：GIVEN proposal 節一條，WHEN tasks 節新增
    // 「@完成後執行全部測試」，THEN 可解析、值逐字元還原、proposal 與 schema 保留。
    let doc = "schema: spec-driven\nrules:\n  proposal:\n    - 提案必須列出影響的 crates\n";
    let rules = section(&[
        ("proposal", &["提案必須列出影響的 crates"]),
        ("tasks", &["@完成後執行全部測試"]),
    ]);
    let out = rewrite(doc, &ContextEdit::Keep, Some(&rules)).expect("rewrite ok");
    let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
    assert_eq!(
        w.rules.get("tasks").map(Vec::as_slice),
        Some(["@完成後執行全部測試".to_string()].as_slice())
    );
    assert_eq!(
        w.rules.get("proposal").map(Vec::as_slice),
        Some(["提案必須列出影響的 crates".to_string()].as_slice())
    );
    assert_eq!(w.schema.as_deref(), Some("spec-driven"));
    // 反引號開頭（既知炸檔地雷）與其他 YAML 保留起始字元亦須 round-trip。
    for evil in ["`cargo test` 全綠", "@標註開頭", "*星號開頭", "&錨點開頭"] {
        let rules = section(&[("tasks", &[evil])]);
        let out = rewrite("{}", &ContextEdit::Keep, Some(&rules)).expect("rewrite ok");
        let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
        assert_eq!(
            w.rules.get("tasks").map(Vec::as_slice),
            Some([evil.to_string()].as_slice()),
            "round-trip for {evil:?}"
        );
        assert!(!w.rules.is_empty(), "document must stay parseable for {evil:?}");
    }
}

#[test]
fn workflow_update_rules_none_leaves_rules_untouched() {
    // rules: None＝不動——政策欄位寫入路徑不得波及 rules。
    let out = rewrite(WF_DOC, &ContextEdit::Keep, None).expect("rewrite ok");
    assert_eq!(wf(&out).rules, wf(WF_DOC).rules);
}

#[test]
fn workflow_update_content_edit_preserves_reserved_keys_verbatim() {
    // MODIFIED 需求保留名單：remote、spec_dir、未知鍵於 context/rules 寫入時逐值保留。
    let doc = "spec_dir: docs/specs\nremote:\n  url: https://example.com\n  repo: main\nfuture_key: keep me\ncontext: old\n";
    let rules = section(&[("tasks", &["新規則"])]);
    let out = rewrite(doc, &ContextEdit::Set("新說明".into()), Some(&rules)).expect("rewrite ok");
    let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
    let orig: serde_yaml::Mapping = serde_yaml::from_str(doc).expect("mapping");
    for key in ["spec_dir", "remote", "future_key"] {
        assert_eq!(m.get(key), orig.get(key), "key '{key}' must carry over");
    }
    assert_eq!(wf(&out).context.as_deref(), Some("新說明"));
}

#[test]
fn workflow_update_content_edit_bad_yaml_is_a_loud_error() {
    // 不經 rewrite helper——壞 YAML 連現值都讀不出，直接以 default 政策呼叫。
    for bad in ["rules: [unclosed", "just a top-level scalar"] {
        assert!(
            update_workflow_config_text(
                bad,
                &WorkflowPolicyFields::default(),
                &ContextEdit::Set("x".into()),
                None,
            )
            .is_err(),
            "must reject {bad:?}"
        );
    }
}

// --- update_workflow_config_text: 文字層手術（workflow-config-surgical-write） ---

/// 只設定 locale、其餘政策欄位維持預設的縮寫。
fn loc(code: &str) -> WorkflowPolicyFields {
    WorkflowPolicyFields { locale: Some(code.into()), ..Default::default() }
}

#[test]
fn surgical_missing_key_inserts_below_schema_with_single_blank_lines() {
    // spec Scenario 缺鍵插於 schema 之下且空行區隔——相鄰處已有空行時不重複補。
    let doc = "# 模板註解\nschema: spec-driven\n\ncontext: |\n  第一行\n\nrules:\n  proposal:\n    - 條目一\n";
    let out = update_workflow_config_text(doc, &loc("tw"), &ContextEdit::Keep, None).expect("rewrite ok");
    assert_eq!(
        out,
        "# 模板註解\nschema: spec-driven\n\nlocale: tw\n\ncontext: |\n  第一行\n\nrules:\n  proposal:\n    - 條目一\n"
    );
    // 相鄰處原本沒有空行時補上：插入區塊與前後內容各恰一空行。
    let out = update_workflow_config_text(
        "schema: spec-driven\ncontext: 說明\n",
        &loc("tw"),
        &ContextEdit::Keep,
        None,
    )
    .expect("rewrite ok");
    assert_eq!(out, "schema: spec-driven\n\nlocale: tw\n\ncontext: 說明\n");
}

#[test]
fn surgical_multiple_missing_keys_insert_one_canonical_block() {
    // 多缺鍵一次寫入：正典序（locale、spec_locale、tdd、audit、worktree）成連續區塊，不分散。
    let fields = WorkflowPolicyFields {
        locale: Some("tw".into()),
        spec_locale: Some("auto".into()),
        tdd: true,
        audit: true,
        worktree: true,
    };
    let out = update_workflow_config_text(
        "schema: spec-driven\n\ncontext: 說明\n",
        &fields,
        &ContextEdit::Keep,
        None,
    )
    .expect("rewrite ok");
    assert_eq!(
        out,
        "schema: spec-driven\n\nlocale: tw\nspec_locale: auto\ntdd: true\naudit: true\nworktree: true\n\ncontext: 說明\n"
    );
}

#[test]
fn surgical_preserves_comments_and_blank_lines_byte_for_byte() {
    // spec Scenario 註解與空行逐位元保留：原位改值只動目標行。
    let doc = "# 頭註解\nschema: spec-driven\n\n# locale 註解\nlocale: tw\n\n# 尾註解\ncontext: 說明\n";
    let out = update_workflow_config_text(doc, &loc("ja"), &ContextEdit::Keep, None).expect("rewrite ok");
    assert_eq!(
        out,
        "# 頭註解\nschema: spec-driven\n\n# locale 註解\nlocale: ja\n\n# 尾註解\ncontext: 說明\n"
    );
}

#[test]
fn surgical_tail_key_updated_in_place_never_moves() {
    // spec Scenario 檔尾既有鍵原位改值不搬家（曾被舊版附加在 rules 之後）。
    let doc = "schema: spec-driven\n\nrules:\n  tasks:\n    - a\n\nlocale: tw\n";
    let out = update_workflow_config_text(doc, &loc("ja"), &ContextEdit::Keep, None).expect("rewrite ok");
    assert_eq!(out, "schema: spec-driven\n\nrules:\n  tasks:\n    - a\n\nlocale: ja\n");
}

#[test]
fn surgical_missing_schema_inserts_at_file_top() {
    // spec Scenario schema 缺席時插於檔案最頂端，與後續內容之間恰一空行。
    let fields = WorkflowPolicyFields { tdd: true, ..Default::default() };
    let out = update_workflow_config_text("context: 說明\n", &fields, &ContextEdit::Keep, None)
        .expect("rewrite ok");
    assert_eq!(out, "tdd: true\n\ncontext: 說明\n");
    // 空檔案：只有插入區塊本身，無多餘空行。
    let out = update_workflow_config_text("", &loc("tw"), &ContextEdit::Keep, None).expect("rewrite ok");
    assert_eq!(out, "locale: tw\n");
}

#[test]
fn surgical_user_content_below_schema_shifts_down_verbatim() {
    // schema 底下使用者自加內容：插入點仍在 schema 鍵行之後，使用者內容原樣後移。
    let doc = "schema: spec-driven\n# 使用者自加說明\nmy_key: 自訂\n\ncontext: 說明\n";
    let out = update_workflow_config_text(doc, &loc("tw"), &ContextEdit::Keep, None).expect("rewrite ok");
    assert_eq!(
        out,
        "schema: spec-driven\n\nlocale: tw\n\n# 使用者自加說明\nmy_key: 自訂\n\ncontext: 說明\n"
    );
}

#[test]
fn surgical_set_false_removes_key_line_keeps_comment_above() {
    // spec Scenario 設 false 移除鍵：只刪鍵行，上方註解行仍在。
    let doc = "schema: spec-driven\n\n# audit 開關\naudit: true\n\ncontext: 說明\n";
    let out = update_workflow_config_text(doc, &WorkflowPolicyFields::default(), &ContextEdit::Keep, None)
        .expect("rewrite ok");
    assert_eq!(out, "schema: spec-driven\n\n# audit 開關\n\ncontext: 說明\n");
}

#[test]
fn surgical_context_replaces_only_its_block() {
    // context 整塊替換：僅 context 區塊變動，前後註解與其他區段逐位元不變；
    // 舊值內含空行（block scalar 中段空行）仍屬同一區塊、不誤切。
    let doc = "schema: spec-driven\n\n# 說明區\ncontext: |\n  舊一\n\n  舊二\n\n# 規則區\nrules:\n  tasks:\n    - a\n";
    let out = update_workflow_config_text(
        doc,
        &WorkflowPolicyFields::default(),
        &ContextEdit::Set("新一\n新二\n".into()),
        None,
    )
    .expect("rewrite ok");
    assert_eq!(
        out,
        "schema: spec-driven\n\n# 說明區\ncontext: |\n  新一\n  新二\n\n# 規則區\nrules:\n  tasks:\n    - a\n"
    );
}

#[test]
fn surgical_multiline_unknown_key_is_not_misjudged() {
    // 未知頂層鍵含多行 block scalar：縮排內容行（即使含冒號）不被誤判為頂層鍵。
    let doc = "notes: |\n  first: 看似鍵\n  second: 也是\nlocale: tw\n";
    let out = update_workflow_config_text(doc, &loc("ja"), &ContextEdit::Keep, None).expect("rewrite ok");
    assert_eq!(out, "notes: |\n  first: 看似鍵\n  second: 也是\nlocale: ja\n");
}

#[test]
fn surgical_no_trailing_newline_and_crlf_edges() {
    // 結尾無換行：插入前先補行終止，插入區塊照常。
    let out = update_workflow_config_text("schema: spec-driven", &loc("tw"), &ContextEdit::Keep, None)
        .expect("rewrite ok");
    assert_eq!(out, "schema: spec-driven\n\nlocale: tw\n");
    // 結尾無換行的末行原位改值：不憑空補結尾換行。
    let out = update_workflow_config_text("schema: spec-driven\nlocale: tw", &loc("ja"), &ContextEdit::Keep, None)
        .expect("rewrite ok");
    assert_eq!(out, "schema: spec-driven\nlocale: ja");
    // CRLF 檔：未動行逐位元保留，改寫行與插入行沿用檔案的 CRLF。
    let fields = WorkflowPolicyFields { locale: Some("ja".into()), tdd: true, ..Default::default() };
    let out = update_workflow_config_text(
        "schema: spec-driven\r\nlocale: tw\r\n",
        &fields,
        &ContextEdit::Keep,
        None,
    )
    .expect("rewrite ok");
    assert_eq!(out, "schema: spec-driven\r\n\r\ntdd: true\r\n\r\nlocale: ja\r\n");
}

#[test]
fn surgical_rewrite_verification_failure_is_fail_closed() {
    // spec Scenario 內部改寫驗證失敗拒絕寫入：引號鍵 'locale' 對文字層手術不可見，
    // 插入裸鍵後重新解析成重複鍵——驗證必須攔下並以單行錯誤拒寫（函式為純 text→text，
    // 呼叫端收到 Err 即不落檔，原檔逐位元不變）。
    let err = update_workflow_config_text("'locale': tw\n", &loc("ja"), &ContextEdit::Keep, None)
        .expect_err("must fail closed")
        .to_string();
    assert!(err.contains("internal rewrite verification failed"), "got: {err}");
    assert!(!err.contains('\n'), "error must be single-line, got: {err}");
    // 另一分支：輸出可解析但與目標狀態不等值——同樣單行拒絕。
    let target: serde_yaml::Mapping = serde_yaml::from_str("locale: tw\n").expect("mapping");
    let err = verify_rewritten_config("locale: ja\n", &target)
        .expect_err("mismatch must fail closed")
        .to_string();
    assert!(err.contains("internal rewrite verification failed"), "got: {err}");
    assert!(!err.contains('\n'), "error must be single-line, got: {err}");
}

// --- update_app_config_tools_text: builtin tool selection rewrite ---

use crate::skills::Tool;

#[test]
fn tools_update_replaces_builtin_selection() {
    let out = update_app_config_tools_text("tools:\n  - claude\n", &[Tool::Claude, Tool::Codex])
        .expect("rewrite ok");
    let a = app(&out);
    assert_eq!(a.tools.len(), 2);
    assert!(matches!(&a.tools[0], ToolEntry::Builtin(s) if s == "claude"));
    assert!(matches!(&a.tools[1], ToolEntry::Builtin(s) if s == "codex"));
}

#[test]
fn tools_update_removes_deselected_builtins() {
    let out = update_app_config_tools_text("tools:\n  - claude\n  - codex\n", &[Tool::Claude])
        .expect("rewrite ok");
    let a = app(&out);
    assert_eq!(a.tools.len(), 1);
    assert!(matches!(&a.tools[0], ToolEntry::Builtin(s) if s == "claude"));
}

#[test]
fn tools_update_preserves_descriptors_and_other_keys() {
    let doc = concat!(
        "spec_dir: docs/specs\n",
        "tools:\n",
        "  - claude\n",
        "  - name: wad-harness\n",
        "    skills_dir: .wad/skills\n",
        "    instructions_file: WAD.md\n",
        "    future_field: keep me\n",
        "remote:\n",
        "  url: https://team.example.com/x\n",
        "  repo: backend\n",
    );
    let out = update_app_config_tools_text(doc, &[Tool::Codex]).expect("rewrite ok");
    let a = app(&out);
    assert_eq!(a.spec_dir.as_deref(), Some("docs/specs"));
    let r = a.remote.as_ref().expect("remote section kept");
    assert_eq!(r.url.as_deref(), Some("https://team.example.com/x"));
    assert_eq!(r.repo.as_deref(), Some("backend"));
    // claude 落選移除；descriptor 原樣保留（保序）；codex 新入選 append 尾端。
    assert_eq!(a.tools.len(), 2);
    match &a.tools[0] {
        ToolEntry::Descriptor(d) => {
            assert_eq!(d.name.as_deref(), Some("wad-harness"));
            assert_eq!(d.skills_dir.as_deref(), Some(".wad/skills"));
            assert_eq!(d.instructions_file.as_deref(), Some("WAD.md"));
        }
        other => panic!("expected descriptor first, got {other:?}"),
    }
    assert!(matches!(&a.tools[1], ToolEntry::Builtin(s) if s == "codex"));
    // 描述子的未知欄位也逐字保留（raw value carry-over）。
    let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
    let tools = m.get("tools").and_then(|v| v.as_sequence()).expect("tools seq");
    assert_eq!(
        tools[0].get("future_field").and_then(|v| v.as_str()),
        Some("keep me")
    );
}

#[test]
fn tools_update_bad_yaml_is_a_loud_error() {
    for bad in ["tools: [unclosed", "just a top-level scalar"] {
        assert!(
            update_app_config_tools_text(bad, &[Tool::Claude]).is_err(),
            "must reject {bad:?}"
        );
    }
}

#[test]
fn tools_update_creates_tools_key_when_absent() {
    for doc in ["", "spec_dir: docs/specs\n"] {
        let out = update_app_config_tools_text(doc, &[Tool::Claude]).expect("rewrite ok");
        let a = app(&out);
        assert_eq!(a.tools.len(), 1, "for input {doc:?}");
        assert!(matches!(&a.tools[0], ToolEntry::Builtin(s) if s == "claude"));
    }
}

/// Spec example「built-in 選集轉換」第三列（codex → claude,codex）：兩者都在且不重複，
/// 同時鎖住 Implementation Contract 的「其他鍵保持可解析且值不變」——包含未知頂層鍵。
#[test]
fn tools_update_adds_missing_builtin_and_keeps_unknown_top_level_keys() {
    let doc = concat!(
        "tools:\n",
        "  - codex\n",
        "  - name: wad-harness\n",
        "    skills_dir: .wad/skills\n",
        "    instructions_file: WAD.md\n",
        "future_top_level: keep me\n",
    );
    let out = update_app_config_tools_text(doc, &[Tool::Claude, Tool::Codex]).expect("rewrite ok");
    let a = app(&out);
    let builtins: Vec<&str> = a
        .tools
        .iter()
        .filter_map(|e| match e {
            ToolEntry::Builtin(s) => Some(s.as_str()),
            ToolEntry::Descriptor(_) => None,
        })
        .collect();
    assert_eq!(builtins, ["codex", "claude"], "既有項保序、新項 append，且不重複");
    assert!(a.tools.iter().any(|e| matches!(e, ToolEntry::Descriptor(_))));
    let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
    assert_eq!(
        m.get("future_top_level").and_then(|v| v.as_str()),
        Some("keep me")
    );
}

#[test]
fn tools_update_empty_selection_keeps_descriptors_only() {
    let doc = "tools:\n  - claude\n  - name: wad-harness\n    skills_dir: .wad/skills\n    instructions_file: WAD.md\n";
    let out = update_app_config_tools_text(doc, &[]).expect("rewrite ok");
    let a = app(&out);
    assert_eq!(a.tools.len(), 1);
    assert!(matches!(&a.tools[0], ToolEntry::Descriptor(_)));
}

fn mapping(text: &str) -> serde_yaml::Mapping {
    match serde_yaml::from_str::<serde_yaml::Value>(text).expect("parses") {
        serde_yaml::Value::Mapping(m) => m,
        other => panic!("expected a mapping, got {other:?}"),
    }
}

/// 鎖定基準：`.speclink.yaml` 不存在時 `write_remote_section` 建檔，且只寫入
/// remote 鍵。
#[test]
fn write_remote_section_creates_the_file_with_only_the_remote_key() {
    let root = TempCfgDir::new("write-creates");
    write_remote_section(&root.dir, "https://example.test/store", Some("acme/specs"))
        .expect("writes");
    let doc = mapping(&root.read_app_yaml());
    assert_eq!(doc.len(), 1, "a fresh file carries only the remote section: {doc:?}");
    let remote = doc.get("remote").expect("remote section").clone();
    assert_eq!(
        remote,
        serde_yaml::from_str::<serde_yaml::Value>(
            "url: https://example.test/store\nrepo: acme/specs\n"
        )
        .unwrap()
    );
}

/// 鎖定基準：寫入與移除 remote section 都不動其他頂層鍵——`spec_dir`、描述子
/// 物件（含其欄位）與未知鍵的值逐一保留。
#[test]
fn the_remote_section_verbs_keep_every_other_top_level_key() {
    let root = TempCfgDir::new("keeps-keys");
    let original = "spec_dir: docs/spec\n\
tools:\n\
  - claude\n\
  - name: my-harness\n\
    skills_dir: .my-harness/skills\n\
    invocation: tool-call\n\
future_key:\n\
  nested: value\n";
    root.app_yaml(original);
    let before = mapping(original);

    write_remote_section(&root.dir, "https://example.test/store", None).expect("writes");
    let after_write = mapping(&root.read_app_yaml());
    for key in ["spec_dir", "tools", "future_key"] {
        assert_eq!(
            after_write.get(key),
            before.get(key),
            "write_remote_section must keep '{key}' untouched"
        );
    }
    assert_eq!(
        after_write.get("remote"),
        Some(&serde_yaml::from_str::<serde_yaml::Value>("url: https://example.test/store\n").unwrap()),
        "a None repo writes url only"
    );

    assert!(remove_remote_section(&root.dir).expect("removes"), "the section was there");
    let after_remove = mapping(&root.read_app_yaml());
    assert!(after_remove.get("remote").is_none(), "the section is gone");
    for key in ["spec_dir", "tools", "future_key"] {
        assert_eq!(
            after_remove.get(key),
            before.get(key),
            "remove_remote_section must keep '{key}' untouched"
        );
    }
}

/// 鎖定基準：非 mapping 的 `.speclink.yaml` 讓兩支動詞以 `invalid .speclink.yaml`
/// 開頭的單行錯誤失敗，且不覆寫使用者的檔案。
#[test]
fn the_remote_section_verbs_refuse_a_non_mapping_document_without_writing() {
    let root = TempCfgDir::new("non-mapping");
    let original = "- claude\n- codex\n";
    root.app_yaml(original);

    for message in [
        write_remote_section(&root.dir, "https://example.test/store", None)
            .expect_err("refuses")
            .to_string(),
        remove_remote_section(&root.dir).expect_err("refuses").to_string(),
    ] {
        assert!(
            message.starts_with("invalid .speclink.yaml"),
            "single-line error naming the file: {message}"
        );
        assert!(!message.contains('\n'), "the error stays one line: {message}");
    }
    assert_eq!(root.read_app_yaml(), original, "a refused verb writes nothing");
}

/// 鎖定基準：沒有 remote 鍵時 `remove_remote_section` 回 `Ok(false)`，檔案位元級不變。
#[test]
fn remove_remote_section_is_a_no_op_without_a_remote_key() {
    let root = TempCfgDir::new("remove-noop");
    let original = "spec_dir: openspec\ntools:\n  - claude\n";
    root.app_yaml(original);
    assert!(!remove_remote_section(&root.dir).expect("no-op"), "nothing to remove");
    assert_eq!(root.read_app_yaml(), original, "the file is byte-identical");
}
