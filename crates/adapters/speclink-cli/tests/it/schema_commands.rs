//! `speclink schemas` / `speclink schema …` — the workflow-schema management surface.
//!
//! The built-in schema is parsed from one canonical document compiled into the binary, so
//! `schema fork spec-driven` must ship that document byte for byte and `schemas` must show
//! the document's own description. The user-level schema location is redirected into the
//! temp project (HOME / USERPROFILE) so a developer's real home directory cannot change a
//! result.

use std::path::PathBuf;
use std::process::{Command, Output};

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn new(tag: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-schema-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("openspec")).unwrap();
        std::fs::create_dir_all(dir.join("home")).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        TempProject { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            .env("NO_COLOR", "1")
            .env("HOME", self.dir.join("home"))
            .env("USERPROFILE", self.dir.join("home"))
            .env_remove("XDG_CONFIG_HOME")
            .env_remove("SPECLINK_STORE_URL")
            .output()
            .expect("run speclink binary")
    }

    fn schema_dir(&self, name: &str) -> PathBuf {
        self.dir.join("openspec").join("schemas").join(name)
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

/// The canonical built-in document, as a parsed mapping.
fn canonical() -> serde_yaml::Value {
    serde_yaml::from_str(speclink_core::schema::FORK_SCHEMA_YAML).expect("canonical yaml parses")
}

#[test]
fn fork_of_the_builtin_ships_the_canonical_document_verbatim() {
    let p = TempProject::new("fork-verbatim");

    let out = p.run(&["schema", "fork", "spec-driven"]);
    assert!(out.status.success(), "fork failed: {}", stderr_of(&out));

    let forked = std::fs::read(p.schema_dir("spec-driven-custom").join("schema.yaml"))
        .expect("forked schema.yaml exists");
    assert_eq!(
        forked,
        speclink_core::schema::FORK_SCHEMA_YAML.as_bytes(),
        "fork 產出必須與正典 YAML 逐位元組相同"
    );
}

#[test]
fn the_builtin_description_comes_from_the_canonical_document() {
    let p = TempProject::new("description");

    let out = p.run(&["schemas", "--json"]);
    assert!(out.status.success(), "schemas failed: {}", stderr_of(&out));

    let listed: serde_json::Value = serde_json::from_slice(&out.stdout).expect("schemas --json");
    let entry = listed
        .as_array()
        .expect("listing is an array")
        .iter()
        .find(|e| e["name"] == "spec-driven")
        .expect("spec-driven is listed");

    let canon = canonical();
    let want = canon["description"].as_str().expect("canonical description");
    assert!(
        want.contains("design optional"),
        "正典 description 應點名 design 為選配：{want}"
    );
    assert_eq!(entry["description"].as_str(), Some(want));
}

#[test]
fn fork_and_init_refuse_a_name_that_is_not_kebab_case() {
    let p = TempProject::new("name-format");

    let cases: [&[&str]; 2] = [
        &["schema", "init", "My_Schema"],
        &["schema", "fork", "spec-driven", "My_Schema"],
    ];
    for args in cases {
        let out = p.run(args);
        assert!(
            !out.status.success(),
            "{args:?} 應以非 0 exit code 拒絕，實得成功"
        );
        let said = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr_of(&out));
        assert!(
            said.contains("lowercase kebab-case"),
            "{args:?} 的訊息須說明名稱格式：{said}"
        );
        assert!(
            !p.schema_dir("My_Schema").exists(),
            "{args:?} 被拒後不得留下目錄"
        );
    }
}

#[test]
fn init_accepts_a_kebab_case_name() {
    let p = TempProject::new("name-ok");

    let out = p.run(&["schema", "init", "my-flow"]);
    assert!(out.status.success(), "init failed: {}", stderr_of(&out));
    assert!(p.schema_dir("my-flow").join("schema.yaml").is_file());
}

/// A minimal, valid custom schema plus its template file.
fn write_custom_schema(p: &TempProject, name: &str, template_body: Option<&str>) {
    let dir = p.schema_dir(name);
    std::fs::create_dir_all(dir.join("templates")).unwrap();
    std::fs::write(
        dir.join("schema.yaml"),
        format!(
            "name: {name}\nversion: 1\nartifacts:\n- id: plan\n  generates: plan.md\n  \
             description: The plan\n  template: plan.md\n"
        ),
    )
    .unwrap();
    if let Some(body) = template_body {
        std::fs::write(dir.join("templates").join("plan.md"), body).unwrap();
    }
}

