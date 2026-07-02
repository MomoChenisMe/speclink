// Included into main.rs. Command handlers and rendering.

use core::model::Change;
use core::paths::Paths;
use core::schema::Schema;

fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init(a) => cmd_init(a),
        Commands::Update(a) => cmd_update(a),
        Commands::List(a) => cmd_list(a),
        Commands::Show(a) => cmd_show(a),
        Commands::Validate(a) => cmd_validate(a),
        Commands::Analyze(a) => cmd_analyze(a),
        Commands::Drift(a) => cmd_drift(a),
        Commands::Archive(a) => cmd_archive(a),
        Commands::Status(a) => cmd_status(a),
        Commands::Instructions(a) => cmd_instructions(a),
        Commands::New(a) => cmd_new(a),
        Commands::Schemas(a) => cmd_schemas(a),
        Commands::Templates(a) => cmd_templates(a),
        Commands::Feedback(a) => cmd_feedback(a),
        Commands::Schema(a) => cmd_schema(a),
        Commands::Config(a) => cmd_config(a),
        Commands::Completion(a) => cmd_completion(a),
        Commands::Task(a) => cmd_task(a),
        Commands::InProgress(a) => cmd_in_progress(a),
        Commands::Demo => cmd_demo(),
        Commands::Discuss(a) => cmd_discuss(a),
    }
}

// --- helpers ---

fn resolve_change(paths: &Paths, name: Option<&str>) -> Result<Change> {
    resolve_change_worded(paths, name, "Use --change to specify one:")
}

/// Positional-style resolution (analyze/drift): Spectra says just "Specify one:".
fn resolve_change_positional(paths: &Paths, name: Option<&str>) -> Result<Change> {
    resolve_change_worded(paths, name, "Specify one:")
}

fn resolve_change_worded(paths: &Paths, name: Option<&str>, specify: &str) -> Result<Change> {
    if let Some(n) = name {
        return core::model::find_change(paths, n)
            .ok_or_else(|| anyhow::anyhow!("Change '{n}' not found."));
    }
    let mut changes = core::model::list_changes(paths);
    match changes.len() {
        0 => bail!("No active changes. Create one with: speclink new change <name>"),
        1 => Ok(changes.remove(0)),
        _ => {
            // Spectra lists the changes by most-recently-modified first.
            changes.sort_by(|a, b| {
                let ma = std::fs::metadata(&a.dir).and_then(|m| m.modified()).ok();
                let mb = std::fs::metadata(&b.dir).and_then(|m| m.modified()).ok();
                mb.cmp(&ma)
            });
            let names: Vec<&str> = changes.iter().map(|c| c.name.as_str()).collect();
            bail!("Multiple changes found. {specify} {}", names.join(", "))
        }
    }
}

/// For read/analysis commands: when no change name is given and no changes exist, print the
/// informational message and signal exit-0 (returns true = handled).
fn info_if_no_changes(paths: &Paths, name: Option<&str>) -> bool {
    if name.is_none() && core::model::list_changes(paths).is_empty() {
        println!("No active changes. Create one with: speclink new change <name>");
        true
    } else {
        false
    }
}

fn schema_for(change: &Change) -> Schema {
    core::schema::resolve(&change.meta.schema_name()).unwrap_or_else(core::schema::spec_driven)
}

fn truncate_summary(text: &str, limit: usize) -> String {
    let first_line = text.trim();
    if first_line.chars().count() <= limit {
        return first_line.to_string();
    }
    // Take the first `limit` characters verbatim (no word-boundary, no trim) and append an ellipsis.
    let head: String = first_line.chars().take(limit).collect();
    format!("{head}…")
}

fn proposal_summary(change: &Change) -> String {
    let proposal = core::util::read_opt(&change.dir.join("proposal.md")).unwrap_or_default();
    // First non-empty, non-header line after "## Why" (or first prose line).
    let mut after_why = false;
    for line in proposal.lines() {
        let t = line.trim();
        if t.starts_with("## Why") {
            after_why = true;
            continue;
        }
        if after_why && !t.is_empty() && !t.starts_with('#') {
            return truncate_summary(t, 30);
        }
    }
    // Fallback: first prose line.
    for line in proposal.lines() {
        let t = line.trim();
        if !t.is_empty() && !t.starts_with('#') && !t.starts_with("<!--") {
            return truncate_summary(t, 30);
        }
    }
    String::new()
}

fn task_counts(change: &Change) -> (usize, usize) {
    let tasks_md = core::util::read_opt(&change.dir.join("tasks.md")).unwrap_or_default();
    let tasks = core::tasks::parse(&tasks_md);
    let (total, complete, _) = core::tasks::progress(&tasks);
    (complete, total)
}

