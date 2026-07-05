// Included into main.rs. Command handlers and rendering.

use core::model::Change;
use core::schema::Schema;
use core::store::Store;
use core::workspace::Workspace;

fn dispatch(cli: Cli) -> Result<()> {
    warn_deprecated_policy_keys();
    match cli.command {
        Commands::Init(a) => cmd_init(a),
        Commands::Update(a) => cmd_update(a),
        Commands::List(a) => cmd_list(a),
        Commands::Show(a) => cmd_show(a),
        Commands::Validate(a) => cmd_validate(a),
        Commands::Analyze(a) => cmd_analyze(a),
        Commands::Drift(a) => cmd_drift(a),
        Commands::Archive(a) => cmd_archive(a),
        Commands::Claim(a) => cmd_claim(a),
        Commands::Link(a) => cmd_link(a),
        Commands::Unlink => cmd_unlink(),
        Commands::Auth(a) => cmd_auth(a),
        Commands::Artifact(a) => cmd_artifact(a),
        Commands::Language(a) => cmd_language(a),
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

// --- claim ---

/// Claiming is an ownership concept of the remote lifecycle; the local fs
/// store has no claim state, so fs mode fails loud instead of pretending.
fn cmd_claim(a: ClaimArgs) -> Result<()> {
    match remote_ctx()? {
        Some(ctx) => remote_claim(&ctx, &a.name),
        None => bail!("claim requires a remote store — this project uses the local fs store"),
    }
}

// --- artifact cat / language show (dual-mode document reads) ---

/// Map a `speclink artifact cat` id onto the store's artifact file path
/// (`specs/<capability>` addresses a delta spec).
fn artifact_rel_path(artifact: &str) -> Result<String> {
    match artifact {
        "proposal" => Ok("proposal.md".to_string()),
        "design" => Ok("design.md".to_string()),
        "tasks" => Ok("tasks.md".to_string()),
        _ => match artifact.strip_prefix("specs/") {
            Some(cap) if !cap.is_empty() && !cap.contains('/') => {
                Ok(format!("specs/{cap}/spec.md"))
            }
            _ => bail!(
                "Unknown artifact '{artifact}'. Use proposal, design, tasks, or specs/<capability>"
            ),
        },
    }
}

fn cmd_artifact(a: ArtifactArgs) -> Result<()> {
    match a.command {
        ArtifactCommands::Cat { artifact, change } => {
            if let Some(ctx) = remote_ctx()? {
                return remote_artifact_cat(&ctx, &artifact, change.as_deref());
            }
            let (_ws, store) = open_project()?;
            let store: &dyn Store = &store;
            let change = resolve_change(store, change.as_deref())?;
            let rel = artifact_rel_path(&artifact)?;
            match store.read_artifact(&change.name, &rel) {
                Some(content) => {
                    print!("{content}");
                    Ok(())
                }
                None => bail!("artifact '{artifact}' not found for change '{}'", change.name),
            }
        }
    }
}

fn cmd_language(a: LanguageArgs) -> Result<()> {
    match a.command {
        LanguageCommands::Show => {
            if let Some(ctx) = remote_ctx()? {
                let payload = ctx.client.language()?;
                print!("{}", v_str(&payload, "content"));
                return Ok(());
            }
            let (_ws, store) = open_project()?;
            let store: &dyn Store = &store;
            match store.read_language() {
                Some(content) => {
                    print!("{content}");
                    Ok(())
                }
                None => bail!("this project has no LANGUAGE document (shared vocabulary)"),
            }
        }
    }
}

// --- helpers ---

/// Deprecation signal for the legacy policy keys: exactly one fixed-prefix stderr line per
/// invocation when `.speclink.yaml` still carries keys whose canonical home is
/// `openspec/config.yaml`. stdout (including `--json`) stays untouched; no keys → no output.
fn warn_deprecated_policy_keys() {
    let Some(ws) = Workspace::discover_cwd() else {
        return;
    };
    let keys = core::config::AppConfig::load(&ws.app_config()).deprecated_policy_keys();
    if keys.is_empty() {
        return;
    }
    eprintln!(
        "speclink: warning: deprecated policy keys in .speclink.yaml: {} (move them to openspec/config.yaml)",
        keys.join(", ")
    );
}

fn resolve_change(store: &dyn Store, name: Option<&str>) -> Result<Change> {
    resolve_change_worded(store, name, "Use --change to specify one:")
}

/// Positional-style resolution (analyze/drift): Spectra says just "Specify one:".
fn resolve_change_positional(store: &dyn Store, name: Option<&str>) -> Result<Change> {
    resolve_change_worded(store, name, "Specify one:")
}

fn resolve_change_worded(store: &dyn Store, name: Option<&str>, specify: &str) -> Result<Change> {
    if let Some(n) = name {
        return core::model::find_change(store, n)
            .ok_or_else(|| anyhow::anyhow!("Change '{n}' not found."));
    }
    let mut changes = core::model::list_changes(store);
    match changes.len() {
        0 => bail!("No active changes. Create one with: speclink new change <name>"),
        1 => Ok(changes.remove(0)),
        _ => {
            // Spectra lists the candidates by most-recently-modified first (newest file
            // mtime inside each change, whole seconds, name tiebreak).
            sort_changes(store, &mut changes, "modified");
            let names: Vec<&str> = changes.iter().map(|c| c.name.as_str()).collect();
            bail!("Multiple changes found. {specify} {}", names.join(", "))
        }
    }
}

/// For read/analysis commands: when no change name is given and no changes exist, print the
/// informational message and signal exit-0 (returns true = handled).
fn info_if_no_changes(store: &dyn Store, name: Option<&str>) -> bool {
    if name.is_none() && core::model::list_changes(store).is_empty() {
        println!("No active changes. Create one with: speclink new change <name>");
        true
    } else {
        false
    }
}

/// Resolve a schema by name (project → user → built-in) or fail with Spectra's messages.
fn resolve_schema(ws: &Workspace, name: &str) -> Result<Schema> {
    match core::schema::resolve_with(Some(ws), name) {
        Some(Ok(s)) => Ok(s),
        Some(Err(e)) => bail!("{e}"),
        None => bail!("{}", core::schema::not_found_msg(name)),
    }
}

fn schema_for(ws: &Workspace, change: &Change) -> Result<Schema> {
    resolve_schema(ws, &change.meta.schema_name())
}

// Shared with the Node SDK: the list serialization path lives in core::listing.
use core::listing::{proposal_summary, sort_changes, specs_json_items, task_counts, ListChangeJson};

// --- init / update ---

fn cmd_init(a: InitArgs) -> Result<()> {
    // The success line echoes the PATH argument verbatim (Spectra prints ".\openspec" for
    // `init .`); the absolute path is only used internally.
    let display_base = match a.path.as_deref() {
        Some(p) => p.to_string(),
        None => std::env::current_dir()?.display().to_string(),
    };
    let root = match a.path.as_deref() {
        Some(p) => {
            let pb = PathBuf::from(p);
            if pb.is_absolute() { pb } else { std::env::current_dir()?.join(pb) }
        }
        None => std::env::current_dir()?,
    };
    match a.store.as_str() {
        "fs" => {}
        "remote" => return cmd_init_remote(&a, &root, &display_base),
        other => bail!("Unknown store '{other}'. Use 'fs' or 'remote'."),
    }
    let spec_dir = a.dir.clone().unwrap_or_else(|| "openspec".to_string());
    // Without --tools, auto-detect installed AI tools by their footprints (falls back to
    // claude) — deliberate difference from Spectra, which generates nothing when omitted.
    let tools = match a.tools.as_deref() {
        Some(spec) => core::init::parse_tools(spec)?,
        None => core::init::detect_tools(&root),
    };
    core::init::init(&root, &tools, a.force, &spec_dir)?;
    println!("{} Initialized at {display_base}{}{spec_dir}", color::green("✓"), std::path::MAIN_SEPARATOR);
    if !tools.is_empty() {
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        println!("Generated files for: {}", names.join(", "));
    }
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
    let _ = a.force;
    let outcome = core::init::update(&root)?;
    for note in &outcome.notes {
        println!("! {note}");
    }
    if outcome.updated.is_empty() && outcome.pruned.is_empty() {
        println!("! No AI tool configurations found. Use 'speclink init --tools' to set up.");
    } else {
        if !outcome.updated.is_empty() {
            println!("{} Updated instruction files for: {}", color::green("✓"), outcome.updated.join(", "));
        }
        if !outcome.pruned.is_empty() {
            println!(
                "! Pruned generated files for deselected tool: {}",
                outcome.pruned.join(", ")
            );
        }
    }
    Ok(())
}

// --- list ---

fn cmd_list(a: ListArgs) -> Result<()> {
    if let Some(ctx) = remote_ctx()? {
        return remote_list(&ctx, &a);
    }
    let (_ws, store) = open_project()?;
    let store: &dyn Store = &store;
    // --specs alone shows only specs; combined with --changes both sections print.
    if a.specs && !a.changes {
        return list_specs(store, a.json);
    }
    let mut changes = core::model::list_changes(store);
    sort_changes(store, &mut changes, &a.sort);
    if a.json {
        let items: Vec<ListChangeJson> = core::listing::changes_json(store, &changes);
        if a.specs {
            let mut payload = serde_json::Map::new();
            payload.insert("changes".into(), serde_json::to_value(&items)?);
            payload.insert("specs".into(), specs_json_items(store));
            return print_json(&serde_json::Value::Object(payload));
        }
        return print_json(&serde_json::json!({ "changes": items }));
    }
    if changes.is_empty() {
        println!("No active changes.");
        if a.specs {
            println!();
            list_specs(store, false)?;
        }
        return Ok(());
    }
    println!("{}", color::bold("Changes:"));
    for c in &changes {
        let (complete, total) = task_counts(store, c);
        let summary = proposal_summary(store, c);
        // Spectra omits the progress marker entirely for changes with zero tasks.
        let marker = if total > 0 {
            format!(" [{complete}/{total}]")
        } else {
            String::new()
        };
        // The dim wrapper always prints — an empty summary yields Spectra's empty
        // \x1b[2m\x1b[0m pair in color mode and nothing in plain mode.
        let suffix = if summary.is_empty() {
            String::new()
        } else {
            format!(" — {summary}")
        };
        println!("  {} {}{marker}{}", color::cyan("•"), c.name, color::dim(&suffix));
    }
    if a.specs {
        println!();
        list_specs(store, false)?;
    }
    Ok(())
}

fn list_specs(store: &dyn Store, json: bool) -> Result<()> {
    let mut specs = store.list_canonical_capabilities();
    specs.sort();
    if json {
        return print_json(&serde_json::json!({ "specs": specs_json_items(store) }));
    }
    if specs.is_empty() {
        println!("No specs.");
        return Ok(());
    }
    println!("{}", color::bold("Specs:"));
    for s in specs {
        println!("  {} {s}", color::cyan("•"));
    }
    Ok(())
}

// --- show ---

fn cmd_show(a: ShowArgs) -> Result<()> {
    let (_ws, store) = open_project()?;
    let store: &dyn Store = &store;
    let item = match a.item {
        Some(n) => n,
        None => bail!("Please specify an item name."),
    };
    let item_type = a.item_type.as_deref();
    if let Some(t) = item_type {
        if t != "change" && t != "spec" {
            bail!("Unknown type: {t}. Use 'change' or 'spec'.");
        }
    }

    let is_spec = store.canonical_spec_exists(&item);
    let change = core::model::find_change(store, &item);

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
        let content = store.read_canonical_spec(&item).unwrap_or_default();
        if a.json {
            return print_json(&serde_json::json!({
                "files": [{ "content": content, "name": "spec.md" }],
                "name": item,
            }));
        }
        println!("{}: {item}", color::bold("Spec"));
        println!();
        println!("{}", color::dim("--- spec.md ---"));
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
    // show never resolves the schema — the Schema/Created lines echo the metadata verbatim,
    // and Spectra treats the metadata as one unit: unless BOTH schema and created are
    // present, neither is reported (missing/partial .openspec.yaml → null).
    let (schema_name, created) = match (&change.meta.schema, &change.meta.created) {
        (Some(s), Some(c)) => (Some(s.clone()), Some(c.clone())),
        _ => (None, None),
    };
    let read_opt_str = |name: &str| store.read_artifact(&change.name, name);

    if a.json {
        let proposal = read_opt_str("proposal.md");
        let design = read_opt_str("design.md");
        let tasks = read_opt_str("tasks.md");
        let caps: Vec<String> = store
            .delta_capabilities(&change.name)
            .into_iter()
            .map(|c| format!("{c}/spec.md"))
            .collect();
        return print_json(&serde_json::json!({
            "name": change.name,
            "schema": schema_name,
            "created": created,
            "proposal": proposal,
            "design": design,
            "tasks": tasks,
            "deltaSpecs": caps,
        }));
    }

    println!("{}: {}", color::bold("Change"), change.name);
    if let Some(schema_name) = &schema_name {
        println!("{}: {schema_name}", color::bold("Schema"));
    }
    if let Some(created) = &created {
        println!("{}: {created}", color::bold("Created"));
    }
    let proposal = read_opt_str("proposal.md");
    let caps = store.delta_capabilities(&change.name);
    // The header's trailing blank line prints only when a section follows.
    if proposal.is_some() || !caps.is_empty() {
        println!();
    }
    // The Proposal section renders whenever the FILE exists (even empty), matching Spectra.
    if let Some(proposal) = proposal {
        println!("{}", color::dim("--- Proposal ---"));
        print!("{proposal}");
        if !caps.is_empty() {
            // The proposal's own trailing newline determines the blank-line count before the header.
            print!("\n\n{}\n", color::dim("--- Delta Specs ---"));
            for c in &caps {
                println!("  {c}/spec.md");
            }
        } else {
            // Spectra always appends a newline after the proposal body (an extra blank
            // line when the file already ends with one), same as the spec branch above.
            println!();
        }
    } else if !caps.is_empty() {
        // No proposal, but delta specs exist — still render the section.
        println!("{}", color::dim("--- Delta Specs ---"));
        for c in &caps {
            println!("  {c}/spec.md");
        }
    }
    Ok(())
}

// --- validate ---

fn cmd_validate(a: ValidateArgs) -> Result<()> {
    let (_ws, store) = open_project()?;
    let store: &dyn Store = &store;
    let mut changes = if let Some(item) = a.item.as_deref() {
        vec![core::model::find_change(store, item)
            .ok_or_else(|| anyhow::anyhow!("Change '{item}' not found."))?]
    } else if a.all || a.changes {
        core::model::list_changes(store)
    } else {
        match resolve_change(store, None) {
            Ok(c) => vec![c],
            Err(_) => core::model::list_changes(store),
        }
    };
    // Multi-change runs are ordered newest-modified first (matches Spectra).
    sort_changes(store, &mut changes, "modified");
    // Spectra's validate never resolves the change's schema (an unresolvable one still validates).
    let schema = core::schema::spec_driven();
    let results: Vec<_> = changes
        .iter()
        .map(|c| core::validate::validate_change(store, c, &schema, a.strict))
        .collect();
    let any_invalid = results.iter().any(|r| !r.valid);
    if a.json {
        print_json(&results)?;
        if any_invalid {
            bail!("Validation failed.");
        }
        return Ok(());
    }
    for r in &results {
        if r.valid {
            println!("{} {} — valid", color::green("✓"), r.change);
        } else {
            println!("{} {} — invalid", color::red("✗"), r.change);
            for e in &r.errors {
                println!("  {} {e}", color::red("error:"));
            }
        }
        for w in &r.warnings {
            println!("  {} {w}", color::yellow("warn:"));
        }
    }
    if any_invalid {
        bail!("Validation failed.");
    }
    Ok(())
}

// --- analyze ---

fn cmd_analyze(a: ChangeArg) -> Result<()> {
    let (_ws, store) = open_project()?;
    let store: &dyn Store = &store;
    if info_if_no_changes(store, a.change.as_deref()) {
        return Ok(());
    }
    let change = resolve_change_positional(store, a.change.as_deref())?;
    // Spectra's analyzer is schema-agnostic (hard-wired to the classic artifacts) and never
    // resolves the change's schema.
    let schema = core::schema::spec_driven();
    let report = core::analyzer::analyze(store, &change, &schema);
    if a.json {
        return print_json(&report);
    }
    render_analyze(&report);
    Ok(())
}

fn render_analyze(report: &core::analyzer::AnalyzeReport) {
    println!("{}: {}", color::bold("Change"), report.change_id);
    println!();
    for d in &report.dimensions {
        let sym = if d.finding_count == 0 {
            color::green("✓")
        } else {
            color::yellow("●")
        };
        // Spectra's bold span covers a 14-wide padded name; the 15th column separator
        // space stays outside it (plain bytes are identical to the old {:<15} form).
        println!(
            "  {sym} {} {} ({} findings)",
            color::bold(&format!("{:<14}", d.dimension)),
            color::dim(&d.status),
            d.finding_count
        );
    }
    // The blank separator is tied to the "Analyzed:" line; an empty change (nothing analyzed)
    // prints "Missing:" directly after the dimensions, matching Spectra.
    if !report.artifacts_analyzed.is_empty() {
        println!();
        println!("  {} {}", color::dim("Analyzed:"), report.artifacts_analyzed.join(", "));
    }
    if !report.artifacts_missing.is_empty() {
        println!("  {} {}", color::yellow("Missing:"), report.artifacts_missing.join(", "));
    }
    if report.findings.is_empty() {
        println!();
        println!("  {} No issues found", color::green("✓"));
        return;
    }
    println!();
    println!("  {} ({}):", color::bold("Findings"), report.findings.len());
    println!();
    for f in &report.findings {
        let tag = match f.severity.as_str() {
            "Critical" => color::bold_red("CRITICAL"),
            "Warning" => color::yellow("WARNING"),
            _ => color::dim("SUGGEST"),
        };
        println!("  [{tag}] {}", f.summary);
        println!("    {} {}", color::dim("at:"), color::dim(&f.location));
        println!("    {} {}", color::dim("→"), color::dim(&f.recommendation));
    }
}

// --- drift ---

fn cmd_drift(a: ChangeArg) -> Result<()> {
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    if info_if_no_changes(store, a.change.as_deref()) {
        return Ok(());
    }
    let change = resolve_change_positional(store, a.change.as_deref())?;
    let report = core::drift::analyze(&ws, store, &change);
    if a.json {
        return print_json(&report);
    }
    render_drift(&report);
    Ok(())
}

fn render_drift(report: &core::drift::DriftReport) {
    println!("{}: {}", color::bold("Drift Report"), report.change_id);
    if let Some(created) = &report.created {
        println!("  Created: {created}");
    }
    println!();
    // Bold spans cover 11/35-wide padded cells with the separator spaces outside, plus
    // a leading-space " Score" cell — plain bytes are identical to the old format.
    println!(
        "  {} {} {}",
        color::bold(&format!("{:<11}", "Dimension")),
        color::bold(&format!("{:<35}", "Status")),
        color::bold(" Score")
    );
    for d in &report.dimensions {
        let score = if d.contributes_to_total {
            format!("+{}", d.score)
        } else {
            "—".to_string()
        };
        println!("  {:<11} {:<36} {:>5}", d.kind, d.status, score);
    }
    println!(
        "  {} {} {}",
        color::bold(&format!("{:<11}", "Total")),
        " ".repeat(35),
        color::bold(&format!("{:>6}", report.total_score))
    );
    if !report.broken_anchors.is_empty() {
        println!();
        println!("Broken anchors");
        for b in &report.broken_anchors {
            println!("  - {} ({}) — {}", b.anchor, b.category, b.reason);
        }
    }
    if !report.spec_assumptions.is_empty() {
        println!();
        println!("Stale delta assumptions");
        for a in &report.spec_assumptions {
            println!(
                "  - {} '{}' in {} — {}",
                a.operation, a.requirement, a.capability, a.reason
            );
        }
    }
    if !report.tasks_maybe_resolved.is_empty() {
        println!();
        println!("Tasks maybe already done");
        for t in &report.tasks_maybe_resolved {
            println!("  - {t}");
        }
    }
    if !report.tasks_blocked_external.is_empty() {
        println!();
        println!("Tasks blocked by external changes");
        for t in &report.tasks_blocked_external {
            println!("  - {t}");
        }
    }
    println!();
    let sev = report.severity.to_uppercase();
    let sev_colored = match sev.as_str() {
        "LIGHT" => color::bold_green(&sev),
        "MEDIUM" => color::bold_yellow(&sev),
        _ => color::bold_red(&sev),
    };
    println!("{}: {} drift", color::bold("Severity"), sev_colored);
    println!("> {}", color::bold_cyan(&report.primary_recommendation));
}

// --- archive ---

fn cmd_archive(a: ArchiveArgs) -> Result<()> {
    if let Some(ctx) = remote_ctx()? {
        return remote_archive(&ctx, &a);
    }
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    if a.all || a.changes.len() > 1 {
        return cmd_archive_bulk(&ws, store, &a);
    }
    let change = resolve_change(store, a.changes.first().map(|s| s.as_str()))?;

    mark_all_tasks_done(store, &change, a.mark_tasks_complete)?;
    let opts = core::archive::ArchiveOptions {
        skip_specs: a.skip_specs,
        no_validate: a.no_validate,
        mark_tasks_complete: a.mark_tasks_complete,
    };
    // Spectra leaves the in-progress marker untouched on archive; so do we.
    let outcome = core::archive::archive(&ws, store, &change, &opts)?;
    print_archive_outcome(&outcome);
    Ok(())
}

fn mark_all_tasks_done(store: &dyn Store, change: &core::model::Change, enabled: bool) -> Result<()> {
    if !enabled {
        return Ok(());
    }
    if let Some(text) = store.read_artifact(&change.name, "tasks.md") {
        // Star-bullet checkboxes are tasks too (matches Spectra).
        let done = text
            .replace("- [ ] ", "- [x] ")
            .replace("- [ ]\t", "- [x]\t")
            .replace("* [ ] ", "* [x] ")
            .replace("* [ ]\t", "* [x]\t");
        store.write_artifact(&change.name, "tasks.md", &done)?;
    }
    Ok(())
}

fn print_archive_outcome(outcome: &core::archive::ArchiveOutcome) {
    println!("{} Archived: {} → {}", color::green("✓"), outcome.change_name, outcome.dated_name);
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
    if outcome.snapshot_created {
        println!("Snapshot created for unarchive support.");
    }
    if let Some((slug, file)) = &outcome.archived_discussion {
        println!("Discussion archived: {slug} → discussions/archive/{file}");
    }
}

/// Bulk archive (speclink-specific). Semantics: requires a clean code work tree (the dirty
/// file set is the @trace source and would be injected into EVERY archived change), archives
/// in created-date order, skips not-ready changes with a reason (never silently), and
/// fail-fasts on the first actual archive error with a three-part report.
fn cmd_archive_bulk(ws: &Workspace, store: &dyn Store, a: &ArchiveArgs) -> Result<()> {
    let dirty = core::tasks::git_changed_files(&ws.root);
    if !dirty.is_empty() {
        bail!(
            "bulk archive requires a clean work tree — these files would be injected into every change's @trace:\n  {}",
            dirty.join("\n  ")
        );
    }

    let mut changes: Vec<core::model::Change> = if a.all {
        core::model::list_changes(store)
    } else {
        let mut v = Vec::new();
        for name in &a.changes {
            v.push(
                core::model::find_change(store, name)
                    .ok_or_else(|| anyhow::anyhow!("Change '{name}' not found."))?,
            );
        }
        v
    };
    if changes.is_empty() {
        println!("No active changes to archive.");
        return Ok(());
    }
    changes.sort_by(|x, y| {
        (x.meta.created.as_deref().unwrap_or(""), &x.name)
            .cmp(&(y.meta.created.as_deref().unwrap_or(""), &y.name))
    });

    let schema = core::schema::spec_driven();
    let mut archived: Vec<String> = Vec::new();
    let mut skipped: Vec<(String, String)> = Vec::new();
    for (idx, change) in changes.iter().enumerate() {
        // Readiness: no stale delta assumptions, valid, tasks complete.
        let stale = core::drift::spec_assumptions(store, change);
        if !stale.is_empty() {
            skipped.push((
                change.name.clone(),
                format!(
                    "{} stale delta assumption(s) — run /speclink-drift {}",
                    stale.len(),
                    change.name
                ),
            ));
            continue;
        }
        if !a.no_validate {
            let res = core::validate::validate_change(store, change, &schema, false);
            if !res.valid {
                skipped.push((change.name.clone(), "validation failed".to_string()));
                continue;
            }
        }
        let tasks = core::tasks::parse(
            &store.read_artifact(&change.name, "tasks.md").unwrap_or_default(),
        );
        let (total, complete, _) = core::tasks::progress(&tasks);
        if total > 0 && complete < total && !a.mark_tasks_complete {
            skipped.push((change.name.clone(), format!("tasks incomplete ({complete}/{total})")));
            continue;
        }
        mark_all_tasks_done(store, change, a.mark_tasks_complete)?;

        let opts = core::archive::ArchiveOptions {
            skip_specs: a.skip_specs,
            no_validate: a.no_validate,
            mark_tasks_complete: a.mark_tasks_complete,
        };
        match core::archive::archive(ws, store, change, &opts) {
            Ok(outcome) => {
                print_archive_outcome(&outcome);
                archived.push(outcome.change_name);
            }
            Err(e) => {
                // Fail-fast: earlier archives are already applied and cannot be rolled back.
                println!();
                println!("Bulk archive aborted at '{}': {e}", change.name);
                if !archived.is_empty() {
                    println!("  Archived: {}", archived.join(", "));
                }
                let untouched: Vec<&str> =
                    changes[idx + 1..].iter().map(|c| c.name.as_str()).collect();
                if !untouched.is_empty() {
                    println!("  Untouched: {}", untouched.join(", "));
                }
                bail!("bulk archive failed at '{}'", change.name);
            }
        }
    }

    println!();
    for (name, why) in &skipped {
        println!("! Skipped: {name} — {why}");
    }
    println!(
        "Bulk archive: {} archived, {} skipped",
        archived.len(),
        skipped.len()
    );
    Ok(())
}

// --- status ---

fn cmd_status(a: StatusArgs) -> Result<()> {
    if let Some(ctx) = remote_ctx()? {
        return remote_status(&ctx, &a);
    }
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    if info_if_no_changes(store, a.change.as_deref()) {
        return Ok(());
    }
    let change = resolve_change(store, a.change.as_deref())?;
    let schema = if let Some(s) = a.schema.as_deref() {
        resolve_schema(&ws, s)?
    } else {
        schema_for(&ws, &change)?
    };
    let report = core::status::build(store, &change, &schema);
    if a.json {
        return print_json(&report);
    }
    render_status_human(&report);
    Ok(())
}

/// Human status rendering — shared by the fs and remote paths so both modes
/// stay byte-identical.
fn render_status_human(report: &core::status::StatusReport) {
    println!("{}: {}", color::bold("Change"), report.change_name);
    println!("{}: {}", color::bold("Schema"), report.schema_name);
    println!();
    for art in &report.artifacts {
        let sym = match art.status.as_str() {
            "done" => color::green("✓"),
            "ready" => color::yellow("○"),
            _ => color::red("✗"),
        };
        println!(
            "  {sym} {} ({})",
            color::bold(&art.id),
            color::dim(&art.output_path)
        );
        if art.status == "blocked" && !art.blocked_by.is_empty() {
            println!("    blocked by: {}", art.blocked_by.join(", "));
        }
    }
    println!();
    if report.is_complete {
        println!("  {} All artifacts complete", color::green("✓"));
    }
}

// --- instructions ---

fn cmd_instructions(a: InstructionsArgs) -> Result<()> {
    if let Some(skill) = a.skill.as_deref() {
        let body = core::skills::skill_body(skill)
            .ok_or_else(|| anyhow::anyhow!("Unknown skill: {skill}"))?;
        print!("{body}");
        return Ok(());
    }
    if let Some(ctx) = remote_ctx()? {
        return remote_instructions(&ctx, &a);
    }
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    if info_if_no_changes(store, a.change.as_deref()) {
        return Ok(());
    }
    let change = resolve_change(store, a.change.as_deref())?;
    let schema = if let Some(s) = a.schema.as_deref() {
        resolve_schema(&ws, s)?
    } else {
        schema_for(&ws, &change)?
    };
    // No-arg default: the first incomplete artifact, or the apply/status view once every
    // artifact exists (matches Spectra — not the proposal-creation instructions).
    let default_artifact = core::status::first_incomplete_artifact(store, &change, &schema)
        .unwrap_or_else(|| "apply".to_string());
    let artifact = a.artifact.as_deref().unwrap_or(&default_artifact);
    if artifact == "apply" {
        let payload = core::instructions::build_apply(&ws, store, &change, &schema);
        if a.json {
            return print_json(&payload);
        }
        render_apply_human(&payload);
        return Ok(());
    }
    let payload = core::instructions::build_artifact(&ws, store, &change, &schema, artifact)
        .ok_or_else(|| anyhow::anyhow!("Artifact '{artifact}' not found in schema"))?;
    if a.json {
        return print_json(&payload);
    }
    render_artifact_human(&payload);
    Ok(())
}

fn render_artifact_human(p: &core::instructions::ArtifactInstructions) {
    println!("{}: {}", color::bold("Artifact"), p.artifact_id);
    println!("{}: {}", color::bold("Output"), p.output_path);
    println!("{}: {}", color::bold("Description"), p.description);
    // Each section is preceded by one blank separator and rendered only when non-empty
    // (a custom schema may have no instruction and an empty template), matching Spectra.
    if let Some(instr) = &p.instruction {
        println!();
        println!("{}", color::bold("Instruction:"));
        print!("{instr}"); // ends with a newline
        println!();
    }
    if !p.dependencies.is_empty() {
        println!();
        println!("{}", color::bold("Dependencies:"));
        // Dependency symbols stay plain (probed — unlike status's colored ones).
        for d in &p.dependencies {
            let sym = if d.done { "✓" } else { "○" };
            println!("  {sym} {} ({})", d.id, d.path);
        }
    }
    if !p.unlocks.is_empty() {
        println!();
        println!("{}", color::bold("Unlocks:"));
        for u in &p.unlocks {
            println!("  - {u}");
        }
    }
    if !p.template.is_empty() {
        println!();
        println!("{}", color::bold("Template:"));
        print!("{}", p.template);
        println!();
    }
}

fn render_apply_human(p: &core::instructions::ApplyInstructions) {
    println!("{}: {}", color::bold("Change"), p.change_name);
    println!("{}: {}", color::bold("Schema"), p.schema_name);
    println!("{}: {}", color::bold("State"), p.state);
    println!(
        "{}: {}/{} complete",
        color::bold("Progress"),
        p.progress.complete,
        p.progress.total
    );
    println!();
    if let Some(missing) = &p.missing_artifacts {
        println!("{}", color::red("Missing artifacts:"));
        for m in missing {
            println!("  - {m}");
        }
    } else {
        println!("{}", color::bold("Tasks:"));
        // Task symbols stay plain here (probed — unlike status's colored ones).
        for t in &p.tasks {
            let sym = if t.done { "✓" } else { "○" };
            println!("  {sym} {}", t.description);
        }
    }
    println!();
    if let Some(instr) = &p.instruction {
        println!("{}", color::bold("Instruction:"));
        print!("{instr}");
        println!();
    }
}

// --- new ---

fn cmd_new(a: NewArgs) -> Result<()> {
    match a.command {
        NewCommands::Change(c) => cmd_new_change(c),
        NewCommands::Artifact(c) => cmd_new_artifact(c),
    }
}

fn cmd_new_change(a: NewChangeArgs) -> Result<()> {
    if let Some(ctx) = remote_ctx()? {
        return remote_new_change(&ctx, &a);
    }
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    // Default schema comes from openspec/config.yaml; the name is NOT validated here (Spectra
    // accepts unknown names and lets downstream commands fail on resolution).
    let schema = a.schema.unwrap_or_else(|| {
        core::config::WorkflowConfig::from_text(store.read_workflow_config().as_deref())
            .schema_name()
    });
    if let Some(slug) = a.from_discussion.as_deref() {
        if core::discuss::info(store, slug).is_none() {
            bail!("discussion '{slug}' not found — run `speclink discuss new` first");
        }
    }
    let dir = core::newcmd::new_change(
        &ws,
        store,
        &a.name,
        a.description.as_deref(),
        &schema,
        a.agent.as_deref(),
        a.from_discussion.as_deref(),
    )?;
    println!("{} Created change: {}", color::green("✓"), a.name);
    println!("  Path: {}", dir.to_string_lossy());
    println!("  Schema: {schema}");
    if let Some(slug) = a.from_discussion.as_deref() {
        core::discuss::mark_promoted(store, slug, &a.name)?;
        println!("  From discussion: {slug}");
    }
    Ok(())
}

fn cmd_new_artifact(a: NewArtifactArgs) -> Result<()> {
    if let Some(ctx) = remote_ctx()? {
        return remote_new_artifact(&ctx, &a);
    }
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
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
            core::model::find_change(store, name)
                .ok_or_else(|| anyhow::anyhow!("Change '{name}' not found"))?
        }
        None => {
            let c = resolve_change(store, None)?;
            if !type_ok {
                return Err(type_err());
            }
            c
        }
    };
    // Best-effort schema resolution: an unresolvable/broken schema still creates the artifact
    // (with no template → an empty file), matching Spectra.
    let schema = match core::schema::resolve_with(Some(&ws), &change.meta.schema_name()) {
        Some(Ok(s)) => s,
        _ => core::schema::Schema {
            name: change.meta.schema_name(),
            display_name: change.meta.schema_name(),
            description: None,
            source: "project".to_string(),
            artifacts: Vec::new(),
            apply_requires: Vec::new(),
            apply_tracks: None,
            apply_instruction: None,
        },
    };
    let content = if a.stdin {
        Some(read_stdin())
    } else {
        None
    };
    let had_content = content.is_some();
    let (artifact_id, path) = core::newcmd::new_artifact(
        store,
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
    println!("{} Created {}: {}", color::green("✓"), a.artifact_type, path.to_string_lossy());
    if had_content {
        println!("  Content validated ✓");
    }
    Ok(())
}

// --- schemas / templates ---

fn cmd_schemas(a: JsonFlag) -> Result<()> {
    let ws = core::workspace::Workspace::discover_cwd();
    let schemas = core::schema::list_all(ws.as_ref());
    if a.json {
        let items: Vec<_> = schemas
            .iter()
            .map(|s| {
                serde_json::json!({
                    "artifacts": s.artifact_ids,
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
        match &s.description {
            Some(d) => println!("  {} ({}) — {}", s.name, s.source, d),
            None => println!("  {} ({})", s.name, s.source),
        }
    }
    Ok(())
}

fn cmd_templates(a: TemplatesArgs) -> Result<()> {
    let ws = core::workspace::Workspace::discover_cwd();
    let schema_name = a.schema.unwrap_or_else(|| "spec-driven".to_string());
    let schema = match core::schema::resolve_with(ws.as_ref(), &schema_name) {
        Some(Ok(s)) => s,
        Some(Err(e)) => bail!("{e}"),
        None => bail!("{}", core::schema::not_found_msg(&schema_name)),
    };
    if a.json {
        let items: Vec<_> = schema
            .artifacts
            .iter()
            .map(|art| {
                serde_json::json!({
                    "artifactId": art.id,
                    "hasContent": art.template.as_deref().map(|t| !t.is_empty()).unwrap_or(false),
                    "templateName": art.template_name,
                })
            })
            .collect();
        return print_json(&items);
    }
    println!("Templates ({})", schema.name);
    for art in &schema.artifacts {
        let sym = if art.template.as_deref().map(|t| !t.is_empty()).unwrap_or(false) { "✓" } else { "✗" };
        println!("  {sym} {} → {}", art.id, art.template_name);
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
    let ws = core::workspace::Workspace::discover_cwd();
    match a.command {
        SchemaCommands::Which { name, all: _, json } => {
            let n = name.unwrap_or_else(|| "spec-driven".to_string());
            let sources = core::schema::sources(ws.as_ref(), &n);
            if sources.is_empty() {
                // Unknown schema is informational, not an error (exit 0).
                println!("Schema: {n}");
                println!("Not found.");
                return Ok(());
            }
            let display = |s: &core::schema::SchemaSource| match &s.path {
                Some(p) => (p.to_string_lossy().to_string(), s.source),
                None => ("(embedded in binary)".to_string(), s.source),
            };
            if json {
                let items: Vec<_> = sources
                    .iter()
                    .map(|s| {
                        let (p, src) = display(s);
                        serde_json::json!({ "path": p, "source": src })
                    })
                    .collect();
                return print_json(&serde_json::json!({
                    "name": n,
                    "resolved": sources[0].source,
                    "sources": items,
                }));
            }
            println!("Schema: {n}");
            for (i, s) in sources.iter().enumerate() {
                let (p, src) = display(s);
                if i == 0 {
                    println!("  → {p} ({src})");
                } else {
                    println!("    {p} ({src})");
                }
            }
        }
        SchemaCommands::Validate { name, verbose: _, json } => {
            let n = name.unwrap_or_else(|| "spec-driven".to_string());
            match core::schema::resolve_with(ws.as_ref(), &n) {
                Some(Ok(s)) => {
                    let count = s.artifacts.len();
                    if json {
                        return print_json(&serde_json::json!({
                            "artifactCount": count,
                            "name": s.name,
                            "valid": true,
                        }));
                    }
                    println!("{} Schema '{}' is valid ({count} artifacts)", color::green("✓"), s.name);
                }
                Some(Err(detail)) => {
                    println!("Schema '{n}' is invalid: {detail}");
                    bail!("Schema validation failed: {detail}");
                }
                None => {
                    let detail = core::schema::not_found_msg(&n);
                    println!("Schema '{n}' is invalid: {detail}");
                    bail!("Schema validation failed: {detail}");
                }
            }
        }
        SchemaCommands::Fork { source, name, force, json: _ } => {
            let ws = require_workspace()?;
            let new_name = core::schema::fork(&ws, &source, name.as_deref(), force)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{} Forked '{source}' → '{new_name}'", color::green("✓"));
        }
        SchemaCommands::Init { name, description, artifacts, default: _, force } => {
            let ws = require_workspace()?;
            let dir = core::schema::init_schema(
                &ws,
                &name,
                artifacts.as_deref(),
                description.as_deref(),
                force,
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{} Created schema '{name}' at {}", color::green("✓"), dir.display());
        }
    }
    Ok(())
}

// --- config (global) ---

fn global_config_path() -> PathBuf {
    core::config::global_config_dir().join("config.yaml")
}

fn cmd_config(a: ConfigArgs) -> Result<()> {
    let path = global_config_path();
    match a.command {
        ConfigCommands::Path => println!("{}", core::util::to_slash(&path)),
        ConfigCommands::List { json } => {
            // The stored file keeps insertion order, but list output is sorted by key.
            let cfg = load_global_map(&path);
            let mut entries: Vec<(String, serde_yaml::Value)> = cfg
                .into_iter()
                .map(|(k, v)| (k.as_str().unwrap_or_default().to_string(), v))
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            if json {
                let mut sorted = serde_yaml::Mapping::new();
                for (k, v) in entries {
                    sorted.insert(serde_yaml::Value::String(k), v);
                }
                return print_json(&sorted);
            }
            if entries.is_empty() {
                println!("No configuration set.");
            }
            for (k, v) in &entries {
                println!("{k} = {}", scalar_str(v));
            }
        }
        ConfigCommands::Get { key } => {
            let cfg = load_global_map(&path);
            match cfg.get(serde_yaml::Value::String(key.clone())) {
                Some(v) => println!("{}", scalar_str(v)),
                None => bail!("Key '{key}' not found."),
            }
        }
        ConfigCommands::Set { key, value, string, allow_unknown: _ } => {
            let mut cfg = load_global_map(&path);
            // Values parse to native YAML scalars (1 → int, true → bool); --string forces
            // string storage (matches Spectra).
            let stored = if string {
                serde_yaml::Value::String(value.clone())
            } else {
                serde_yaml::from_str(&value)
                    .unwrap_or_else(|_| serde_yaml::Value::String(value.clone()))
            };
            cfg.insert(serde_yaml::Value::String(key.clone()), stored);
            save_global_map(&path, &cfg)?;
            println!("{} {key} = {value}", color::green("✓"));
        }
        ConfigCommands::Unset { key } => {
            let mut cfg = load_global_map(&path);
            cfg.remove(serde_yaml::Value::String(key.clone()));
            save_global_map(&path, &cfg)?;
            // Printed whether or not the key existed (matches Spectra).
            println!("{} Removed key: {key}", color::green("✓"));
        }
        ConfigCommands::Reset { all: _, yes: _ } => {
            if path.exists() {
                std::fs::remove_file(&path)?;
            }
            println!("{} Config reset.", color::green("✓"));
        }
        ConfigCommands::Edit => {
            // VISUAL wins over EDITOR; the vi fallback matches Spectra (including the
            // failure message when no editor can be spawned).
            let editor = std::env::var("VISUAL")
                .or_else(|_| std::env::var("EDITOR"))
                .unwrap_or_else(|_| "vi".to_string());
            let status = std::process::Command::new(&editor).arg(&path).status();
            match status {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    bail!("Failed to open editor '{editor}': program not found")
                }
                Err(e) => bail!("Failed to open editor '{editor}': {e}"),
            }
        }
    }
    Ok(())
}

/// Bare display form of a YAML scalar (strings unquoted, numbers/bools as literals).
fn scalar_str(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        other => serde_yaml::to_string(other).unwrap_or_default().trim_end().to_string(),
    }
}

// Insertion-ordered mapping (serde_yaml::Mapping) — Spectra preserves the order keys were
// first set in both the stored YAML and `config list` output.
fn load_global_map(path: &std::path::Path) -> serde_yaml::Mapping {
    match core::util::read_opt(path) {
        Some(s) => serde_yaml::from_str(&s).unwrap_or_default(),
        None => Default::default(),
    }
}

fn save_global_map(path: &std::path::Path, map: &serde_yaml::Mapping) -> Result<()> {
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
            if sh == clap_complete::Shell::Bash {
                // Spectra's (older clap_complete) bash script offers positional value
                // names as completion candidates ("[CHANGE]", "<KEY>"); newer
                // clap_complete dropped them, so they are re-injected here.
                let mut buf: Vec<u8> = Vec::new();
                clap_complete::generate(sh, &mut cmd, "speclink", &mut buf);
                let script = String::from_utf8_lossy(&buf).to_string();
                print!("{}", bash_inject_positionals(&script, &cmd));
                return Ok(());
            }
            clap_complete::generate(sh, &mut cmd, "speclink", &mut std::io::stdout());
        }
        CompletionCommands::Install { shell, verbose: _ } => {
            // Spectra does not write to the shell profile; it prints guidance.
            let name = completion_shell(shell.as_deref())?;
            println!("Note: Shell completion for {name} — generate and source the output.");
            println!("Run: speclink completion generate {name} > completion_script");
            println!("Then source it in your shell profile.");
        }
        CompletionCommands::Uninstall { shell, yes: _ } => {
            let name = completion_shell(shell.as_deref())?;
            println!("Note: Remove the completion script for {name} from your shell profile.");
        }
    }
    Ok(())
}

/// Append positional value-name placeholders (`<KEY>`, `[CHANGE]`) to each `opts="..."`
/// line of a clap_complete bash script, matching the older clap_complete Spectra ships.
/// Command paths are recovered from the script's own `parent,child) cmd="label"` arms.
fn bash_inject_positionals(script: &str, root: &clap::Command) -> String {
    use std::collections::HashMap;
    // label -> command path (root label "speclink" -> []).
    let mut paths: HashMap<String, Vec<String>> = HashMap::new();
    paths.insert("speclink".to_string(), Vec::new());
    let lines: Vec<&str> = script.lines().collect();
    for w in lines.windows(2) {
        let arm = w[0].trim();
        let assign = w[1].trim();
        let (Some(arm), Some(label)) = (
            arm.strip_suffix(')'),
            assign.strip_prefix("cmd=\"").and_then(|s| s.strip_suffix('"')),
        ) else {
            continue;
        };
        if let Some((parent, child)) = arm.split_once(',') {
            if let Some(parent_path) = paths.get(parent).cloned() {
                let mut p = parent_path;
                p.push(child.to_string());
                paths.insert(label.to_string(), p);
            }
        }
    }
    let placeholder = |path: &[String]| -> String {
        let mut c = root;
        for name in path {
            match c.get_subcommands().find(|s| s.get_name() == *name) {
                Some(sub) => c = sub,
                None => return String::new(),
            }
        }
        if c.has_subcommands() {
            return String::new();
        }
        let mut out = String::new();
        for a in c.get_positionals() {
            let name = a
                .get_value_names()
                .and_then(|v| v.first().map(|s| s.to_string()))
                .unwrap_or_else(|| a.get_id().to_string().to_uppercase());
            if a.is_required_set() {
                out.push_str(&format!(" <{name}>"));
            } else {
                out.push_str(&format!(" [{name}]"));
            }
        }
        out
    };
    let mut out = String::new();
    let mut current_label: Option<String> = None;
    for line in script.lines() {
        let t = line.trim();
        if let Some(l) = t.strip_suffix(')') {
            if paths.contains_key(l) {
                current_label = Some(l.to_string());
            }
        }
        if let (Some(label), true) = (&current_label, t.starts_with("opts=\"")) {
            if let Some(path) = paths.get(label) {
                let ph = placeholder(path);
                if !ph.is_empty() {
                    if let Some(stripped) = line.strip_suffix('"') {
                        out.push_str(stripped);
                        out.push_str(&ph);
                        out.push('"');
                        out.push('\n');
                        continue;
                    }
                }
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

// --- task ---

fn cmd_task(a: TaskArgs) -> Result<()> {
    match a.command {
        TaskCommands::Done { task_id, change, json } => {
            if let Some(ctx) = remote_ctx()? {
                return remote_task_done(&ctx, &task_id, change.as_deref(), json);
            }
            let (ws, store) = open_project()?;
            let store: &dyn Store = &store;
            // `task done` does not require the change to exist — it goes straight to tasks.md
            // (matching Spectra, which reports "tasks.md not found for change '<name>'").
            let change_name = match change.as_deref() {
                Some(name) => name.to_string(),
                None => resolve_change(store, None)?.name,
            };
            // Check tasks.md existence BEFORE validating the id (matches Spectra's order).
            let text = store
                .read_artifact(&change_name, "tasks.md")
                .ok_or_else(|| anyhow::anyhow!("tasks.md not found for change '{change_name}'"))?;
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
            store.write_artifact(&change_name, "tasks.md", &new_content)?;

            // Record touched files: only those not already attributed to an earlier task;
            // when nothing new is dirty, no entry is appended at all (matches Spectra).
            let mut record = core::tasks::TouchedRecord::load(&ws, &change_name);
            record.change = change_name.clone();
            let seen = record.all_files();
            let files: Vec<String> = core::tasks::git_changed_files(&ws.root)
                .into_iter()
                .filter(|f| !seen.contains(f))
                .collect();
            if !files.is_empty() {
                record.touched.push(core::tasks::TouchedEntry {
                    task_id: task_id.to_string(),
                    task_desc: desc.clone(),
                    files,
                });
                record.save(&ws)?;
            }

            if json {
                // Compact single-line JSON, matching Spectra.
                let v = serde_json::json!({
                    "change": change_name,
                    "status": "done",
                    "task_desc": desc,
                    "task_id": task_id.to_string(),
                });
                println!("{}", serde_json::to_string(&v)?);
                return Ok(());
            }
            println!("{} Task {task_id} marked as done: {desc}", color::green("✓"));
        }
    }
    Ok(())
}

// --- in-progress ---

fn cmd_in_progress(a: InProgressArgs) -> Result<()> {
    match a.command {
        InProgressCommands::Add { name } => {
            let ws = require_workspace()?;
            core::inprogress::add(&ws, &name)?;
        }
    }
    Ok(())
}

// --- demo ---

fn cmd_demo() -> Result<()> {
    let (ws, store) = open_project()?;
    let outcome = core::demo::generate(&ws, &store)?;
    println!("{} Created demo change: {}", color::green("✓"), outcome.name);
    println!("  Theme: {}", outcome.theme);
    println!("  Path: {}", core::util::to_slash(&outcome.path));
    Ok(())
}

// --- discuss ---

fn cmd_discuss(a: DiscussArgs) -> Result<()> {
    if let Some(ctx) = remote_ctx()? {
        return remote_discuss(&ctx, a);
    }
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    match a.command {
        DiscussCommands::New { topic, json } => {
            let info = core::discuss::new_discussion(store, &topic)?;
            if json {
                return print_json(&info);
            }
            println!("{} Created discussion: {}", color::green("✓"), info.slug);
            println!("  Topic: {}", info.topic);
            println!("  Path: {}", info.path);
        }
        DiscussCommands::List { archived, json } => {
            let items = if archived {
                core::discuss::list_archived(store)
            } else {
                core::discuss::list_discussions(store)
            };
            if json {
                return print_json(&serde_json::json!({ "discussions": items }));
            }
            if items.is_empty() {
                let what = if archived { "archived discussions" } else { "discussions" };
                println!("No {what} found.");
                return Ok(());
            }
            let heading = if archived { "Archived discussions:" } else { "Discussions:" };
            println!("{heading}");
            for d in &items {
                println!("  • {} [{}] ({} rounds) — {}", d.slug, d.status, d.rounds, d.topic);
            }
        }
        DiscussCommands::Show { slug, json } => {
            let content = core::discuss::show_discussion(store, &slug)
                .ok_or_else(|| anyhow::anyhow!("discussion '{slug}' not found"))?;
            if json {
                let info = core::discuss::info(store, &slug);
                return print_json(&serde_json::json!({ "info": info, "content": content }));
            }
            print!("{content}");
        }
        DiscussCommands::Context { slug, stdin, json } => {
            let content = if stdin { read_stdin() } else { String::new() };
            core::discuss::set_context(store, &slug, &content)?;
            if json {
                return print_json(&serde_json::json!({ "slug": slug, "context": "set" }));
            }
            println!("{} Set context for discussion '{slug}'", color::green("✓"));
        }
        DiscussCommands::AddRound { slug, mode, stdin, json } => {
            let content = if stdin { read_stdin() } else { String::new() };
            let round = core::discuss::add_round(store, &slug, &mode, &content)?;
            if json {
                return print_json(&serde_json::json!({ "slug": slug, "round": round, "mode": mode }));
            }
            println!("{} Recorded round {round} ({mode}) to discussion '{slug}'", color::green("✓"));
        }
        DiscussCommands::Conclude { slug, stdin, json } => {
            let content = if stdin { read_stdin() } else { String::new() };
            core::discuss::conclude(store, &slug, &content)?;
            if json {
                return print_json(&serde_json::json!({ "slug": slug, "status": "concluded" }));
            }
            println!("{} Concluded discussion '{slug}'", color::green("✓"));
        }
        DiscussCommands::Archive { slug, json } => {
            match core::discuss::archive_discussion(store, &slug)? {
                Some(file) => {
                    if json {
                        return print_json(&serde_json::json!({
                            "slug": slug,
                            "archived_to": format!("discussions/archive/{file}"),
                        }));
                    }
                    println!("{} Archived discussion: {slug} → discussions/archive/{file}", color::green("✓"));
                }
                None => bail!("discussion '{slug}' not found"),
            }
        }
        DiscussCommands::Discard { slug, force, json } => {
            core::discuss::discard_discussion(store, &slug, force)?;
            if json {
                return print_json(&serde_json::json!({ "slug": slug, "status": "discarded" }));
            }
            println!("{} Discarded discussion: {slug}", color::green("✓"));
        }
        DiscussCommands::Promote { slug, name, json } => {
            match core::discuss::info(store, &slug) {
                None => bail!("discussion '{slug}' not found — run `speclink discuss new` first"),
                Some(i) if i.archived => {
                    bail!("discussion '{slug}' is archived — move it out of discussions/archive/ to promote it")
                }
                Some(_) => {}
            }
            let change_name = name.unwrap_or_else(|| slug.clone());
            let schema = core::config::WorkflowConfig::from_text(
                store.read_workflow_config().as_deref(),
            )
            .schema_name();
            let dir = core::newcmd::new_change(
                &ws,
                store,
                &change_name,
                None,
                &schema,
                None,
                Some(&slug),
            )?;
            // Prefill the proposal's Why from the discussion conclusion (topic as fallback);
            // the remaining sections stay as TBD markers for /speclink-propose to complete.
            let why = core::discuss::conclusion_text(store, &slug).unwrap_or_else(|| {
                core::discuss::info(store, &slug)
                    .map(|i| i.topic)
                    .unwrap_or_else(|| slug.clone())
            });
            let proposal = format!(
                "## Why\n\n{why}\n\n## What Changes\n\n<!-- TBD: derive from the discussion -->\n\n## Capabilities\n\n### New Capabilities\n\n<!-- TBD -->\n\n## Impact\n\n<!-- TBD -->\n"
            );
            store.write_artifact(&change_name, "proposal.md", &proposal)?;
            core::discuss::mark_promoted(store, &slug, &change_name)?;
            if json {
                return print_json(&serde_json::json!({
                    "change": change_name,
                    "path": core::util::to_slash(&dir),
                    "slug": slug,
                    "status": "promoted",
                }));
            }
            println!("{} Promoted discussion '{slug}' → change '{change_name}'", color::green("✓"));
            println!("  Path: {}", dir.to_string_lossy());
            println!("  Proposal prefilled from the conclusion — run /speclink-propose to complete the artifacts");
        }
    }
    Ok(())
}