#[test]
fn which_all_lists_every_schema_with_its_resolution() {
    let p = TempProject::new("which-all");
    write_custom_schema(&p, "my-flow", Some("# plan\n"));

    let out = p.run(&["schema", "which", "--all", "--json"]);
    assert!(out.status.success(), "which --all failed: {}", stderr_of(&out));

    let listed: serde_json::Value = serde_json::from_slice(&out.stdout).expect("which --all --json");
    let rows = listed.as_array().expect("listing is an array");
    let names: Vec<&str> = rows.iter().filter_map(|r| r["name"].as_str()).collect();
    assert!(names.contains(&"spec-driven"), "內建應列出：{names:?}");
    assert!(names.contains(&"my-flow"), "專案自訂應列出：{names:?}");

    let builtin = rows.iter().find(|r| r["name"] == "spec-driven").unwrap();
    assert_eq!(builtin["resolved"].as_str(), Some("built-in"));
    let custom = rows.iter().find(|r| r["name"] == "my-flow").unwrap();
    assert_eq!(custom["resolved"].as_str(), Some("project"));
    let source = custom["sources"][0]["path"].as_str().expect("source path");
    assert!(source.contains("my-flow"), "來源路徑應指向該 schema：{source}");
}

#[test]
fn validate_reports_a_missing_template_file() {
    let p = TempProject::new("validate-template");
    write_custom_schema(&p, "no-template", None);

    let out = p.run(&["schema", "validate", "no-template"]);
    assert!(!out.status.success(), "缺 template 檔應以非 0 exit code 結束");
    let said = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr_of(&out));
    assert!(said.contains("plan.md"), "訊息須指名缺席的 template 檔：{said}");
}

#[test]
fn validate_verbose_prints_each_check() {
    let p = TempProject::new("validate-verbose");
    write_custom_schema(&p, "my-flow", Some("# plan\n"));

    let out = p.run(&["schema", "validate", "my-flow", "--verbose"]);
    assert!(out.status.success(), "validate failed: {}", stderr_of(&out));
    let said = String::from_utf8_lossy(&out.stdout).to_string();
    for step in ["parse", "artifact ids", "dependency references", "cycles", "templates"] {
        assert!(said.contains(step), "--verbose 須逐項印出「{step}」：{said}");
    }
}

#[test]
fn init_default_sets_the_project_schema_and_preserves_the_rest() {
    let p = TempProject::new("init-default");
    let config = p.dir.join("openspec").join("config.yaml");
    let original = "# a leading comment\nlocale: tw\ntdd: true\n";
    std::fs::write(&config, original).unwrap();

    let out = p.run(&["schema", "init", "my-flow", "--default"]);
    assert!(out.status.success(), "init --default failed: {}", stderr_of(&out));

    let after = std::fs::read_to_string(&config).unwrap();
    assert!(
        after.contains("schema: my-flow"),
        "config.yaml 應帶 schema: my-flow：{after}"
    );
    for kept in ["# a leading comment", "locale: tw", "tdd: true"] {
        assert!(after.contains(kept), "既有內容 {kept} 須逐字保留：{after}");
    }
}

#[test]
fn init_default_creates_the_config_when_it_is_absent() {
    let p = TempProject::new("init-default-absent");
    let config = p.dir.join("openspec").join("config.yaml");
    assert!(!config.exists());

    let out = p.run(&["schema", "init", "my-flow", "--default"]);
    assert!(out.status.success(), "init --default failed: {}", stderr_of(&out));
    assert_eq!(std::fs::read_to_string(&config).unwrap(), "schema: my-flow\n");
}