// --- init / update ---

fn cmd_init(a: InitArgs) -> Result<()> {
    let root = a
        .path
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let root = if root.is_absolute() {
        root
    } else {
        std::env::current_dir()?.join(root)
    };
    let spec_dir = a.dir.clone().unwrap_or_else(|| "openspec".to_string());
    let tools = match a.tools.as_deref() {
        Some(spec) => core::init::parse_tools(spec)?,
        None => Vec::new(),
    };
    let outcome = core::init::init(&root, &tools, a.force, &spec_dir)?;
    println!("✓ Initialized at {}", core::util::to_slash(&outcome.spec_dir_abs));
    Ok(())
}

fn cmd_update(a: UpdateArgs) -> Result<()> {
    let root = a
        .path
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let root = if root.is_absolute() {
        root
    } else {
        std::env::current_dir()?.join(root)
    };
    if !root.join(".speclink.yaml").is_file() && !root.join("openspec").is_dir() {
        bail!("Not initialized. Run 'speclink init' to initialize.");
    }
    core::init::update(&root, a.force)?;
    println!("✓ Updated instruction files");
    Ok(())
}

// --- list ---

#[derive(serde::Serialize)]
struct ListChangeJson {
    #[serde(rename = "completedTasks")]
    completed_tasks: usize,
    name: String,
    status: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    summary: String,
    #[serde(rename = "totalTasks")]
    total_tasks: usize,
}

/// Order changes for `list`. Spectra lists alphabetically by name (both by default and with
/// `--sort name`).
fn sort_changes(changes: &mut [Change], _sort: &str) {
    changes.sort_by(|x, y| x.name.cmp(&y.name));
}

fn cmd_list(a: ListArgs) -> Result<()> {
    let paths = require_paths()?;
    if a.specs {
        return list_specs(&paths, a.json);
    }
    let mut changes = core::model::list_changes(&paths);
    sort_changes(&mut changes, &a.sort);
    if a.json {
        let items: Vec<ListChangeJson> = changes
            .iter()
            .map(|c| {
                let (complete, total) = task_counts(c);
                ListChangeJson {
                    completed_tasks: complete,
                    name: c.name.clone(),
                    status: "in-progress".to_string(),
                    summary: proposal_summary(c),
                    total_tasks: total,
                }
            })
            .collect();
        return print_json(&serde_json::json!({ "changes": items }));
    }
    if changes.is_empty() {
        println!("No active changes.");
        return Ok(());
    }
    println!("Changes:");
    for c in &changes {
        let (complete, total) = task_counts(c);
        let summary = proposal_summary(c);
        // Spectra omits the progress marker entirely for changes with zero tasks.
        let marker = if total > 0 {
            format!(" [{complete}/{total}]")
        } else {
            String::new()
        };
        if summary.is_empty() {
            println!("  • {}{marker}", c.name);
        } else {
            println!("  • {}{marker} — {summary}", c.name);
        }
    }
    Ok(())
}

fn list_specs(paths: &Paths, json: bool) -> Result<()> {
    let mut specs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(paths.specs_dir()) {
        for e in entries.flatten() {
            if e.path().is_dir() && e.path().join("spec.md").is_file() {
                specs.push(e.file_name().to_string_lossy().to_string());
            }
        }
    }
    specs.sort();
    if json {
        let items: Vec<_> = specs
            .iter()
            .map(|s| {
                serde_json::json!({
                    "id": s,
                    "path": paths.specs_dir().join(s).to_string_lossy(),
                })
            })
            .collect();
        return print_json(&serde_json::json!({ "specs": items }));
    }
    if specs.is_empty() {
        println!("No specs found.");
        return Ok(());
    }
    println!("Specs:");
    for s in specs {
        println!("  • {s}");
    }
    Ok(())
}

// --- show ---

