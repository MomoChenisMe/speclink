//! Build `instructions` payloads (per-artifact and apply mode) with config injection.

use crate::config::WorkflowConfig;
use crate::model::{self, Change};
use crate::preflight::Preflight;
use crate::schema::Schema;
use crate::store::Store;
use crate::tasks::{self, Task};
use crate::workspace::Workspace;
use serde::Serialize;
use std::path::Path;

fn join_display(base: &Path, rel: &str) -> String {
    base.join(rel).to_string_lossy().to_string()
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct Dependency {
    pub id: String,
    pub done: bool,
    pub path: String,
    pub description: String,
}

/// Per-artifact instructions payload (frozen field order).
#[derive(Debug, Serialize, serde::Deserialize)]
pub struct ArtifactInstructions {
    #[serde(rename = "changeName")]
    pub change_name: String,
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    #[serde(rename = "schemaName")]
    pub schema_name: String,
    #[serde(rename = "changeDir")]
    pub change_dir: String,
    #[serde(rename = "outputPath")]
    pub output_path: String,
    pub description: String,
    /// Omitted entirely when the (custom) schema has no instruction (frozen output shape).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<String>>,
    pub locale: String,
    pub template: String,
    pub dependencies: Vec<Dependency>,
    pub unlocks: Vec<String>,
}

/// Append `note` as the instruction's last paragraph, one blank line after the text before
/// it; it is the whole instruction when the schema gives the artifact none. The result ends
/// with one newline, like a schema instruction does — the human renderer relies on that.
fn append_paragraph(instruction: &mut Option<String>, note: &str) {
    let before = instruction.take().unwrap_or_default();
    let before = before.trim_end_matches('\n');
    *instruction = Some(if before.is_empty() {
        format!("{note}\n")
    } else {
        format!("{before}\n\n{note}\n")
    });
}

/// Build per-artifact instructions. `Ok(None)` = unknown artifact id; `Err` =
/// a config file exists but cannot be parsed (fail-closed, no default policy).
pub fn build_artifact(
    store: &dyn Store,
    env: &crate::config::EnvOverrides,
    change: &Change,
    schema: &Schema,
    artifact_id: &str,
) -> Result<Option<ArtifactInstructions>, crate::config::ConfigError> {
    let Some(artifact) = schema.artifact(artifact_id) else {
        return Ok(None);
    };
    let wf = WorkflowConfig::from_store(store)?;
    // Policy values come from the three-layer resolution (env > config.yaml >
    // default) — never from one config file alone. The env layer arrives
    // injected from the Host boundary.
    let policy = crate::config::resolve_policy(env, &wf);

    let dependencies = artifact
        .requires
        .iter()
        .filter_map(|dep_id| {
            let da = schema.artifact(dep_id)?;
            Some(Dependency {
                id: da.id.to_string(),
                done: model::artifact_done(store, &change.name, da),
                path: da.output_path.to_string(),
                description: da.description.to_string(),
            })
        })
        .collect();

    // `unlocks` = downstream artifacts for which THIS artifact is the last unmet dependency:
    // not yet done, list this artifact in their `requires`, and have every other requirement done.
    // Empty once this artifact itself is done (it has already unlocked its dependents).
    let self_done = model::artifact_done(store, &change.name, artifact);
    let unlocks: Vec<String> = if self_done {
        Vec::new()
    } else {
        // Listed in display order (topological tier, alphabetical tiebreak) — frozen output shape.
        crate::status::display_order(schema)
            .into_iter()
            .filter(|y| y.id != artifact.id)
            .filter(|y| y.requires.contains(&artifact.id))
            .filter(|y| !model::artifact_done(store, &change.name, y))
            .filter(|y| {
                y.requires.iter().all(|d| {
                    *d == artifact.id
                        || schema
                            .artifact(d)
                            .map(|da| model::artifact_done(store, &change.name, da))
                            .unwrap_or(false)
                })
            })
            .map(|y| y.id.to_string())
            .collect()
    };

    // The specs payload answers "which language?" itself, so the agent never reads a config:
    // `locale` carries the resolved spec language and the instruction always ends by
    // stating it (English included). Every other artifact keeps the project locale.
    let mut instruction = artifact.instruction.clone();
    let mut locale = policy.locale;
    if artifact.id == "specs" {
        let lang = policy.spec_locale.as_deref();
        locale = crate::config::locale_display(lang);
        let cjk_note = if lang.is_some_and(|l| {
            let lower = l.to_ascii_lowercase();
            lower == "tw" || lower.starts_with("zh")
        }) {
            " Vague Chinese wording (應該、可能、也許、或許、大概、考慮、盡量、待定) is \
flagged by the analyzer just like should/may/TBD — state requirements with SHALL/MUST."
        } else {
            ""
        };
        append_paragraph(
            &mut instruction,
            &format!(
                "Write spec prose in {locale}. Structural markers (`## ADDED/MODIFIED/REMOVED/RENAMED \
Requirements`, `### Requirement:`, `#### Scenario:`, `- **WHEN**`/`- **THEN**`) and normative \
keywords (SHALL/MUST/SHALL NOT/MUST NOT) stay in English so the analyzer and archive tooling \
can parse them.{cjk_note}"
            ),
        );
    }
    // Same pattern for the workflow toggles: when the RESOLVED policy enables them, the
    // tasks instruction states the discipline concretely — no new `--json` fields.
    if artifact.id == "tasks" {
        let mut notes = Vec::new();
        if policy.tdd {
            notes.push(
                "This project enables TDD — structure implementation tasks in \
Red-Green-Refactor order: each unit of work starts with a failing-test task, then the \
implementation task that turns it green. The apply step supplies the full TDD discipline; \
you do not need it to write the tasks.",
            );
        }
        if policy.audit {
            notes.push(
                "This project enables the audit discipline — when tasks add APIs, \
configuration options, or parameter handling, include a step to apply the sharp-edges \
audit checklist. The apply step supplies the checklist; you do not need it to write the tasks.",
            );
        }
        for note in notes {
            append_paragraph(&mut instruction, note);
        }
    }

    Ok(Some(ArtifactInstructions {
        change_name: change.name.clone(),
        artifact_id: artifact.id.clone(),
        schema_name: schema.display_name.clone(),
        change_dir: change.dir.to_string_lossy().to_string(),
        output_path: artifact.output_path.clone(),
        description: artifact.description.clone(),
        instruction,
        context: wf.context_text(),
        rules: wf.rules_for(&artifact.id),
        locale,
        // The payload template is looked up in the BUILT-IN schema matching the yaml display
        // name — never in the custom templates/ file (which only
        // `new artifact` reads). A custom display name therefore yields an empty template.
        template: if schema.is_builtin() {
            artifact.template.clone().unwrap_or_default()
        } else {
            crate::schema::builtin_template(&schema.display_name, &artifact.id).unwrap_or_default()
        },
        dependencies,
        unlocks,
    }))
}

#[derive(Debug, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub total: usize,
    pub complete: usize,
    pub remaining: usize,
    pub code_total: usize,
    pub code_complete: usize,
    pub code_remaining: usize,
}