#[test]
fn init_produces_a_skeleton_that_loads_and_validates() {
    let p = TempProject::new("init-skeleton");

    let out = p.run(&["schema", "init", "my-flow"]);
    assert!(out.status.success(), "init failed: {}", stderr_of(&out));

    // The default description carries a colon; an unquoted scalar would make the
    // document unparseable and every verb would then refuse the schema.
    let listed = p.run(&["schemas", "--json"]);
    let listed: serde_json::Value = serde_json::from_slice(&listed.stdout).expect("schemas --json");
    let entry = listed
        .as_array()
        .expect("listing is an array")
        .iter()
        .find(|e| e["name"] == "my-flow")
        .expect("my-flow is listed");
    assert_eq!(
        entry["artifacts"],
        serde_json::json!(["plan", "tasks"]),
        "骨架的 artifact 清單須讀得出來（文件可解析）"
    );

    for (template, body) in [("plan.md", "# plan\n"), ("tasks.md", "# tasks\n")] {
        let path = p.schema_dir("my-flow").join("templates").join(template);
        assert_eq!(
            std::fs::read_to_string(&path).ok().as_deref(),
            Some(body),
            "骨架 template {template} 的內容須為標題行"
        );
    }
    let yaml = std::fs::read_to_string(p.schema_dir("my-flow").join("schema.yaml")).unwrap();
    assert!(yaml.contains("apply:"), "schema.yaml 須含 apply 區塊：{yaml}");

    let validated = p.run(&["schema", "validate", "my-flow"]);
    assert!(
        validated.status.success(),
        "init 產出須通過自身 validate: {}",
        stderr_of(&validated)
    );
}

#[test]
fn init_default_refuses_when_the_config_cannot_be_read() {
    let p = TempProject::new("init-default-unreadable");
    let config = p.dir.join("openspec").join("config.yaml");
    // 非 UTF-8 內容：檔案存在但讀取失敗——與「缺席」是兩回事，不得被當成空白覆寫。
    std::fs::write(&config, [0xFF, 0xFE, b'x']).unwrap();
    let before = std::fs::read(&config).unwrap();

    let out = p.run(&["schema", "init", "my-flow", "--default"]);
    assert!(!out.status.success(), "讀不到的 config 應中止而非覆寫");
    assert_eq!(std::fs::read(&config).unwrap(), before, "config.yaml 一個位元組都不得動");
    let said = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr_of(&out));
    assert!(said.contains("unchanged"), "訊息須說明預設未設：{said}");
    assert!(
        p.schema_dir("my-flow").join("schema.yaml").is_file(),
        "骨架本身仍應建立（契約：骨架已建、預設未設）"
    );
}

#[test]
fn init_refuses_artifact_ids_that_escape_or_collide() {
    let p = TempProject::new("init-artifact-ids");

    let esc = p.run(&["schema", "init", "esc", "--artifacts", "../x"]);
    assert!(!esc.status.success(), "路徑逸出的 artifact id 應被拒");
    assert!(!p.schema_dir("esc").exists(), "被拒後不得留下任何檔案");

    let dup = p.run(&["schema", "init", "dup", "--artifacts", "plan,plan"]);
    assert!(!dup.status.success(), "重複 artifact id 應被拒");
    let said = format!("{}{}", String::from_utf8_lossy(&dup.stdout), stderr_of(&dup));
    assert!(said.contains("Duplicate artifact ID"), "{said}");
    assert!(!p.schema_dir("dup").exists(), "被拒後不得留下任何檔案");
}

#[test]
fn init_survives_a_description_with_newlines() {
    let p = TempProject::new("init-multiline-desc");

    let out = p.run(&["schema", "init", "ml", "--description", "line one\nline two: colon"]);
    assert!(out.status.success(), "init failed: {}", stderr_of(&out));
    let v = p.run(&["schema", "validate", "ml"]);
    assert!(
        v.status.success(),
        "多行 description 的骨架必須通過 validate: {}",
        stderr_of(&v)
    );
}

#[test]
fn which_all_deduplicates_a_shadowed_name() {
    let p = TempProject::new("which-all-shadow");
    // 專案自訂一個與內建同名的 schema——解析時 project 遮蔽 built-in。
    write_custom_schema(&p, "spec-driven", Some("# plan\n"));

    let out = p.run(&["schema", "which", "--all", "--json"]);
    assert!(out.status.success(), "which --all failed: {}", stderr_of(&out));
    let rows: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let rows = rows.as_array().expect("listing is an array");
    let matches: Vec<_> = rows.iter().filter(|r| r["name"] == "spec-driven").collect();
    assert_eq!(matches.len(), 1, "同名 schema 只列一列：{rows:?}");
    assert_eq!(matches[0]["resolved"].as_str(), Some("project"), "解析結果是勝出者");
    let sources = matches[0]["sources"].as_array().expect("sources");
    assert_eq!(sources.len(), 2, "被遮蔽的 built-in 一併列出：{sources:?}");
    assert_eq!(sources[1]["source"].as_str(), Some("built-in"));
}