fn cmd_show(a: ShowArgs) -> Result<()> {
    let paths = require_paths()?;
    let item = match a.item {
        Some(n) => n,
        None => bail!("Please specify an item name."),
    };
    let item_type = a.item_type.as_deref();

    let spec_md = paths.specs_dir().join(&item).join("spec.md");
    let is_spec = spec_md.is_file();
    let change = core::model::find_change(&paths, &item);

    // Decide whether the item is a spec or a change.
    let show_spec = item_type == Some("spec")
        || (item_type != Some("change") && change.is_none() && is_spec);

    if show_spec {
        if !is_spec {
            if item_type == Some("spec") {
                bail!("Spec '{item}' not found.");
            }
            bail!("Item '{item}' not found as a change or spec.");
        }
        let content = core::util::read_opt(&spec_md).unwrap_or_default();
        if a.json {
            return print_json(&serde_json::json!({
                "files": [{ "content": content, "name": "spec.md" }],
                "name": item,
            }));
        }
        println!("Spec: {item}");
        println!();
        println!("--- spec.md ---");
        print!("{content}");
        println!(); // Spectra always emits a trailing newline (an extra blank when content ends with \n)
        return Ok(());
    }

    let Some(change) = change else {
        if item_type == Some("change") {
            bail!("Change '{item}' not found.");
        }
        bail!("Item '{item}' not found as a change or spec.");
    };
    let schema = schema_for(&change);
    let read_opt_str = |name: &str| core::util::read_opt(&change.dir.join(name));

    if a.json {
        let proposal = read_opt_str("proposal.md");
        let design = read_opt_str("design.md");
        let tasks = read_opt_str("tasks.md");
        let caps: Vec<String> = core::model::delta_capabilities(&change.dir)
            .into_iter()
            .map(|c| format!("{c}/spec.md"))
            .collect();
        return print_json(&serde_json::json!({
            "name": change.name,
            "schema": schema.name,
            "created": change.meta.created,
            "proposal": proposal,
            "design": design,
            "tasks": tasks,
            "deltaSpecs": caps,
        }));
    }

    println!("Change: {}", change.name);
    println!("Schema: {}", schema.name);
    if let Some(created) = &change.meta.created {
        println!("Created: {created}");
    }
    println!();
    let proposal = read_opt_str("proposal.md").unwrap_or_default();
    let caps = core::model::delta_capabilities(&change.dir);
    if !proposal.trim().is_empty() {
        println!("--- Proposal ---");
        print!("{proposal}");
        if !caps.is_empty() {
            // The proposal's own trailing newline determines the blank-line count before the header.
            print!("\n\n--- Delta Specs ---\n");
            for c in &caps {
                println!("  {c}/spec.md");
            }
        } else if !proposal.ends_with('\n') {
            println!();
        }
    } else if !caps.is_empty() {
        // No proposal, but delta specs exist — still render the section.
        println!("--- Delta Specs ---");
        for c in &caps {
            println!("  {c}/spec.md");
        }
    }
    Ok(())
}

// --- validate ---

fn cmd_validate(a: ValidateArgs) -> Result<()> {
    let paths = require_paths()?;
    let changes = if let Some(item) = a.item.as_deref() {
        vec![core::model::find_change(&paths, item)
            .ok_or_else(|| anyhow::anyhow!("Change '{item}' not found."))?]
    } else if a.all || a.changes {
        core::model::list_changes(&paths)
    } else {
        match resolve_change(&paths, None) {
            Ok(c) => vec![c],
            Err(_) => core::model::list_changes(&paths),
        }
    };
    let results: Vec<_> = changes
        .iter()
        .map(|c| core::validate::validate_change(c, &schema_for(c), a.strict))
        .collect();
    if a.json {
        return print_json(&results);
    }
    for r in &results {
        if r.valid {
            println!("✓ {} — valid", r.change);
        } else {
            println!("✗ {} — {} error(s)", r.change, r.errors.len());
            for e in &r.errors {
                println!("  - {e}");
            }
        }
        for w in &r.warnings {
            println!("  warn: {w}");
        }
    }
    Ok(())
}

// --- analyze ---

fn cmd_analyze(a: ChangeArg) -> Result<()> {
    let paths = require_paths()?;
    if info_if_no_changes(&paths, a.change.as_deref()) {
        return Ok(());
    }
    let change = resolve_change_positional(&paths, a.change.as_deref())?;
    let schema = schema_for(&change);
    let report = core::analyzer::analyze(&change, &schema);
    if a.json {
        return print_json(&report);
    }
    render_analyze(&report);
    Ok(())
}

fn render_analyze(report: &core::analyzer::AnalyzeReport) {
    println!("Change: {}", report.change_id);
    println!();
    for d in &report.dimensions {
        let sym = if d.finding_count == 0 { "✓" } else { "●" };
        println!(
            "  {sym} {:<15}{} ({} findings)",
            d.dimension, d.status, d.finding_count
        );
    }
    // The blank separator is tied to the "Analyzed:" line; an empty change (nothing analyzed)
    // prints "Missing:" directly after the dimensions, matching Spectra.
    if !report.artifacts_analyzed.is_empty() {
        println!();
        println!("  Analyzed: {}", report.artifacts_analyzed.join(", "));
    }
    if !report.artifacts_missing.is_empty() {
        println!("  Missing: {}", report.artifacts_missing.join(", "));
    }
    if report.findings.is_empty() {
        println!();
        println!("  ✓ No issues found");
        return;
    }
    println!();
    println!("  Findings ({}):", report.findings.len());
    println!();
    for f in &report.findings {
        let tag = match f.severity.as_str() {
            "Critical" => "CRITICAL",
            "Warning" => "WARNING",
            _ => "SUGGEST",
        };
        println!("  [{tag}] {}", f.summary);
        println!("    at: {}", f.location);
        println!("    → {}", f.recommendation);
    }
}