impl From<&tasks::Counts> for Progress {
    fn from(c: &tasks::Counts) -> Self {
        Progress {
            total: c.total,
            complete: c.complete,
            remaining: c.remaining,
            code_total: c.code_total,
            code_complete: c.code_complete,
            code_remaining: c.code_remaining,
        }
    }
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct TaskJson {
    pub id: String,
    pub description: String,
    pub done: bool,
    pub manual: bool,
}

impl From<&Task> for TaskJson {
    fn from(t: &Task) -> Self {
        TaskJson {
            id: t.id.to_string(),
            description: t.description.clone(),
            done: t.done,
            manual: t.manual,
        }
    }
}

/// Apply-mode instructions payload (frozen field order).
#[derive(Debug, Serialize, serde::Deserialize)]
pub struct ApplyInstructions {
    #[serde(rename = "changeName")]
    pub change_name: String,
    #[serde(rename = "changeDir")]
    pub change_dir: String,
    #[serde(rename = "schemaName")]
    pub schema_name: String,
    #[serde(rename = "contextFiles")]
    pub context_files: std::collections::BTreeMap<String, String>,
    pub progress: Progress,
    pub tasks: Vec<TaskJson>,
    pub state: String,
    #[serde(rename = "missingArtifacts", skip_serializing_if = "Option::is_none")]
    pub missing_artifacts: Option<Vec<String>>,
    pub locale: String,
    /// Effective policy toggles from the three-layer resolution — the apply skill
    /// reads its TDD/audit discipline switches from the payload, never from a
    /// config file of its own.
    pub tdd: bool,
    pub audit: bool,
    /// Omitted when the (custom) schema defines no apply instruction (frozen output shape).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preflight: Option<Preflight>,
}

/// Remote mode retargets apply `contextFiles` onto the Context Projection
/// mirror: the key set and the server's
/// collection logic stay untouched — every value becomes the corresponding
/// path under the projection's spec-root mirror, so skills read the
/// read-only projection instead of unreachable server paths. Never called
/// on the local fs path, whose output stays byte-identical.
pub fn project_context_files(
    context_files: &mut std::collections::BTreeMap<String, String>,
    projection_spec_root: &Path,
    change_name: &str,
) {
    let change_dir = projection_spec_root.join("changes").join(change_name);
    for (key, value) in context_files.iter_mut() {
        *value = match key.as_str() {
            // The spec-driven artifact locations build_apply emits.
            "proposal" => join_display(&change_dir, "proposal.md"),
            "design" => join_display(&change_dir, "design.md"),
            "tasks" => join_display(&change_dir, "tasks.md"),
            "specs" => join_display(&change_dir, "specs/**/*.md"),
            // Unknown keys keep the server's spec-root-relative value,
            // re-rooted under the projection mirror.
            _ => join_display(projection_spec_root, value),
        };
    }
}

/// Compute apply state: blocked | ready | all_done.
pub fn apply_state(schema: &Schema, store: &dyn Store, change: &Change, tasks: &[Task]) -> String {
    let tasks_artifact = schema.artifact("tasks");
    let tasks_done = tasks_artifact
        .map(|a| model::artifact_done(store, &change.name, a))
        .unwrap_or(false);
    if !tasks_done || tasks.is_empty() {
        return "blocked".to_string();
    }
    let (total, complete, _) = tasks::progress(tasks);
    if total > 0 && complete == total {
        "all_done".to_string()
    } else {
        "ready".to_string()
    }
}

pub fn build_apply(
    ws: &Workspace,
    store: &dyn Store,
    env: &crate::config::EnvOverrides,
    change: &Change,
    schema: &Schema,
) -> Result<ApplyInstructions, crate::config::ConfigError> {
    let wf = WorkflowConfig::from_store(store)?;
    let policy = crate::config::resolve_policy(env, &wf);
    let tasks_md = store.read_artifact(&change.name, "tasks.md").unwrap_or_default();
    let parsed = tasks::parse(&tasks_md);
    let counts = tasks::counts(&parsed);

    // contextFiles includes artifacts whose files exist (empty files count) — frozen output shape.
    let mut context_files = std::collections::BTreeMap::new();
    if store.artifact_exists(&change.name, "proposal.md") {
        context_files.insert("proposal".to_string(), join_display(&change.dir, "proposal.md"));
    }
    if !store.delta_capabilities(&change.name).is_empty() {
        context_files.insert("specs".to_string(), join_display(&change.dir, "specs/**/*.md"));
    }
    if store.artifact_exists(&change.name, "design.md") {
        context_files.insert("design".to_string(), join_display(&change.dir, "design.md"));
    }
    if store.artifact_exists(&change.name, "tasks.md") {
        context_files.insert("tasks".to_string(), join_display(&change.dir, "tasks.md"));
    }

    let state = apply_state(schema, store, change, &parsed);
    let blocked = state == "blocked";

    let missing_artifacts = if blocked {
        let missing: Vec<String> = schema
            .apply_requires
            .iter()
            .filter(|id| {
                schema
                    .artifact(id)
                    .map(|a| !model::artifact_done(store, &change.name, a))
                    .unwrap_or(true)
            })
            .map(|s| s.to_string())
            .collect();
        // Blocked by zero checkboxes rather than a missing artifact → the key is
        // omitted entirely (frozen output shape: no empty missingArtifacts array).
        if missing.is_empty() {
            None
        } else {
            Some(missing)
        }
    } else {
        None
    };
    // Preflight only in the ready state — the field drops out again once all tasks are done.
    let preflight = if state == "ready" {
        Some(Preflight::compute(ws, store, change))
    } else {
        None
    };

    Ok(ApplyInstructions {
        change_name: change.name.clone(),
        change_dir: change.dir.to_string_lossy().to_string(),
        schema_name: schema.display_name.clone(),
        context_files,
        progress: Progress::from(&counts),
        tasks: parsed.iter().map(TaskJson::from).collect(),
        state,
        missing_artifacts,
        locale: policy.locale,
        tdd: policy.tdd,
        audit: policy.audit,
        instruction: schema.apply_instruction.clone(),
        preflight,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- remote contextFiles 重定向：key 與集合邏輯不變、值指向投影鏡像 ---

    #[test]
    fn project_context_files_retargets_values_and_keeps_keys() {
        let mut files = std::collections::BTreeMap::from([
            ("design".to_string(), "changes/demo/design.md".to_string()),
            ("proposal".to_string(), "changes/demo/proposal.md".to_string()),
            ("specs".to_string(), "changes/demo/specs/**/*.md".to_string()),
            ("tasks".to_string(), "changes/demo/tasks.md".to_string()),
            ("extra".to_string(), "changes/demo/extra.md".to_string()),
        ]);
        let root = Path::new("/ws/.speclink/context/openspec");
        project_context_files(&mut files, root, "demo");

        // key 集合不變。
        let keys: Vec<&str> = files.keys().map(String::as_str).collect();
        assert_eq!(keys, ["design", "extra", "proposal", "specs", "tasks"]);

        // 已知 key 的值 = 投影鏡像下該 change 的對應路徑。
        let change_dir = root.join("changes").join("demo");
        assert_eq!(files["proposal"], change_dir.join("proposal.md").to_string_lossy());
        assert_eq!(files["design"], change_dir.join("design.md").to_string_lossy());
        assert_eq!(files["tasks"], change_dir.join("tasks.md").to_string_lossy());
        assert_eq!(files["specs"], change_dir.join("specs/**/*.md").to_string_lossy());
        // 未知 key 保留 server 相對值、改掛投影鏡像根下。
        assert_eq!(files["extra"], root.join("changes/demo/extra.md").to_string_lossy());

        // 每個值都在投影下。
        for v in files.values() {
            assert!(v.starts_with(&root.to_string_lossy().to_string()), "{v} under projection");
        }
    }

    // --- spec「任務 payload 的 manual 欄位與寫碼進度」---

    #[test]
    fn task_json_carries_the_manual_flag() {
        let parsed = tasks::parse("- [ ] [M] 手測匯入\n- [x] [P] 寫解析器\n");
        let json: Vec<TaskJson> = parsed.iter().map(TaskJson::from).collect();
        assert!(json[0].manual, "[M] 任務 manual=true");
        assert_eq!(json[0].description, "手測匯入", "描述不含標記");
        assert!(!json[1].manual, "[P] 任務 manual=false");
        assert_eq!(json[1].description, "寫解析器", "[P] 前綴須剝除");
    }

    #[test]
    fn task_json_field_set_is_frozen_without_parallel() {
        // verb-contract「動詞 --json 輸出形狀凍結」：任務項欄位集合為 id/description/done/manual
        let json = TaskJson::from(&tasks::parse("- [ ] [M] 手測匯入\n")[0]);
        let value = serde_json::to_value(&json).expect("TaskJson 可序列化");
        let mut keys: Vec<&str> =
            value.as_object().expect("物件").keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, ["description", "done", "id", "manual"], "欄位集合凍結、無 parallel");
    }

    #[test]
    fn progress_carries_code_counts_beside_the_full_ones() {
        // 九個已勾寫碼任務 + 一個未勾 [M]：total=10/complete=9/remaining=1，code 三欄 9/9/0。
        let mut md = String::new();
        for i in 1..=9 {
            md.push_str(&format!("- [x] task {i}\n"));
        }
        md.push_str("- [ ] [M] 手測\n");
        let p = Progress::from(&tasks::counts(&tasks::parse(&md)));
        assert_eq!((p.total, p.complete, p.remaining), (10, 9, 1));
        assert_eq!((p.code_total, p.code_complete, p.code_remaining), (9, 9, 0));
    }

    #[test]
    fn code_counts_mirror_full_counts_without_manual_tasks() {
        let p = Progress::from(&tasks::counts(&tasks::parse("- [x] a\n- [ ] b\n")));
        assert_eq!((p.code_total, p.code_complete, p.code_remaining), (2, 1, 1));
        assert_eq!((p.total, p.complete, p.remaining), (2, 1, 1));
    }

    // --- spec「instructions 對 specs 產出物回報解析後的規格語言」---

    use crate::config::EnvOverrides;
    use crate::teststore::TestStore;

    const META: &str = "schema: spec-driven\ncreated: 2026-11-24\n";
    const TW: &str = "Traditional Chinese (繁體中文)";
    const JA: &str = "Japanese (日本語)";
    const EN: &str = "English";

    fn built(config: &str, env: &EnvOverrides, schema: &Schema, artifact: &str) -> ArtifactInstructions {
        let store = TestStore::with_meta("demo", META);
        *store.workflow_config.borrow_mut() = Some(config.to_string());
        let change = store.find_change("demo").expect("change listed");
        build_artifact(&store, env, &change, schema, artifact)
            .expect("config parses")
            .expect("artifact known")
    }

    fn last_paragraph(instruction: &str) -> &str {
        instruction.rsplit("\n\n").next().unwrap_or(instruction)
    }

    #[test]
    fn specs_locale_reports_the_resolved_spec_language() {
        // Example「規格語言的解析」每一列：(locale, spec_locale, SPECLINK_SPEC_LOCALE, specs, proposal)。
        let rows: [(Option<&str>, Option<&str>, Option<&str>, &str, &str); 7] = [
            (Some("tw"), None, None, EN, TW),
            (Some("tw"), Some("en"), None, EN, TW),
            (Some("tw"), Some("auto"), None, TW, TW),
            (Some("tw"), Some("ja"), None, JA, TW),
            (None, Some("auto"), None, EN, EN),
            (None, Some("tw"), None, TW, EN),
            (Some("tw"), Some("en"), Some("ja"), JA, TW),
        ];
        let schema = crate::schema::spec_driven();
        for (i, (locale, spec_locale, env_spec, want_specs, want_proposal)) in rows.iter().enumerate() {
            let mut config = String::from("schema: spec-driven\n");
            if let Some(l) = locale {
                config.push_str(&format!("locale: {l}\n"));
            }
            if let Some(s) = spec_locale {
                config.push_str(&format!("spec_locale: {s}\n"));
            }
            let env = EnvOverrides {
                spec_locale: env_spec.map(str::to_string),
                ..EnvOverrides::default()
            };
            let specs = built(&config, &env, &schema, "specs");
            assert_eq!(specs.locale, *want_specs, "row {i}: specs 的 locale");
            let proposal = built(&config, &env, &schema, "proposal");
            assert_eq!(proposal.locale, *want_proposal, "row {i}: proposal 的 locale");

            let instruction = specs.instruction.expect("specs 帶 instruction");
            let last = last_paragraph(&instruction);
            assert!(
                last.starts_with(&format!("Write spec prose in {want_specs}.")),
                "row {i}: 最後一段寫明規格語言: {last}"
            );
            for marker in ["### Requirement:", "#### Scenario:", "SHALL/MUST/SHALL NOT/MUST NOT"] {
                assert!(last.contains(marker), "row {i}: 最後一段點名 {marker}: {last}");
            }
            assert_eq!(
                last.contains("Vague Chinese wording"),
                *want_specs == TW,
                "row {i}: 中文模糊用語的提醒只在 tw: {last}"
            );
            for banned in ["config.yaml", "English by default", "spec_locale"] {
                assert!(!instruction.contains(banned), "row {i}: instruction 不含 {banned}");
            }
        }
    }

    #[test]
    fn specs_without_schema_instruction_get_only_the_language_paragraph() {
        let mut schema = crate::schema::spec_driven();
        let specs = schema.artifacts.iter_mut().find(|a| a.id == "specs").expect("specs");
        specs.instruction = None;
        let got = built("locale: tw\n", &EnvOverrides::default(), &schema, "specs");
        let instruction = got.instruction.expect("引擎補上語言段落");
        assert!(instruction.starts_with("Write spec prose in English."), "{instruction}");
        assert!(!instruction.contains("\n\n"), "只有這一段: {instruction}");
        assert!(instruction.ends_with("can parse them.\n"), "和 schema 的 instruction 一樣以換行結尾: {instruction:?}");
    }

    /// 引擎補的段落和前文之間剛好空一行，整份 instruction 仍以一個換行結尾——
    /// 人眼輸出的 renderer 依賴這個結尾。
    #[test]
    fn appended_paragraphs_follow_one_blank_line_and_end_with_a_newline() {
        let schema = crate::schema::spec_driven();
        let appended = |config: &str, artifact: &str| {
            let raw = schema.artifact(artifact).and_then(|a| a.instruction.clone()).expect("schema 原稿");
            let got = built(config, &EnvOverrides::default(), &schema, artifact).instruction.expect("instruction");
            got.strip_prefix(raw.trim_end()).unwrap_or_else(|| panic!("以 schema 原稿開頭: {got:?}")).to_string()
        };

        let specs = appended("locale: tw\n", "specs");
        assert!(specs.starts_with("\n\nWrite spec prose in English."), "{specs:?}");
        assert!(specs.ends_with("can parse them.\n") && !specs.ends_with("\n\n"), "{specs:?}");

        assert_eq!(
            appended("tdd: true\naudit: true\n", "tasks"),
            "\n\nThis project enables TDD — structure implementation tasks in Red-Green-Refactor order: \
each unit of work starts with a failing-test task, then the implementation task that turns it green. \
The apply step supplies the full TDD discipline; you do not need it to write the tasks.\
\n\nThis project enables the audit discipline — when tasks add APIs, configuration options, or \
parameter handling, include a step to apply the sharp-edges audit checklist. The apply step supplies \
the checklist; you do not need it to write the tasks.\n"
        );
    }

    /// 讀取端對代碼寬鬆：`zh` 開頭的代碼（不分大小寫）也帶中文模糊用語的提醒；
    /// 環境變數給 `auto` 時跟著 locale。
    #[test]
    fn spec_language_note_covers_zh_codes_and_an_auto_override() {
        let schema = crate::schema::spec_driven();
        for code in ["zh-Hant", "ZH-Hant"] {
            let specs = built(&format!("spec_locale: {code}\n"), &EnvOverrides::default(), &schema, "specs");
            let instruction = specs.instruction.expect("instruction");
            assert!(last_paragraph(&instruction).contains("Vague Chinese wording"), "{code}: {instruction}");
        }
        let env = EnvOverrides { spec_locale: Some("auto".to_string()), ..EnvOverrides::default() };
        let specs = built("locale: tw\nspec_locale: en\n", &env, &schema, "specs");
        assert_eq!(specs.locale, TW, "SPECLINK_SPEC_LOCALE=auto 跟著 locale");
    }

    // --- spec「tasks 指示的紀律說明不指向帶 --skill 的 instructions」---

    #[test]
    fn tasks_discipline_notes_do_not_point_at_skill_instructions() {
        let schema = crate::schema::spec_driven();
        let tasks = |config: &str| {
            built(config, &EnvOverrides::default(), &schema, "tasks")
                .instruction
                .expect("tasks 帶 instruction")
        };

        let tdd = tasks("tdd: true\n");
        assert!(tdd.contains("Red-Green-Refactor"), "{tdd}");
        assert!(tdd.contains("The apply step supplies the full TDD discipline"), "{tdd}");
        assert!(!tdd.contains("instructions --skill"), "{tdd}");

        let audit = tasks("audit: true\n");
        assert!(audit.contains("sharp-edges audit checklist"), "{audit}");
        assert!(audit.contains("The apply step supplies the checklist"), "{audit}");
        assert!(!audit.contains("instructions --skill"), "{audit}");

        let raw = schema.artifact("tasks").and_then(|a| a.instruction.clone());
        assert_eq!(Some(tasks("schema: spec-driven\n")), raw, "兩個開關都關閉時逐字等於 schema 原稿");
    }

    #[test]
    fn apply_payload_serializes_new_fields_in_camel_case() {
        let p = Progress::from(&tasks::counts(&tasks::parse("- [x] a\n- [ ] [M] m\n")));
        let v = serde_json::to_value(&p).expect("progress serializes");
        assert_eq!(v["codeTotal"], 1);
        assert_eq!(v["codeComplete"], 1);
        assert_eq!(v["codeRemaining"], 0);
        assert_eq!(v["total"], 2, "既有欄位不變");

        let t = TaskJson::from(&tasks::parse("- [ ] [M] m\n")[0]);
        assert_eq!(serde_json::to_value(&t).expect("task serializes")["manual"], true);
    }
}