// --- drift ---

fn cmd_drift(a: ChangeArg) -> Result<()> {
    let paths = require_paths()?;
    if info_if_no_changes(&paths, a.change.as_deref()) {
        return Ok(());
    }
    let change = resolve_change_positional(&paths, a.change.as_deref())?;
    let report = core::drift::analyze(&paths, &change);
    if a.json {
        return print_json(&report);
    }
    render_drift(&report);
    Ok(())
}

fn render_drift(report: &core::drift::DriftReport) {
    println!("Drift Report: {}", report.change_id);
    if let Some(created) = &report.created {
        println!("  Created: {created}");
    }
    println!();
    println!("  {:<11} {:<36} {}", "Dimension", "Status", "Score");
    for d in &report.dimensions {
        let score = if d.contributes_to_total {
            format!("+{}", d.score)
        } else {
            "—".to_string()
        };
        println!("  {:<11} {:<36} {:>5}", d.kind, d.status, score);
    }
    println!("  {:<11} {:<36} {:>5}", "Total", "", report.total_score);
    if !report.broken_anchors.is_empty() {
        println!();
        println!("Broken anchors");
        for b in &report.broken_anchors {
            println!("  - {} ({}) — {}", b.anchor, b.category, b.reason);
        }
    }
    println!();
    println!("Severity: {} drift", report.severity.to_uppercase());
    println!("> {}", report.primary_recommendation);
}

// --- archive ---

fn cmd_archive(a: ArchiveArgs) -> Result<()> {
    let paths = require_paths()?;
    let change = resolve_change(&paths, a.change.as_deref())?;

    if a.mark_tasks_complete {
        let tasks_path = change.dir.join("tasks.md");
        if let Some(text) = core::util::read_opt(&tasks_path) {
            let done = text
                .replace("- [ ] ", "- [x] ")
                .replace("- [ ]\t", "- [x]\t");
            core::util::write_file(&tasks_path, &done)?;
        }
    }

    let opts = core::archive::ArchiveOptions {
        skip_specs: a.skip_specs,
        no_validate: a.no_validate,
        mark_tasks_complete: a.mark_tasks_complete,
    };
    let outcome = core::archive::archive(&paths, &change, &opts)?;
    let _ = core::inprogress::remove(&paths, &change.name);

    println!("✓ Archived: {} → {}", outcome.change_name, outcome.dated_name);
    if !outcome.caps.is_empty() {
        let names: Vec<&str> = outcome.caps.iter().map(|c| c.capability.as_str()).collect();
        let added: usize = outcome.caps.iter().map(|c| c.added).sum();
        let modified: usize = outcome.caps.iter().map(|c| c.modified).sum();
        let removed: usize = outcome.caps.iter().map(|c| c.removed).sum();
        let renamed: usize = outcome.caps.iter().map(|c| c.renamed).sum();
        println!(
            "Specs applied: {} (added: {added}, modified: {modified}, removed: {removed}, renamed: {renamed})",
            names.join(", ")
        );
    }
    if outcome.snapshot_created && !outcome.skipped_specs && !outcome.caps.is_empty() {
        println!("Snapshot created for unarchive support.");
    }
    Ok(())
}

// --- status ---

fn cmd_status(a: StatusArgs) -> Result<()> {
    let paths = require_paths()?;
    if info_if_no_changes(&paths, a.change.as_deref()) {
        return Ok(());
    }
    let change = resolve_change(&paths, a.change.as_deref())?;
    let schema = if let Some(s) = a.schema.as_deref() {
        core::schema::resolve(s).ok_or_else(|| anyhow::anyhow!("unknown schema: {s}"))?
    } else {
        schema_for(&change)
    };
    let report = core::status::build(&change, &schema);
    if a.json {
        return print_json(&report);
    }
    println!("Change: {}", report.change_name);
    println!("Schema: {}", report.schema_name);
    println!();
    for art in &report.artifacts {
        let sym = match art.status.as_str() {
            "done" => "✓",
            "ready" => "○",
            _ => "✗",
        };
        println!("  {sym} {} ({})", art.id, art.output_path);
        if art.status == "blocked" && !art.blocked_by.is_empty() {
            println!("    blocked by: {}", art.blocked_by.join(", "));
        }
    }
    println!();
    if report.is_complete {
        println!("  ✓ All artifacts complete");
    }
    Ok(())
}

// --- instructions ---

fn cmd_instructions(a: InstructionsArgs) -> Result<()> {
    if let Some(skill) = a.skill.as_deref() {
        let body = core::skills::skill_body(skill)
            .ok_or_else(|| anyhow::anyhow!("Unknown skill: {skill}"))?;
        print!("{body}");
        return Ok(());
    }
    let paths = require_paths()?;
    if info_if_no_changes(&paths, a.change.as_deref()) {
        return Ok(());
    }
    let change = resolve_change(&paths, a.change.as_deref())?;
    let schema = if let Some(s) = a.schema.as_deref() {
        core::schema::resolve(s).ok_or_else(|| anyhow::anyhow!("unknown schema: {s}"))?
    } else {
        schema_for(&change)
    };
    let default_artifact = core::status::first_incomplete_artifact(&change, &schema)
        .unwrap_or_else(|| "proposal".to_string());
    let artifact = a.artifact.as_deref().unwrap_or(&default_artifact);
    if artifact == "apply" {
        let payload = core::instructions::build_apply(&paths, &change, &schema);
        if a.json {
            return print_json(&payload);
        }
        render_apply_human(&payload);
        return Ok(());
    }
    let payload = core::instructions::build_artifact(&paths, &change, &schema, artifact)
        .ok_or_else(|| anyhow::anyhow!("Artifact '{artifact}' not found in schema"))?;
    if a.json {
        return print_json(&payload);
    }
    render_artifact_human(&payload);
    Ok(())
}

fn render_artifact_human(p: &core::instructions::ArtifactInstructions) {
    println!("Artifact: {}", p.artifact_id);
    println!("Output: {}", p.output_path);
    println!("Description: {}", p.description);
    println!();
    println!("Instruction:");
    print!("{}", p.instruction); // ends with a newline
    println!();
    println!();
    if !p.dependencies.is_empty() {
        println!("Dependencies:");
        for d in &p.dependencies {
            let sym = if d.done { "✓" } else { "○" };
            println!("  {sym} {} ({})", d.id, d.path);
        }
        println!();
    }
    if !p.unlocks.is_empty() {
        println!("Unlocks:");
        for u in &p.unlocks {
            println!("  - {u}");
        }
        println!();
    }
    println!("Template:");
    print!("{}", p.template);
    println!();
}

fn render_apply_human(p: &core::instructions::ApplyInstructions) {
    println!("Change: {}", p.change_name);
    println!("Schema: {}", p.schema_name);
    println!("State: {}", p.state);
    println!(
        "Progress: {}/{} complete",
        p.progress.complete, p.progress.total
    );
    println!();
    if let Some(missing) = &p.missing_artifacts {
        println!("Missing artifacts:");
        for m in missing {
            println!("  - {m}");
        }
    } else {
        println!("Tasks:");
        for t in &p.tasks {
            let sym = if t.done { "✓" } else { "○" };
            println!("  {sym} {}", t.description);
        }
    }
    println!();
    println!("Instruction:");
    print!("{}", p.instruction);
    println!();
}

// --- new ---

fn cmd_new(a: NewArgs) -> Result<()> {
    match a.command {
        NewCommands::Change(c) => cmd_new_change(c),
        NewCommands::Artifact(c) => cmd_new_artifact(c),
    }
}

fn cmd_new_change(a: NewChangeArgs) -> Result<()> {
    let paths = require_paths()?;
    let schema = a.schema.unwrap_or_else(|| "spec-driven".to_string());
    let dir = core::newcmd::new_change(&paths, &a.name, a.description.as_deref(), &schema, a.agent.as_deref())?;
    println!("✓ Created change: {}", a.name);
    println!("  Path: {}", dir.to_string_lossy());
    println!("  Schema: {schema}");
    Ok(())
}

fn cmd_new_artifact(a: NewArtifactArgs) -> Result<()> {
    let paths = require_paths()?;
    let type_ok = ["proposal", "design", "tasks", "spec"].contains(&a.artifact_type.as_str());
    let type_err = || {
        anyhow::anyhow!(
            "Unknown artifact type '{}'. Valid types: proposal, design, tasks, spec",
            a.artifact_type
        )
    };
    // Spectra's order: with an explicit --change, validate the type before existence; when
    // auto-detecting, resolve the change first (so "No active changes" wins over a bad type).
    // Change-not-found is reported WITHOUT a trailing period.
    let change = match a.change.as_deref() {
        Some(name) => {
            if !type_ok {
                return Err(type_err());
            }
            core::model::find_change(&paths, name)
                .ok_or_else(|| anyhow::anyhow!("Change '{name}' not found"))?
        }
        None => {
            let c = resolve_change(&paths, None)?;
            if !type_ok {
                return Err(type_err());
            }
            c
        }
    };
    let schema = schema_for(&change);
    let content = if a.stdin {
        Some(read_stdin())
    } else {
        None
    };
    let had_content = content.is_some();
    let (artifact_id, path) = core::newcmd::new_artifact(
        &change,
        &schema,
        &a.artifact_type,
        a.capability.as_deref(),
        content.as_deref(),
        a.force,
    )?;
    let _ = artifact_id;
    if a.json {
        // Compact single-line JSON, matching Spectra.
        let v = serde_json::json!({
            "artifact": a.artifact_type,
            "change": change.name,
            "path": path.to_string_lossy(),
            "status": "created",
            "validated": had_content,
            "warnings": [],
        });
        println!("{}", serde_json::to_string(&v)?);
        return Ok(());
    }
    println!("✓ Created {}: {}", a.artifact_type, path.to_string_lossy());
    if had_content {
        println!("  Content validated ✓");
    }
    Ok(())
}

// --- schemas / templates ---

fn cmd_schemas(a: JsonFlag) -> Result<()> {
    let schemas = core::schema::all();
    if a.json {
        let items: Vec<_> = schemas
            .iter()
            .map(|s| {
                serde_json::json!({
                    "artifacts": s.artifact_ids(),
                    "description": s.description,
                    "name": s.name,
                    "source": s.source,
                })
            })
            .collect();
        return print_json(&items);
    }
    println!("Available schemas:");
    for s in &schemas {
        println!("  {} ({}) — {}", s.name, s.source, s.description);
    }
    Ok(())
}

fn cmd_templates(a: TemplatesArgs) -> Result<()> {
    let schema_name = a.schema.unwrap_or_else(|| "spec-driven".to_string());
    let schema = core::schema::resolve(&schema_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Schema not found: Schema '{schema_name}' not found in project, user, or built-in locations"
        )
    })?;
    if a.json {
        let items: Vec<_> = schema
            .artifacts
            .iter()
            .map(|art| {
                serde_json::json!({
                    "artifactId": art.id,
                    "hasContent": !art.template.is_empty(),
                    "templateName": art.template_name,
                })
            })
            .collect();
        return print_json(&items);
    }
    println!("Templates ({})", schema.name);
    for art in &schema.artifacts {
        println!("  ✓ {} → {}", art.id, art.template_name);
    }
    Ok(())
}

// --- feedback ---

fn cmd_feedback(a: FeedbackArgs) -> Result<()> {
    let _ = a.body;
    println!("Thanks for your feedback!");
    println!("Please open an issue at https://github.com/speclink-app/speclink/issues");
    println!("Message: {}", a.message);
    Ok(())
}

// --- schema management ---

fn cmd_schema(a: SchemaArgs) -> Result<()> {
    match a.command {
        SchemaCommands::Which { name, all: _, json } => {
            let n = name.unwrap_or_else(|| "spec-driven".to_string());
            match core::schema::resolve(&n) {
                Some(s) => {
                    if json {
                        return print_json(&serde_json::json!({
                            "name": s.name,
                            "resolved": "built-in",
                            "sources": [{ "path": "(embedded in binary)", "source": "built-in" }],
                        }));
                    }
                    println!("Schema: {}", s.name);
                    println!("  → (embedded in binary) (built-in)");
                }
                None => {
                    // Unknown schema is informational, not an error (exit 0).
                    println!("Schema: {n}");
                    println!("Not found.");
                }
            }
        }
        SchemaCommands::Validate { name, verbose: _, json } => {
            let n = name.unwrap_or_else(|| "spec-driven".to_string());
            let s = core::schema::resolve(&n);
            match s {
                Some(s) => {
                    let count = s.artifacts.len();
                    if json {
                        return print_json(&serde_json::json!({
                            "artifactCount": count,
                            "name": s.name,
                            "valid": true,
                        }));
                    }
                    println!("✓ Schema '{}' is valid ({count} artifacts)", s.name);
                }
                None => {
                    let detail = format!(
                        "Schema not found: Schema '{n}' not found in project, user, or built-in locations"
                    );
                    println!("Schema '{n}' is invalid: {detail}");
                    bail!("Schema validation failed: {detail}");
                }
            }
        }
        SchemaCommands::Fork { .. } | SchemaCommands::Init { .. } => {
            bail!("custom schema management is not supported in speclink");
        }
    }
    Ok(())
}

// --- config (global) ---

fn global_config_path() -> PathBuf {
    // Per-OS config-dir convention (mirrors dirs::config_dir): Windows %APPDATA%,
    // macOS ~/Library/Application Support, Linux $XDG_CONFIG_HOME or ~/.config.
    let base = if cfg!(windows) {
        std::env::var("APPDATA").map(PathBuf::from).ok()
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
            .ok()
    } else {
        std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .ok()
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::var("HOME").map(|h| PathBuf::from(h).join(".config")).ok())
    };
    base.unwrap_or_else(|| PathBuf::from("."))
        .join("speclink")
        .join("config.yaml")
}

fn cmd_config(a: ConfigArgs) -> Result<()> {
    let path = global_config_path();
    match a.command {
        ConfigCommands::Path => println!("{}", core::util::to_slash(&path)),
        ConfigCommands::List { json } => {
            let cfg = load_global_map(&path);
            if json {
                return print_json(&cfg);
            }
            for (k, v) in &cfg {
                println!("{k} = {v}");
            }
        }
        ConfigCommands::Get { key } => {
            let cfg = load_global_map(&path);
            match cfg.get(&key) {
                Some(v) => println!("{v}"),
                None => bail!("key '{key}' not set"),
            }
        }
        ConfigCommands::Set { key, value, string: _, allow_unknown: _ } => {
            let mut cfg = load_global_map(&path);
            cfg.insert(key.clone(), value.clone());
            save_global_map(&path, &cfg)?;
            println!("✓ {key} = {value}");
        }
        ConfigCommands::Unset { key } => {
            let mut cfg = load_global_map(&path);
            cfg.remove(&key);
            save_global_map(&path, &cfg)?;
            println!("✓ Unset {key}");
        }
        ConfigCommands::Reset => {
            if path.exists() {
                std::fs::remove_file(&path)?;
            }
            println!("✓ Reset");
        }
        ConfigCommands::Edit => {
            println!("Config path: {}", core::util::to_slash(&path));
        }
    }
    Ok(())
}

fn load_global_map(path: &std::path::Path) -> std::collections::BTreeMap<String, String> {
    match core::util::read_opt(path) {
        Some(s) => serde_yaml::from_str(&s).unwrap_or_default(),
        None => Default::default(),
    }
}

fn save_global_map(
    path: &std::path::Path,
    map: &std::collections::BTreeMap<String, String>,
) -> Result<()> {
    let yaml = serde_yaml::to_string(map)?;
    core::util::write_file(path, &yaml)?;
    Ok(())
}

// --- completion ---

/// Validated display name for a completion shell. Elvish IS supported, but the error message
/// only lists the four common shells — replicated from Spectra verbatim.
fn completion_shell(shell: Option<&str>) -> Result<&'static str> {
    match shell.unwrap_or("bash") {
        "bash" => Ok("Bash"),
        "zsh" => Ok("Zsh"),
        "fish" => Ok("Fish"),
        "powershell" => Ok("PowerShell"),
        "elvish" => Ok("Elvish"),
        other => bail!("Unsupported shell: {other}. Use bash, zsh, fish, or powershell."),
    }
}

fn cmd_completion(a: CompletionArgs) -> Result<()> {
    match a.command {
        CompletionCommands::Generate { shell } => {
            use clap::CommandFactory;
            let sh = match completion_shell(shell.as_deref())? {
                "Zsh" => clap_complete::Shell::Zsh,
                "Fish" => clap_complete::Shell::Fish,
                "PowerShell" => clap_complete::Shell::PowerShell,
                "Elvish" => clap_complete::Shell::Elvish,
                _ => clap_complete::Shell::Bash,
            };
            let mut cmd = Cli::command();
            clap_complete::generate(sh, &mut cmd, "speclink", &mut std::io::stdout());
        }
        CompletionCommands::Install { shell, verbose: _ } => {
            // Spectra does not write to the shell profile; it prints guidance.
            let name = completion_shell(shell.as_deref())?;
            println!("Note: Shell completion for {name} — generate and source the output.");
            println!("Run: speclink completion generate {name} > completion_script");
            println!("Then source it in your shell profile.");
        }
        CompletionCommands::Uninstall { shell } => {
            let name = completion_shell(shell.as_deref())?;
            println!("Note: Remove the completion script for {name} from your shell profile.");
        }
    }
    Ok(())
}

// --- task ---

fn cmd_task(a: TaskArgs) -> Result<()> {
    match a.command {
        TaskCommands::Done { task_id, change, json } => {
            let paths = require_paths()?;
            // `task done` does not require the change to exist — it goes straight to tasks.md
            // (matching Spectra, which reports "tasks.md not found for change '<name>'").
            let change = match change.as_deref() {
                Some(name) => core::model::Change {
                    name: name.to_string(),
                    meta: core::model::ChangeMeta::load(&paths.change_dir(name)),
                    dir: paths.change_dir(name),
                },
                None => resolve_change(&paths, None)?,
            };
            // Check tasks.md existence BEFORE validating the id (matches Spectra's order).
            let tasks_path = change.dir.join("tasks.md");
            let text = core::util::read_opt(&tasks_path)
                .ok_or_else(|| anyhow::anyhow!("tasks.md not found for change '{}'", change.name))?;
            let id: usize = task_id
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid task ID '{task_id}': must be a number"))?;
            if id < 1 {
                bail!("Task ID must be >= 1");
            }
            let total = core::tasks::parse(&text).len();
            let (new_content, desc, already) = core::tasks::mark_done(&text, id)
                .ok_or_else(|| anyhow::anyhow!("Task {id} not found (total: {total})"))?;
            if already {
                bail!("Task {id} is already done");
            }
            core::util::write_file(&tasks_path, &new_content)?;

            // Record touched files.
            let files = core::tasks::git_changed_files(&paths.root);
            let mut record = core::tasks::TouchedRecord::load(&paths, &change.name);
            record.change = change.name.clone();
            record.touched.push(core::tasks::TouchedEntry {
                task_id: task_id.to_string(),
                task_desc: desc.clone(),
                files,
            });
            record.save(&paths)?;

            if json {
                // Compact single-line JSON, matching Spectra.
                let v = serde_json::json!({
                    "change": change.name,
                    "status": "done",
                    "task_desc": desc,
                    "task_id": task_id.to_string(),
                });
                println!("{}", serde_json::to_string(&v)?);
                return Ok(());
            }
            println!("✓ Task {task_id} marked as done: {desc}");
        }
    }
    Ok(())
}

// --- in-progress ---

fn cmd_in_progress(a: InProgressArgs) -> Result<()> {
    match a.command {
        InProgressCommands::Add { name } => {
            let paths = require_paths()?;
            core::inprogress::add(&paths, &name)?;
        }
    }
    Ok(())
}

// --- demo ---

fn cmd_demo() -> Result<()> {
    let paths = require_paths()?;
    let outcome = core::demo::generate(&paths)?;
    println!("✓ Created demo change: {}", outcome.name);
    println!("  Theme: {}", outcome.theme);
    println!("  Path: {}", core::util::to_slash(&outcome.path));
    Ok(())
}

// --- discuss ---

fn cmd_discuss(a: DiscussArgs) -> Result<()> {
    let paths = require_paths()?;
    match a.command {
        DiscussCommands::New { topic, json } => {
            let info = core::discuss::new_discussion(&paths, &topic)?;
            if json {
                return print_json(&info);
            }
            println!("✓ Created discussion: {}", info.slug);
            println!("  Topic: {}", info.topic);
            println!("  Path: {}", info.path);
        }
        DiscussCommands::List { json } => {
            let items = core::discuss::list_discussions(&paths);
            if json {
                return print_json(&serde_json::json!({ "discussions": items }));
            }
            if items.is_empty() {
                println!("No discussions found.");
                return Ok(());
            }
            println!("Discussions:");
            for d in &items {
                println!("  • {} [{}] ({} rounds) — {}", d.slug, d.status, d.rounds, d.topic);
            }
        }
        DiscussCommands::Show { slug, json } => {
            let content = core::discuss::show_discussion(&paths, &slug)
                .ok_or_else(|| anyhow::anyhow!("discussion '{slug}' not found"))?;
            if json {
                let info = core::discuss::info(&paths, &slug);
                return print_json(&serde_json::json!({ "info": info, "content": content }));
            }
            print!("{content}");
        }
        DiscussCommands::AddRound { slug, mode, stdin, json } => {
            let content = if stdin { read_stdin() } else { String::new() };
            let round = core::discuss::add_round(&paths, &slug, &mode, &content)?;
            if json {
                return print_json(&serde_json::json!({ "slug": slug, "round": round, "mode": mode }));
            }
            println!("✓ Recorded round {round} ({mode}) to discussion '{slug}'");
        }
        DiscussCommands::Conclude { slug, stdin, json } => {
            let content = if stdin { read_stdin() } else { String::new() };
            core::discuss::conclude(&paths, &slug, &content)?;
            if json {
                return print_json(&serde_json::json!({ "slug": slug, "status": "concluded" }));
            }
            println!("✓ Concluded discussion '{slug}'");
        }
    }
    Ok(())
}
