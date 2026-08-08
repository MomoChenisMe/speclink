// Included into main.rs. Command handlers and rendering.

use core::store::Store;
use core::workspace::Workspace;

fn dispatch(cli: Cli) -> Result<()> {
    warn_deprecated_policy_keys();
    warn_leftover_remote_file();
    match cli.command {
        Commands::Init(a) => cmd_init(a),
        Commands::Update(a) => cmd_update(a),
        Commands::List(a) => cmd_list(a),
        Commands::Show(a) => cmd_show(a),
        Commands::Validate(a) => cmd_validate(a),
        Commands::Analyze(a) => cmd_analyze(a),
        Commands::Drift(a) => cmd_drift(a),
        Commands::Archive(a) => cmd_archive(a),
        Commands::Discard(a) => cmd_discard(a),
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
        Commands::WorkflowConfig(a) => cmd_workflow_config(a),
        Commands::Completion(a) => cmd_completion(a),
        Commands::Task(a) => cmd_task(a),
        Commands::InProgress(a) => cmd_in_progress(a),
        Commands::Demo => cmd_demo(),
        Commands::Discuss(a) => cmd_discuss(a),
        Commands::Review(a) => cmd_review(a),
        Commands::Verify(a) => cmd_verify(a),
    }
}

// --- claim ---

/// Claiming is an ownership concept of the remote lifecycle; the local fs
/// store has no claim state, so fs mode fails loud instead of pretending.
fn cmd_claim(a: ClaimArgs) -> Result<()> {
    match remote_ctx()? {
        Some(ctx) => remote_claim(&ctx, &a.name),
        // fs 模式是純拒絕、不觸 Store（在非專案目錄也同一句）——訊息與
        // runtime 的 Claim 分支共用同一份 frozen 文字（node dispatch 經該分支）。
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
            let (ws, store) = open_project()?;
            let store: &dyn Store = &store;
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::ArtifactCat { artifact, change },
            )?;
            let core::command::CommandOutcome::ArtifactCat(content) = outcome else {
                unreachable!("artifact cat yields raw content");
            };
            print!("{content}");
            Ok(())
        }
    }
}

fn cmd_language(a: LanguageArgs) -> Result<()> {
    match a.command {
        LanguageCommands::Show => {
            if let Some(ctx) = remote_ctx()? {
                print!("{}", ctx.client.language()?.content);
                return Ok(());
            }
            let (ws, store) = open_project()?;
            let store: &dyn Store = &store;
            let outcome = run_command(store, Some(&ws), core::command::Command::LanguageShow)?;
            let core::command::CommandOutcome::Language(content) = outcome else {
                unreachable!("language show yields raw content");
            };
            print!("{content}");
            Ok(())
        }
    }
}

// --- helpers ---

/// Deprecation signal for the legacy policy keys: exactly one fixed-prefix stderr line per
/// invocation when `.speclink.yaml` still carries keys whose canonical home is
/// `openspec/config.yaml`. stdout (including `--json`) stays untouched; no keys → no output.
fn warn_deprecated_policy_keys() {
    // Warning helpers stay silent on discovery/parse errors — the command's own
    // path surfaces the fail-closed config error with the proper exit code.
    let Ok(Some(ws)) = Workspace::discover_cwd() else {
        return;
    };
    let Ok(app) = core::config::AppConfig::load(&ws.app_config()) else {
        return;
    };
    let keys = app.deprecated_policy_keys();
    if keys.is_empty() {
        return;
    }
    eprintln!(
        "speclink: warning: deprecated policy keys in .speclink.yaml: {} (move them to openspec/config.yaml)",
        keys.join(", ")
    );
}

/// Migration signal for the abolished connection file: exactly one fixed-prefix stderr
/// line per invocation while `.speclink.remote.yaml` lingers in the project root. The
/// file is never parsed and never affects the store mode — an unmigrated project runs
/// in fs mode until the fields move. stdout and exit codes stay untouched.
fn warn_leftover_remote_file() {
    let Ok(Some(ws)) = Workspace::discover_cwd() else {
        return;
    };
    if !ws.has_leftover_remote_file() {
        return;
    }
    eprintln!(
        "speclink: warning: .speclink.remote.yaml is no longer read and does not affect the store mode — move its url/repo into the `remote:` section of .speclink.yaml and delete the old file"
    );
}

/// Route a command through the engine runtime. The typed error converts into
/// the anyhow error main() prints — `Error: {message}`, text frozen by the
/// regression baseline. Events are not consumed by the CLI (yet).
fn run_command(
    store: &dyn Store,
    ws: Option<&core::workspace::Workspace>,
    cmd: core::command::Command,
) -> Result<core::command::CommandOutcome> {
    // Host boundary: identity and the SPECLINK_* env layer are resolved here
    // and injected — the engine runtime only ever consumes the context.
    let ctx = core::command::ExecutionContext {
        actor: ws.and_then(|w| speclink_host::context::git_identity(&w.root)),
        repo: Some(speclink_host::binding::local_default_binding().repo.as_str().to_string()),
        env: speclink_host::policy::process_env_overrides(),
        workspace: ws.cloned(),
        user_config_dir: Some(speclink_host::context::global_config_dir()),
    };
    let (outcome, _events) = core::command::execute(store, &ctx, cmd).map_err(anyhow::Error::new)?;
    Ok(outcome)
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

// Shared with the Node SDK: the list serialization path lives in core::listing.
use core::listing::ListChangeJson;

// --- init / update ---

fn cmd_init(a: InitArgs) -> Result<()> {
    // The success line echoes the PATH argument verbatim (`init .` prints ".\openspec");
    // the absolute path is only used internally.
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
        "fs" | "remote" => {}
        other => bail!("Unknown store '{other}'. Use 'fs' or 'remote'."),
    }
    // Tools are resolved BEFORE the fs/remote split, so both paths start from the same
    // validated non-empty selection and no store writes anything until it is settled.
    let stdin = std::io::stdin();
    let interactive = stdin.is_terminal();
    let tools = resolve_init_tools(
        a.tools.as_deref(),
        interactive,
        &mut stdin.lock(),
        &mut std::io::stderr(),
    )?;
    if a.store == "remote" {
        return cmd_init_remote(&a, &root, &display_base, &tools);
    }
    let spec_dir = a.dir.clone().unwrap_or_else(|| "openspec".to_string());
    core::init::init(&root, &tools, a.force, &spec_dir)?;
    println!("{} Initialized at {display_base}{}{spec_dir}", color::green("✓"), std::path::MAIN_SEPARATOR);
    let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
    println!("Generated files for: {}", names.join(", "));
    Ok(())
}

/// The single line every missing/empty selection ends on — it names the flag and all
/// three valid values, so a failed non-interactive run is self-correcting.
const TOOLS_HINT: &str =
    "no AI tool selected — pass --tools claude, --tools codex, or --tools claude,codex";

/// Resolve init's built-in tool selection. An explicit `--tools` is validated and used
/// as-is (no prompt). Without the flag an interactive terminal is asked question by
/// question — prompts go to `out` (stderr in production) so stdout stays the machine
/// surface; a non-interactive terminal fails here, before any core write, and its
/// redirected stdin is never read as an answer.
fn resolve_init_tools(
    spec: Option<&str>,
    interactive: bool,
    input: &mut impl BufRead,
    out: &mut impl Write,
) -> Result<Vec<core::skills::Tool>> {
    match spec {
        Some(spec) => {
            let tools = core::init::parse_tools(spec)?;
            if tools.is_empty() {
                bail!("{TOOLS_HINT}");
            }
            Ok(tools)
        }
        None if interactive => prompt_for_tools(input, out),
        None => bail!("{TOOLS_HINT}"),
    }
}

/// Ask for Claude and Codex in turn, repeating the pair until at least one is picked —
/// an empty selection is not an answer. Plain text only: nothing here is styled, so
/// `--no-color` changes nothing about the prompts.
fn prompt_for_tools(
    input: &mut impl BufRead,
    out: &mut impl Write,
) -> Result<Vec<core::skills::Tool>> {
    use core::skills::Tool;
    loop {
        let mut picked = Vec::new();
        for (tool, label) in [(Tool::Claude, "Claude"), (Tool::Codex, "Codex")] {
            if ask_yes_no(input, out, label)? {
                picked.push(tool);
            }
        }
        if !picked.is_empty() {
            return Ok(picked);
        }
        writeln!(out, "Pick at least one tool: claude, codex, or both.")?;
    }
}

/// One yes/no question. Unrecognized input re-asks the same question; EOF is a loud
/// single-line error rather than an endless loop.
fn ask_yes_no(input: &mut impl BufRead, out: &mut impl Write, label: &str) -> Result<bool> {
    loop {
        write!(out, "Generate Speclink files for {label}? (y/n): ")?;
        out.flush()?;
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            bail!("{TOOLS_HINT}");
        }
        match line.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => writeln!(out, "Please answer y or n.")?,
        }
    }
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
    // 降級守門在引擎的 update 本體（唯一實作落點）；--allow-downgrade 是唯一越過。
    let outcome = core::init::update(&root, a.allow_downgrade)?;
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
    let (ws, fs_store) = open_project()?;
    // Worktree overlay (D3): only a local MAIN checkout with the policy on gets
    // here — everywhere else `facts` is empty, git is never spawned, and both
    // the store and the payload are exactly what they were before this feature.
    let facts = speclink_host::worktree::observed_facts(&ws, &fs_store, |key| std::env::var(key).ok());
    let overlaid = speclink_host::worktree::WorktreeOverlay::new(
        &fs_store,
        facts
            .iter()
            .map(|(name, e)| {
                let store: Box<dyn Store> =
                    Box::new(speclink_fs::FsStore::new(&e.path, &ws.spec_dir_name));
                (name.clone(), store)
            })
            .collect(),
    );
    let store: &dyn Store = if facts.is_empty() { &fs_store } else { &overlaid };
    let outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::List {
            sort: a.sort.clone(),
            specs: a.specs,
            changes: a.changes,
            worktrees: speclink_host::worktree::payload_objects(&facts),
        },
    )?;
    let core::command::CommandOutcome::List(list) = outcome else {
        unreachable!("list command yields a list outcome");
    };
    render_list(&list, a.json)
}

/// fs 與 remote 共用的 list 渲染：兩模式各自組出同一個 ListOutcome，輸出逐位元
/// 一致。模式差異只活在上游的資料組裝——remote 恆無 worktree 事實，其餘欄位同形。
fn render_list(list: &core::command::ListOutcome, json: bool) -> Result<()> {
    // --specs alone omits the changes section (outcome.changes is None).
    let Some(items) = &list.changes else {
        return render_specs_section(list.specs.as_ref().expect("specs requested"), json);
    };
    if json {
        if let Some(specs) = &list.specs {
            let mut payload = serde_json::Map::new();
            payload.insert("changes".into(), serde_json::to_value(items)?);
            payload.insert("specs".into(), specs.clone());
            return print_json(&serde_json::Value::Object(payload));
        }
        return print_json(&serde_json::json!({ "changes": items }));
    }
    if items.is_empty() {
        println!("No active changes.");
        if let Some(specs) = &list.specs {
            println!();
            render_specs_section(specs, false)?;
        }
        return Ok(());
    }
    println!("{}", color::bold("Changes:"));
    for c in items {
        // The progress marker is omitted entirely for changes with zero tasks.
        let marker = if c.total_tasks > 0 {
            format!(" [{}/{}]", c.completed_tasks, c.total_tasks)
        } else {
            String::new()
        };
        // The dim wrapper always prints — an empty summary yields the empty
        // \x1b[2m\x1b[0m pair in color mode and nothing in plain mode.
        let suffix = if c.summary.is_empty() {
            String::new()
        } else {
            format!(" — {}", c.summary)
        };
        // Fail-closed diagnostic (frozen marker text): only corrupt-metadata
        // lines gain the trailing marker; valid lines stay byte-identical.
        let invalid = if c.meta_error.is_some() {
            format!(" {}", color::red("(invalid .openspec.yaml)"))
        } else {
            String::new()
        };
        // Fixed literal, no color: `--no-color` must read exactly the same.
        let worktree = if c.worktree.is_some() { " [worktree]" } else { "" };
        println!(
            "  {} {}{marker}{}{invalid}{worktree}",
            color::cyan("•"),
            c.name,
            color::dim(&suffix)
        );
    }
    if let Some(specs) = &list.specs {
        println!();
        render_specs_section(specs, false)?;
    }
    Ok(())
}

/// The engine's specs payload built from the typed wire summaries. The
/// conversion belongs on this side of the include: the remote intercept layer
/// speaks protocol DTOs only (tests/it/no_raw_wire_json.rs), while the engine's
/// `ListOutcome` carries its specs section as an opaque payload.
fn wire_specs_payload(specs: Vec<protocol_query::SpecSummary>) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(specs)?)
}

/// Render the specs section from the outcome's `{id, path}` items.
fn render_specs_section(specs: &serde_json::Value, json: bool) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({ "specs": specs }));
    }
    let items = specs.as_array().map(Vec::as_slice).unwrap_or_default();
    if items.is_empty() {
        println!("No specs.");
        return Ok(());
    }
    println!("{}", color::bold("Specs:"));
    for s in items {
        println!("  {} {}", color::cyan("•"), s["id"].as_str().unwrap_or_default());
    }
    Ok(())
}

// --- show ---

fn cmd_show(a: ShowArgs) -> Result<()> {
    if let Some(ctx) = remote_ctx()? {
        let show = remote_show_outcome(&ctx, a.item.as_deref(), a.item_type.as_deref())?;
        return render_show(show, a.json);
    }
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    let outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::Show {
            item: a.item.clone(),
            item_type: a.item_type.clone(),
        },
    )?;
    let core::command::CommandOutcome::Show(show) = outcome else {
        unreachable!("show command yields a show outcome");
    };
    render_show(show, a.json)
}

/// fs 與 remote 共用的 show 渲染：兩模式餵進同一個 ShowOutcome，輸出逐位元一致。
fn render_show(show: core::command::ShowOutcome, json: bool) -> Result<()> {
    match show {
        core::command::ShowOutcome::Spec { name, content } => {
            if json {
                return print_json(&serde_json::json!({
                    "files": [{ "content": content, "name": "spec.md" }],
                    "name": name,
                }));
            }
            println!("{}: {name}", color::bold("Spec"));
            println!();
            println!("{}", color::dim("--- spec.md ---"));
            print!("{content}");
            println!(); // A trailing newline is always emitted (an extra blank when content ends with \n)
        }
        core::command::ShowOutcome::Change(c) => {
            let caps: Vec<String> = c
                .delta_capabilities
                .iter()
                .map(|cap| format!("{cap}/spec.md"))
                .collect();
            if json {
                return print_json(&serde_json::json!({
                    "name": c.name,
                    "schema": c.schema,
                    "created": c.created,
                    "proposal": c.proposal,
                    "design": c.design,
                    "tasks": c.tasks,
                    "deltaSpecs": caps,
                    "fromDiscussions": c.from_discussions,
                    "restaleFrom": c.restale_from,
                }));
            }
            println!("{}: {}", color::bold("Change"), c.name);
            if let Some(schema_name) = &c.schema {
                println!("{}: {schema_name}", color::bold("Schema"));
            }
            if let Some(created) = &c.created {
                println!("{}: {created}", color::bold("Created"));
            }
            // The header's trailing blank line prints only when a section follows.
            if c.proposal.is_some() || !caps.is_empty() {
                println!();
            }
            // The Proposal section renders whenever the FILE exists (even empty) — frozen behavior.
            if let Some(proposal) = &c.proposal {
                println!("{}", color::dim("--- Proposal ---"));
                print!("{proposal}");
                if !caps.is_empty() {
                    // The proposal's own trailing newline determines the blank-line count before the header.
                    print!("\n\n{}\n", color::dim("--- Delta Specs ---"));
                    for cap in &caps {
                        println!("  {cap}");
                    }
                } else {
                    // A newline is always appended after the proposal body (an extra blank
                    // line when the file already ends with one), same as the spec branch above.
                    println!();
                }
            } else if !caps.is_empty() {
                // No proposal, but delta specs exist — still render the section.
                println!("{}", color::dim("--- Delta Specs ---"));
                for cap in &caps {
                    println!("  {cap}");
                }
            }
        }
    }
    Ok(())
}

// --- validate ---

fn cmd_validate(a: ValidateArgs) -> Result<()> {
    if let Some(ctx) = remote_ctx()? {
        return remote_validate(&ctx, &a);
    }
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    let outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::Validate {
            item: a.item.clone(),
            all: a.all,
            changes: a.changes,
            strict: a.strict,
        },
    )?;
    let core::command::CommandOutcome::Validate(v) = outcome else {
        unreachable!("validate command yields a validate outcome");
    };
    render_validate_results(&v.results, a.json)
}

/// fs 與 remote 共用的 validate 渲染：--json 印完整結果陣列、人眼逐 change
/// 列 error/warn；任一 invalid 以「Validation failed.」非零收尾。
fn render_validate_results(results: &[core::validate::ValidationResult], json: bool) -> Result<()> {
    let any_invalid = results.iter().any(|r| !r.valid);
    if json {
        print_json(&results)?;
        if any_invalid {
            bail!("Validation failed.");
        }
        return Ok(());
    }
    for r in results {
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
    if let Some(ctx) = remote_ctx()? {
        return remote_analyze(&ctx, &a);
    }
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    if info_if_no_changes(store, a.change.as_deref()) {
        return Ok(());
    }
    let outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::Analyze { change: a.change.clone() },
    )?;
    let core::command::CommandOutcome::Analyze(report) = outcome else {
        unreachable!("analyze command yields an analyze outcome");
    };
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
        // The bold span covers a 14-wide padded name; the 15th column separator
        // space stays outside it (plain bytes are identical to the old {:<15} form).
        println!(
            "  {sym} {} {} ({} findings)",
            color::bold(&format!("{:<14}", d.dimension)),
            color::dim(&d.status),
            d.finding_count
        );
    }
    // The blank separator is tied to the "Analyzed:" line; an empty change (nothing analyzed)
    // prints "Missing:" directly after the dimensions (frozen output shape).
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
    if let Some(ctx) = remote_ctx()? {
        return remote_drift(&ctx, &a);
    }
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    if info_if_no_changes(store, a.change.as_deref()) {
        return Ok(());
    }
    // Client/server drift split, orchestrated locally: the Host collects the
    // workspace facts and fixes the basis; the Engine computes the spec side and
    // the workspace side; the single merger assembles the frozen report shape.
    let change = core::command::resolve_guarded_change(store, a.change.as_deref())
        .map_err(anyhow::Error::new)?;
    let binding = speclink_host::binding::local_default_binding();
    let bundle = speclink_host::drift::produce_drift_bundle(store, &ws, &change, &binding);
    let facts = speclink_host::drift::collect_workspace_facts(&ws, store, &change);
    let spec = core::drift::compute_spec_drift(store, &change);
    let workspace = core::drift::compute_workspace_drift(store, &change, Some(&facts));
    let basis = core::drift::DriftBasis {
        expected: bundle.basis_digests.clone(),
        current: core::tasks::current_basis_digests(store, &change.name),
    };
    let report = core::drift::merge_drift_reports(&change, spec, workspace, Some(&basis));
    if a.json {
        return print_json(&report);
    }
    render_drift(&report.report);
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
    // mark-tasks-complete 與封存語意（含 in-progress 標記不動）單點在 runtime。
    let outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::Archive {
            change: a.changes.first().cloned(),
            skip_specs: a.skip_specs,
            no_validate: a.no_validate,
            mark_tasks_complete: a.mark_tasks_complete,
            carry_review: a.carry_review,
            carry_verify: a.carry_verify,
        },
    )?;
    let core::command::CommandOutcome::Archive(outcome) = outcome else {
        unreachable!("archive yields an archive outcome");
    };
    print_archive_outcome(&outcome);
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
    for (slug, file) in &outcome.archived_discussions {
        println!("Discussion archived: {slug} → discussions/archive/{file}");
    }
    // 零證據提示（spec verify-evidence「archive trace 注入與零證據提示」）：一行、
    // 不擋人、不影響 exit code——純規格變更本來就掙不到證據，這行只是讓代理回頭
    // 確認是不是漏走了 apply。
    if !outcome.evidence_recorded {
        eprintln!(
            "note: no task evidence recorded for change '{}' — fine for spec-only changes; otherwise check that tasks went through apply",
            outcome.change_name
        );
    }
}

/// Bulk archive (speclink-specific). Semantics: archives in created-date order, skips
/// not-ready changes with a reason (never silently), and fail-fasts on the first actual
/// archive error with a three-part report. The work tree's state is irrelevant — @trace
/// no longer carries a file list, so a dirty file cannot leak into any archived change.
fn cmd_archive_bulk(ws: &Workspace, store: &dyn Store, a: &ArchiveArgs) -> Result<()> {
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
        // Readiness: nothing the merge gate would refuse (the pre-check reads the
        // engine's own judgement — spec archive-merge「過期判定單源共用」), valid,
        // tasks complete. --skip-specs bypasses spec application, so the gate never
        // runs there and the pre-check must not filter on it either.
        if !a.skip_specs {
            let refused = core::archive::merge_violations(store, &change.name);
            if !refused.is_empty() {
                skipped.push((
                    change.name.clone(),
                    format!(
                        "{} delta operation(s) archive would refuse — run /speclink-drift {}",
                        refused.len(),
                        change.name
                    ),
                ));
                continue;
            }
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
        let archive_cmd = core::command::Command::Archive {
            change: Some(change.name.clone()),
            skip_specs: a.skip_specs,
            no_validate: a.no_validate,
            mark_tasks_complete: a.mark_tasks_complete,
            carry_review: a.carry_review,
            carry_verify: a.carry_verify,
        };
        match run_command(store, Some(ws), archive_cmd) {
            Ok(core::command::CommandOutcome::Archive(outcome)) => {
                print_archive_outcome(&outcome);
                archived.push(outcome.change_name);
            }
            Ok(_) => unreachable!("archive yields an archive outcome"),
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

// --- discard ---

fn cmd_discard(a: DiscardArgs) -> Result<()> {
    if let Some(ctx) = remote_ctx()? {
        return remote_discard(&ctx, &a);
    }
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    let cmd_outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::Discard { change: a.change.clone(), force: a.force },
    )?;
    let core::command::CommandOutcome::Discard(outcome) = cmd_outcome else {
        unreachable!("discard yields a discard outcome");
    };
    render_discard(&outcome.change_name, &outcome.unlinked_discussions, a.json)
}

/// fs 與 remote 共用的 discard 渲染：--json 為 change＋unlinkedDiscussions、
/// 人眼列出每筆 unlink 後狀態。
fn render_discard(change: &str, unlinked: &[(String, String)], json: bool) -> Result<()> {
    if json {
        let discussions: Vec<serde_json::Value> = unlinked
            .iter()
            .map(|(slug, status)| serde_json::json!({ "slug": slug, "status": status }))
            .collect();
        return print_json(&serde_json::json!({
            "change": change,
            "unlinkedDiscussions": discussions,
        }));
    }
    println!("{} Discarded change: {change}", color::green("✓"));
    for (slug, status) in unlinked {
        println!("  Discussion unlinked: {slug} → {status}");
    }
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
    let outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::Status {
            change: a.change.clone(),
            schema: a.schema.clone(),
        },
    )?;
    let core::command::CommandOutcome::Status(report) = outcome else {
        unreachable!("status command yields a status outcome");
    };
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
    let outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::Instructions {
            artifact: a.artifact.clone(),
            change: a.change.clone(),
            schema: a.schema.clone(),
        },
    )?;
    let core::command::CommandOutcome::Instructions(instr) = outcome else {
        unreachable!("instructions command yields an instructions outcome");
    };
    match instr {
        core::command::InstructionsOutcome::Apply(payload) => {
            if a.json {
                return print_json(&payload);
            }
            render_apply_human(&payload);
        }
        core::command::InstructionsOutcome::Artifact(payload) => {
            if a.json {
                return print_json(&payload);
            }
            render_artifact_human(&payload);
        }
    }
    Ok(())
}

fn render_artifact_human(p: &core::instructions::ArtifactInstructions) {
    println!("{}: {}", color::bold("Artifact"), p.artifact_id);
    println!("{}: {}", color::bold("Output"), p.output_path);
    println!("{}: {}", color::bold("Description"), p.description);
    // Each section is preceded by one blank separator and rendered only when non-empty
    // (a custom schema may have no instruction and an empty template) — frozen output shape.
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
    let outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::NewChange {
            name: a.name.clone(),
            description: a.description.clone(),
            schema: a.schema.clone(),
            agent: a.agent.clone(),
            from_discussion: a.from_discussion.clone(),
        },
    )?;
    let core::command::CommandOutcome::NewChange(o) = outcome else {
        unreachable!("new change yields a new-change outcome");
    };
    render_new_change(
        &o.name,
        Some(&o.dir.to_string_lossy()),
        Some(&o.schema),
        a.from_discussion.as_deref(),
    );
    Ok(())
}

/// `path` absent is the remote mode's declared divergence (design D5): the
/// change's directory is a store-side location with no meaning on the caller's
/// machine. `schema` is likewise only printed when the answer carries one.
fn render_new_change(
    name: &str,
    path: Option<&str>,
    schema: Option<&str>,
    from_discussion: Option<&str>,
) {
    println!("{} Created change: {name}", color::green("✓"));
    if let Some(path) = path {
        println!("  Path: {path}");
    }
    if let Some(schema) = schema {
        println!("  Schema: {schema}");
    }
    if let Some(slug) = from_discussion {
        println!("  From discussion: {slug}");
    }
}

fn cmd_new_artifact(a: NewArtifactArgs) -> Result<()> {
    if let Some(ctx) = remote_ctx()? {
        return remote_new_artifact(&ctx, &a);
    }
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    let content = if a.stdin {
        Some(read_stdin())
    } else {
        None
    };
    let outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::NewArtifact {
            kind: a.artifact_type.clone(),
            capability: a.capability.clone(),
            change: a.change.clone(),
            content,
            force: a.force,
        },
    )?;
    let core::command::CommandOutcome::NewArtifact(o) = outcome else {
        unreachable!("new artifact yields a new-artifact outcome");
    };
    if a.json {
        // Compact single-line JSON, frozen shape ("artifact" echoes the
        // input token, not the schema artifact id).
        let v = serde_json::json!({
            "artifact": a.artifact_type,
            "change": o.change,
            "path": o.path.to_string_lossy(),
            "status": "created",
            "validated": o.had_content,
            "warnings": [],
        });
        println!("{}", serde_json::to_string(&v)?);
        return Ok(());
    }
    println!("{} Created {}: {}", color::green("✓"), a.artifact_type, o.path.to_string_lossy());
    if o.had_content {
        println!("  Content validated ✓");
    }
    Ok(())
}

// --- schemas / templates ---

fn cmd_schemas(a: JsonFlag) -> Result<()> {
    let ws = core::workspace::Workspace::discover_cwd()?;
    let schemas = core::schema::list_all(ws.as_ref(), Some(&speclink_host::context::global_config_dir()));
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
    let ws = core::workspace::Workspace::discover_cwd()?;
    let schema_name = a.schema.unwrap_or_else(|| "spec-driven".to_string());
    let schema = match core::schema::resolve_with(ws.as_ref(), Some(&speclink_host::context::global_config_dir()), &schema_name) {
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
    let ws = core::workspace::Workspace::discover_cwd()?;
    match a.command {
        SchemaCommands::Which { name, all: _, json } => {
            let n = name.unwrap_or_else(|| "spec-driven".to_string());
            let sources = core::schema::sources(ws.as_ref(), Some(&speclink_host::context::global_config_dir()), &n);
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
            match core::schema::resolve_with(ws.as_ref(), Some(&speclink_host::context::global_config_dir()), &n) {
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
            let new_name = core::schema::fork(&ws, Some(&speclink_host::context::global_config_dir()), &source, name.as_deref(), force)
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
    speclink_host::context::global_config_dir().join("config.yaml")
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
            // string storage (frozen behavior).
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
            // Printed whether or not the key existed (frozen behavior).
            println!("{} Removed key: {key}", color::green("✓"));
        }
        ConfigCommands::Reset { all: _, yes: _ } => {
            if path.exists() {
                std::fs::remove_file(&path)?;
            }
            println!("{} Config reset.", color::green("✓"));
        }
        ConfigCommands::Edit => {
            // VISUAL wins over EDITOR; the vi fallback and the failure message when no
            // editor can be spawned are both frozen.
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

// Insertion-ordered mapping (serde_yaml::Mapping) — the order keys were first set is
// preserved in both the stored YAML and `config list` output.
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

// --- workflow-config (openspec/config.yaml) ---

/// The policy keys `workflow-config set` accepts, in canonical order.
const POLICY_KEYS: [&str; 5] = ["locale", "spec_locale", "tdd", "audit", "worktree"];

/// `workflow-config show --json` payload. camelCase field names are the contract;
/// the values are CANONICAL (what the document says), never the four-layer
/// resolution — effective policy is the instructions payload's job.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowConfigJson {
    locale: Option<String>,
    spec_locale: Option<String>,
    tdd: bool,
    audit: bool,
    worktree: bool,
    context: Option<String>,
    rules: std::collections::BTreeMap<String, Vec<String>>,
}

/// A write subcommand once argv and stdin are resolved — the shape both the fs
/// and the remote branch hand to the shared rewrite.
enum WorkflowConfigWrite {
    Policy { key: String, value: String },
    Context(String),
    Rules { artifact: String, body: String },
}

/// The rewrite a write subcommand produces: the complete new document plus the
/// line printed on a successful write.
struct WorkflowConfigEdit {
    new_text: String,
    summary: String,
}

fn cmd_workflow_config(a: WorkflowConfigArgs) -> Result<()> {
    let json = matches!(a.command, WorkflowConfigCommands::Show { json: true });
    let write = workflow_config_write(&a.command)?;
    if let Some(ctx) = remote_ctx()? {
        return remote_workflow_config(&ctx, write, json);
    }
    let ws = require_workspace()?;
    let label = format!("{}/config.yaml", ws.spec_dir_name);
    let path = ws.spec_dir().join("config.yaml");
    let original = core::util::read_opt(&path).unwrap_or_default();
    let Some((write, dry_run)) = write else {
        return print_workflow_config(&original, &label, json);
    };
    let edit = plan_workflow_config_edit(&write, &original, &label, Some(&ws))?;
    if dry_run {
        print!("{}", unified_diff(&label, &original, &edit.new_text));
        return Ok(());
    }
    // worktree 的寫入牽動技能足跡，故三步有序（design D2）：先擋、再寫、後同步。
    // 擋下時整體不動；同步失敗時 config 已是正典，錯誤浮出並指向重跑 update。
    // 擋下只在「由 true 改 false」（政策已關時技能本就不在，殘留的 worktree
    // 沒有收尾工具被抽走的風險，那個 no-op 寫入不該被拒）。
    let worktree_target = worktree_write_target(&write);
    let worktree_was_on = serde_yaml::from_str::<core::config::WorkflowConfig>(&original)
        .map(|c| c.worktree.unwrap_or(false))
        .unwrap_or(false);
    if worktree_target == Some(false) && worktree_was_on {
        refuse_teardown_with_active_worktrees(&ws)?;
    }
    std::fs::write(&path, &edit.new_text).map_err(|e| anyhow::anyhow!("{label}: write failed: {e}"))?;
    println!("{} {}", color::green("✓"), edit.summary);
    if worktree_target.is_some() {
        let outcome = core::init::update(&ws.root, false).map_err(|e| {
            anyhow::anyhow!("{label} written, but the skill footprint did not sync: {e} — fix the cause above, then re-run `speclink update` to rebuild it")
        })?;
        println!(
            "{} skills synced ({})",
            color::green("✓"),
            if outcome.updated.is_empty() { "no tools configured".to_string() } else { outcome.updated.join(", ") }
        );
    }
    Ok(())
}

/// The `worktree` value a write is steering towards, or None when the write does
/// not touch that key — the trigger for both the teardown check and the sync.
fn worktree_write_target(write: &WorkflowConfigWrite) -> Option<bool> {
    match write {
        WorkflowConfigWrite::Policy { key, value } if key == "worktree" => {
            Some(matches!(value.trim(), "true"))
        }
        _ => None,
    }
}

/// Refuse to turn the policy off while linked worktrees are still open: doing so
/// would retire the merge skill they depend on. Fail-open cases (no git) list
/// nothing and let the write through.
fn refuse_teardown_with_active_worktrees(ws: &core::workspace::Workspace) -> Result<()> {
    let store = speclink_fs::FsStore::new(&ws.root, &ws.spec_dir_name);
    let blockers = speclink_host::worktree::teardown_blockers(ws, &store);
    if blockers.is_empty() {
        return Ok(());
    }
    let list: String = blockers
        .iter()
        .map(|b| format!("\n  - {} ({}) at {}", b.change, b.branch, b.path.display()))
        .collect();
    bail!(
        "worktree is still in use — turning the policy off would remove the merge skill these worktrees need:{list}\n\
Wrap each one up with `speclink-worktree-merge` first, then set worktree false."
    )
}

/// Split the parsed subcommand into `show` (None) or a resolved write plus its
/// `--dry-run` flag. stdin is consumed here, at the argv layer, so the rewrite
/// itself stays a pure text→text step shared by both modes.
fn workflow_config_write(
    cmd: &WorkflowConfigCommands,
) -> Result<Option<(WorkflowConfigWrite, bool)>> {
    Ok(match cmd {
        WorkflowConfigCommands::Show { .. } => None,
        WorkflowConfigCommands::Set { key, value, dry_run } => Some((
            WorkflowConfigWrite::Policy { key: key.clone(), value: value.clone() },
            *dry_run,
        )),
        WorkflowConfigCommands::Context { stdin, dry_run } => {
            require_stdin_flag(*stdin, "context --stdin")?;
            Some((WorkflowConfigWrite::Context(read_stdin()), *dry_run))
        }
        WorkflowConfigCommands::Rules { artifact, stdin, dry_run } => {
            require_stdin_flag(*stdin, &format!("rules {artifact} --stdin"))?;
            Some((
                WorkflowConfigWrite::Rules { artifact: artifact.clone(), body: read_stdin() },
                *dry_run,
            ))
        }
    })
}

/// Content-taking subcommands require the flag explicitly: without it the
/// command would silently write an empty document from an interactive terminal.
fn require_stdin_flag(flag: bool, usage: &str) -> Result<()> {
    if flag {
        return Ok(());
    }
    bail!("content is read from stdin — run: speclink workflow-config {usage}")
}

/// Render the canonical view of a workflow-config document.
fn print_workflow_config(original: &str, label: &str, json: bool) -> Result<()> {
    let cfg = core::config::WorkflowConfig::from_text(Some(original))
        .map_err(|e| anyhow::anyhow!("invalid {label}: {}", e.reason))?;
    if json {
        return print_json(&WorkflowConfigJson {
            locale: cfg.locale.clone(),
            spec_locale: cfg.spec_locale.clone(),
            tdd: cfg.tdd.unwrap_or(false),
            audit: cfg.audit.unwrap_or(false),
            worktree: cfg.worktree.unwrap_or(false),
            context: cfg.context_text(),
            rules: cfg.rules.clone(),
        });
    }
    println!("{} {label}", color::bold("Workflow config:"));
    println!();
    let locale = match cfg.locale.as_deref() {
        Some(v) => v.to_string(),
        None => "unset (English)".to_string(),
    };
    let spec_locale = match cfg.spec_locale.as_deref() {
        Some(v) => v.to_string(),
        None => "unset (specs in English)".to_string(),
    };
    println!("  {:<13}{locale}", "locale");
    println!("  {:<13}{spec_locale}", "spec_locale");
    println!("  {:<13}{}", "tdd", toggle_display(cfg.tdd));
    println!("  {:<13}{}", "audit", toggle_display(cfg.audit));
    println!("  {:<13}{}", "worktree", toggle_display(cfg.worktree));
    let context = match cfg.context_text() {
        Some(text) => format!("{} lines", text.lines().count()),
        None => "none".to_string(),
    };
    println!("  {:<13}{context}", "context");
    let rules: Vec<String> = cfg
        .rules
        .iter()
        .filter(|(_, entries)| !entries.is_empty())
        .map(|(artifact, entries)| format!("{artifact} {}", entries.len()))
        .collect();
    let rules = if rules.is_empty() { "none".to_string() } else { rules.join(", ") };
    println!("  {:<13}{rules}", "rules");
    Ok(())
}

/// A toggle's canonical display: `false` is never stored, so "not set" and
/// "set to false" are the same state — name the default so it reads as one.
fn toggle_display(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "on",
        Some(false) | None => "unset (off)",
    }
}

/// Apply one write to `original` through the core rewrite seam — the single
/// place fs and remote share, so their write semantics can never diverge.
/// Fails closed on an unparseable document: the read-modify-write would
/// otherwise destroy the user's content.
fn plan_workflow_config_edit(
    write: &WorkflowConfigWrite,
    original: &str,
    label: &str,
    ws: Option<&Workspace>,
) -> Result<WorkflowConfigEdit> {
    let current = core::config::WorkflowConfig::from_text(Some(original))
        .map_err(|e| anyhow::anyhow!("invalid {label}: {} — write refused", e.reason))?;
    // The seam takes the COMPLETE target state of the four policy keys, so the
    // current values are read back first and only the edited key moves.
    let mut fields = core::config::WorkflowPolicyFields {
        locale: current.locale.clone(),
        spec_locale: current.spec_locale.clone(),
        tdd: current.tdd.unwrap_or(false),
        audit: current.audit.unwrap_or(false),
        worktree: current.worktree.unwrap_or(false),
    };
    let mut context = core::config::ContextEdit::Keep;
    let mut rules: Option<Vec<(String, Vec<String>)>> = None;
    let summary = match write {
        WorkflowConfigWrite::Policy { key, value } => {
            set_policy_field(&mut fields, key, value)?;
            format!("{key} = {value}")
        }
        WorkflowConfigWrite::Context(text) => {
            let summary = if text.trim().is_empty() {
                "context removed".to_string()
            } else {
                format!("context set ({} lines)", text.trim_end().lines().count())
            };
            context = core::config::ContextEdit::Set(text.clone());
            summary
        }
        WorkflowConfigWrite::Rules { artifact, body } => {
            let artifacts = workflow_schema_artifacts(ws, &current);
            if artifacts.is_empty() {
                bail!("{}", core::schema::not_found_msg(&current.schema_name()));
            }
            if !artifacts.iter().any(|id| id == artifact) {
                bail!(
                    "Unknown artifact '{artifact}' for schema '{}'. Use one of: {}",
                    current.schema_name(),
                    artifacts.join(", ")
                );
            }
            let entries: Vec<String> = body
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();
            let summary = if entries.is_empty() {
                format!("rules.{artifact} removed")
            } else {
                format!("rules.{artifact} = {} entries", entries.len())
            };
            rules = Some(merged_rules(&current.rules, &artifacts, artifact, entries));
            summary
        }
    };
    let new_text =
        core::config::update_workflow_config_text(original, &fields, &context, rules.as_deref())?;
    Ok(WorkflowConfigEdit { new_text, summary })
}

/// Map one `set <key> <value>` onto the complete-target-state fields. An empty
/// locale value and `false` both mean "back to default" — the seam then removes
/// the key, keeping unset-means-default intact.
fn set_policy_field(
    fields: &mut core::config::WorkflowPolicyFields,
    key: &str,
    value: &str,
) -> Result<()> {
    let text = || {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    };
    match key {
        "locale" => fields.locale = text(),
        "spec_locale" => fields.spec_locale = text(),
        "tdd" => fields.tdd = policy_bool(key, value)?,
        "audit" => fields.audit = policy_bool(key, value)?,
        "worktree" => fields.worktree = policy_bool(key, value)?,
        _ => bail!("Unknown key '{key}'. Use one of: {}", POLICY_KEYS.join(", ")),
    }
    Ok(())
}

/// Toggles accept only `true`/`false` — "1", "yes" and friends are refused
/// rather than guessed at.
fn policy_bool(key: &str, value: &str) -> Result<bool> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => bail!("Value for '{key}' must be true or false (got '{value}')"),
    }
}

/// The active schema's artifact ids in display order — both the accepted keys
/// for `rules <artifact>` and the section order written back. Empty when the
/// schema cannot be resolved (the caller turns that into the not-found error).
fn workflow_schema_artifacts(
    ws: Option<&Workspace>,
    cfg: &core::config::WorkflowConfig,
) -> Vec<String> {
    let user_dir = speclink_host::context::global_config_dir();
    match core::schema::resolve_with(ws, Some(&user_dir), &cfg.schema_name()) {
        Some(Ok(schema)) => core::status::display_order(&schema)
            .into_iter()
            .map(|a| a.id.clone())
            .collect(),
        _ => Vec::new(),
    }
}

/// The complete rules map the seam replaces wholesale: the target section
/// swapped in, every other section carried over. Ordered by the schema's
/// artifact display order (the layout the desktop settings page also writes),
/// with any section outside the schema appended after so nothing is dropped.
/// Empty sections are removed by the seam.
fn merged_rules(
    current: &std::collections::BTreeMap<String, Vec<String>>,
    artifacts: &[String],
    target: &str,
    entries: Vec<String>,
) -> Vec<(String, Vec<String>)> {
    let mut keys: Vec<String> = artifacts.to_vec();
    for key in current.keys() {
        if !keys.contains(key) {
            keys.push(key.clone());
        }
    }
    let mut out = Vec::with_capacity(keys.len());
    for key in keys {
        let section = if key == target {
            entries.clone()
        } else {
            current.get(&key).cloned().unwrap_or_default()
        };
        out.push((key, section));
    }
    out
}

/// Unified diff over lines, generated here rather than shelled out to a system
/// `diff` (Windows has none): the changed span between the common prefix and
/// suffix, with up to three lines of context on each side. Empty when the two
/// texts are identical.
fn unified_diff(label: &str, old: &str, new: &str) -> String {
    if old == new {
        return String::new();
    }
    let o: Vec<&str> = old.lines().collect();
    let n: Vec<&str> = new.lines().collect();
    let mut pre = 0;
    while pre < o.len() && pre < n.len() && o[pre] == n[pre] {
        pre += 1;
    }
    let mut suf = 0;
    while suf < o.len() - pre && suf < n.len() - pre && o[o.len() - 1 - suf] == n[n.len() - 1 - suf]
    {
        suf += 1;
    }
    const CTX: usize = 3;
    let start = pre - pre.min(CTX);
    let o_end = o.len() - suf + suf.min(CTX);
    let n_end = n.len() - suf + suf.min(CTX);
    let mut out = format!("--- a/{label}\n+++ b/{label}\n");
    out.push_str(&format!(
        "@@ -{} +{} @@\n",
        hunk_range(start, o_end - start),
        hunk_range(start, n_end - start)
    ));
    for line in &o[start..pre] {
        out.push_str(&format!(" {line}\n"));
    }
    for line in &o[pre..o.len() - suf] {
        out.push_str(&format!("-{line}\n"));
    }
    for line in &n[pre..n.len() - suf] {
        out.push_str(&format!("+{line}\n"));
    }
    for line in &o[o.len() - suf..o_end] {
        out.push_str(&format!(" {line}\n"));
    }
    out
}

/// One side of a hunk header. An empty side is `0,0` by unified-diff convention
/// (there is no line 1 to point at).
fn hunk_range(start: usize, count: usize) -> String {
    if count == 0 {
        "0,0".to_string()
    } else {
        format!("{},{count}", start + 1)
    }
}

// --- completion ---

/// Validated display name for a completion shell. Elvish IS supported, but the error message
/// only lists the four common shells — frozen verbatim.
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
                // The frozen bash script (from an older clap_complete) offers positional
                // value names as completion candidates ("[CHANGE]", "<KEY>"); newer
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
            // The shell profile is never written to; guidance is printed instead.
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
/// line of a clap_complete bash script, matching the frozen older clap_complete output.
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
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::TaskDone { task_id: task_id.clone(), change },
            )?;
            let core::command::CommandOutcome::TaskDone(o) = outcome else {
                unreachable!("task done yields a task-flip outcome");
            };
            render_task_flip(
                TaskFlip::Done,
                &o.change,
                &o.task_id.to_string(),
                &o.task_id_arg,
                &o.description,
                o.already,
                json,
            )?;
        }
        TaskCommands::Undone { task_id, change, json } => {
            if let Some(ctx) = remote_ctx()? {
                return remote_task_undone(&ctx, &task_id, change.as_deref(), json);
            }
            let (ws, store) = open_project()?;
            let store: &dyn Store = &store;
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::TaskUndone { task_id: task_id.clone(), change },
            )?;
            let core::command::CommandOutcome::TaskUndone(o) = outcome else {
                unreachable!("task undone yields a task-flip outcome");
            };
            render_task_flip(
                TaskFlip::Undone,
                &o.change,
                &o.task_id.to_string(),
                &o.task_id_arg,
                &o.description,
                o.already,
                json,
            )?;
        }
    }
    Ok(())
}

/// `task done` 與 `task undone` 的輸出只差狀態字與動詞片語，共用一支渲染。
#[derive(Clone, Copy)]
enum TaskFlip {
    Done,
    Undone,
}

impl TaskFlip {
    fn status(self) -> &'static str {
        match self {
            TaskFlip::Done => "done",
            TaskFlip::Undone => "undone",
        }
    }

    /// 已是該狀態時的拒絕文字，與成功行的動詞片語。
    fn phrases(self) -> (&'static str, &'static str) {
        match self {
            TaskFlip::Done => ("is already done", "marked as done"),
            TaskFlip::Undone => ("is already not done", "marked as not done"),
        }
    }
}

/// `already` 維持現行錯誤結束（引擎已保證零檔案效果）。`--json` 是緊湊單行，
/// 欄位順序凍結，兩種翻轉對稱。
///
/// 兩個 id 不是同一個東西，也不能互相取代：`refused` 是拒絕訊息用的識別（fs
/// 給引擎解析出的序號），`arg` 是原始 argv，走 stdout 的兩條路都用它。remote
/// 只有 argv 一種，兩處都餵它。
fn render_task_flip(
    flip: TaskFlip,
    change: &str,
    refused: &str,
    arg: &str,
    description: &str,
    already: bool,
    json: bool,
) -> Result<()> {
    let (already_phrase, verb_phrase) = flip.phrases();
    if already {
        bail!("Task {refused} {already_phrase}");
    }
    if json {
        let v = serde_json::json!({
            "change": change,
            "status": flip.status(),
            "task_desc": description,
            "task_id": arg,
        });
        println!("{}", serde_json::to_string(&v)?);
        return Ok(());
    }
    println!("{} Task {arg} {verb_phrase}: {description}", color::green("✓"));
    Ok(())
}

/// 實際移除與「本來就沒開工」印不同的行——引擎與 wire 都據實回報，渲染只讀事實。
fn render_in_progress_remove(name: &str, removed: bool) {
    if removed {
        println!(
            "{} Removed the in-progress marker from '{name}' — back to proposed",
            color::green("✓")
        );
    } else {
        println!("Change '{name}' has no in-progress marker — already proposed");
    }
}

// --- in-progress ---

fn cmd_in_progress(a: InProgressArgs) -> Result<()> {
    match a.command {
        InProgressCommands::Add { name } => {
            // remote 模式路由至 server（started_by 由 server 認證身分蓋章）；
            // 靜默 exit 0 的 parity 凍結形狀兩模式一致。
            if let Some(ctx) = remote_ctx()? {
                ctx.client.in_progress_add(&name)?;
                return Ok(());
            }
            let (ws, store) = open_project()?;
            let store: &dyn Store = &store;
            run_command(store, Some(&ws), core::command::Command::InProgressAdd { name })?;
        }
        InProgressCommands::Remove { name } => {
            if let Some(ctx) = remote_ctx()? {
                return remote_in_progress_remove(&ctx, &name);
            }
            let (ws, store) = open_project()?;
            let store: &dyn Store = &store;
            let outcome =
                run_command(store, Some(&ws), core::command::Command::InProgressRemove { name })?;
            let core::command::CommandOutcome::InProgressRemove(o) = outcome else {
                unreachable!("in-progress remove yields an in-progress-remove outcome");
            };
            render_in_progress_remove(&o.name, o.removed);
        }
    }
    Ok(())
}

// --- demo ---

fn cmd_demo() -> Result<()> {
    // 本質本機動詞：remote 模式明確拒絕（design D7，比照 claim 在 fs 的
    // fail-loud）。只判斷連線設定、不走 handshake——離線同樣拒絕、server
    // 零請求。
    if let Some(ws) = core::workspace::Workspace::discover_cwd()? {
        if matches!(
            speclink_host::context::resolve_store_mode(&ws)?.mode,
            core::workspace::StoreMode::Remote(_)
        ) {
            bail!("demo is not available in remote mode — it seeds a demo change into a local openspec/ tree");
        }
    }
    let (ws, store) = open_project()?;
    let outcome = core::demo::generate(&store, speclink_host::context::git_identity(&ws.root).as_deref())?;
    println!("{} Created demo change: {}", color::green("✓"), outcome.name);
    println!("  Theme: {}", outcome.theme);
    println!("  Path: {}", core::util::to_slash(&outcome.path));
    Ok(())
}

// --- discuss ---

// --- review 品質站 ---

/// 兩個品質站共用的動詞形狀（design D1 的 CLI 面）：clap 的兩個子命令 enum
/// 各自保留自己的說明文字與旗標可用性（verify 無 `prepare`），在此正規化為
/// 同一組動詞，往下只有一份實作。
pub(crate) enum StationVerb {
    Scope {
        change: String,
        json: bool,
        base: Option<String>,
        candidate_hash: Option<String>,
        include_hunk: Vec<String>,
    },
    AddRound {
        change: String,
        stdin: bool,
    },
    Show {
        change: String,
        json: bool,
    },
    Stamp {
        change: String,
        accept: bool,
        agent: Option<String>,
    },
    Discard {
        change: String,
    },
}

/// 一個品質站在 CLI 這層的全部站別差異：引擎常數組（工單檔名、meta 前綴、
/// 訊息用詞）與 host-local snapshot namespace。
pub(crate) struct StationCli {
    pub station: &'static core::station::Station,
    pub ns: speclink_host::change_diff::StationNs,
}

pub(crate) const REVIEW_CLI: StationCli = StationCli {
    station: &core::review::STATION,
    ns: speclink_host::change_diff::StationNs::Review,
};

pub(crate) const VERIFY_CLI: StationCli = StationCli {
    station: &core::verify::STATION,
    ns: speclink_host::change_diff::StationNs::Verify,
};

fn cmd_review(a: ReviewArgs) -> Result<()> {
    let verb = match a.command {
        ReviewCommands::Prepare { change } => return review_prepare(change),
        ReviewCommands::Scope { change, json, base, candidate_hash, include_hunk } => {
            StationVerb::Scope { change, json, base, candidate_hash, include_hunk }
        }
        ReviewCommands::AddRound { change, stdin } => StationVerb::AddRound { change, stdin },
        ReviewCommands::Show { change, json } => StationVerb::Show { change, json },
        ReviewCommands::Stamp { change, accept, agent } => {
            StationVerb::Stamp { change, accept, agent }
        }
        ReviewCommands::Discard { change } => StationVerb::Discard { change },
    };
    run_station(&REVIEW_CLI, verb)
}

fn cmd_verify(a: VerifyArgs) -> Result<()> {
    let verb = match a.command {
        VerifyCommands::Scope { change, json, base, candidate_hash, include_hunk } => {
            StationVerb::Scope { change, json, base, candidate_hash, include_hunk }
        }
        VerifyCommands::AddRound { change, stdin } => StationVerb::AddRound { change, stdin },
        VerifyCommands::Show { change, json } => StationVerb::Show { change, json },
        VerifyCommands::Stamp { change, accept, agent } => {
            StationVerb::Stamp { change, accept, agent }
        }
        VerifyCommands::Discard { change } => StationVerb::Discard { change },
    };
    run_station(&VERIFY_CLI, verb)
}

/// `review prepare` 的入口（驗證站無此動詞：Apply baseline 兩站共用）。
fn review_prepare(change: String) -> Result<()> {
    if let Some(ctx) = remote_ctx()? {
        return remote_review_prepare(&ctx, change);
    }
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    if !store.change_exists(&change) {
        bail!("change not found: {change}");
    }
    // started 判讀走 fail-closed 解析：壞 meta 不得被讀作「未開始」。
    let raw_meta = store.read_change_meta(&change);
    let meta = core::model::ChangeMeta::from_text(raw_meta.as_deref())
        .map_err(|e| anyhow::anyhow!(e))?;
    run_review_prepare(&ws, &change, meta.started_at.is_some())
}

fn run_station(cli: &StationCli, verb: StationVerb) -> Result<()> {
    if let Some(ctx) = remote_ctx()? {
        return remote_station(&ctx, cli, verb);
    }
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    let st = cli.station;
    match verb {
        StationVerb::Scope { change, json, base, candidate_hash, include_hunk } => {
            if !store.change_exists(&change) {
                bail!("change not found: {change}");
            }
            let ticket = store
                .artifact_exists(&change, st.doc)
                .then(|| core::station::show(st, store, &change))
                .transpose()?
                .map(|t| speclink_host::change_diff::TicketBinding {
                    patch_hash_chain: patch_hash_chain(
                        t.rounds.iter().map(|r| r.patch_hash.as_deref()),
                    ),
                    finding_paths: t
                        .last_round()
                        .findings
                        .iter()
                        .map(|f| f.path.clone())
                        .collect(),
                });
            let names = store.list_changes().into_iter().map(|c| c.name).collect();
            let req = build_scope_request(
                &ws,
                change,
                names,
                ticket,
                base,
                candidate_hash,
                include_hunk,
                cli.ns,
            );
            run_station_scope(&ws, st, &req, json)?;
        }
        StationVerb::AddRound { change, stdin } => {
            let content = read_stdin_content(stdin);
            let round = core::station::add_round(st, store, &change, &content)?;
            render_station_action(st, StationAction::AddRound(round), &change);
        }
        StationVerb::Show { change, json } => {
            let ticket = core::station::show(st, store, &change)?;
            // 人眼路徑印工單原文（show 已驗證存在與格式）。
            let doc = store
                .read_artifact(&change, st.doc)
                .expect("show verified the ticket exists");
            return render_station_show(st, &change, &ticket, Some(&doc), json);
        }
        StationVerb::Stamp { change, accept, agent } => {
            let actor = speclink_host::context::git_identity(&ws.root);
            let read_file = |p: &str| std::fs::read_to_string(ws.root.join(p)).ok();
            let file_exists = |p: &str| ws.root.join(p).is_file();
            core::station::stamp(
                st,
                store,
                &change,
                accept,
                actor.as_deref(),
                agent.as_deref(),
                &read_file,
                &file_exists,
            )?;
            render_station_action(st, StationAction::Stamp, &change);
            clear_snapshots_warning(&ws, st, cli.ns, &change);
        }
        StationVerb::Discard { change } => {
            core::station::discard(st, store, &change)?;
            render_station_action(st, StationAction::Discard, &change);
            clear_snapshots_warning(&ws, st, cli.ns, &change);
        }
    }
    Ok(())
}

/// 兩站三個「動作完成」動詞的成功行——只差站名片語，共用一支渲染。
fn render_station_action(st: &core::station::Station, action: StationAction, change: &str) {
    let (verb, tail) = match action {
        StationAction::AddRound(round) => {
            println!(
                "{} Recorded {} Round {round} for change '{change}'",
                color::green("✓"),
                st.noun_phrase
            );
            return;
        }
        StationAction::Stamp => ("Stamped", ""),
        StationAction::Discard => ("Discarded", " ticket"),
    };
    println!("{} {verb} {}{tail} for change '{change}'", color::green("✓"), st.noun_phrase);
}

enum StationAction {
    AddRound(usize),
    Stamp,
    Discard,
}

/// 工單閱讀的共用渲染。`--json` 一律走 `ticket_json`——payload 的欄位集合是
/// 對外契約，工單原文不屬於它，所以原文只走人眼路徑。`doc` 缺席是 remote 對
/// 舊 server 的退化：印結構化摘要，而不是拿結構化欄位反推一份假原文。
fn render_station_show(
    st: &core::station::Station,
    change: &str,
    ticket: &core::station::Ticket,
    doc: Option<&str>,
    json: bool,
) -> Result<()> {
    if json {
        return print_json(&ticket_json(change, ticket));
    }
    if let Some(doc) = doc {
        print!("{doc}");
        return Ok(());
    }
    println!("{} — {change}", st.title);
    for r in &ticket.rounds {
        println!("\nRound {}", r.index);
        if let Some(phase) = &r.phase {
            println!("  Phase: {}", phase.as_str());
        }
        if let Some(hash) = &r.patch_hash {
            println!("  Patch: {hash}");
        }
        if !r.scope.is_empty() {
            println!("  Scope: {}", r.scope.join(", "));
        }
        for f in &r.findings {
            println!("  - [{}] {} — {}", f.severity.as_str(), f.path, f.text);
        }
    }
    Ok(())
}

/// 工單的 `--json` payload（local／remote 之外，兩站也共用同一份組裝——欄位集合
/// 與 null 語意是對外契約）。
pub(crate) fn ticket_json(change: &str, ticket: &core::station::Ticket) -> serde_json::Value {
    let round_json = |r: &core::station::Round| {
        serde_json::json!({
            "index": r.index,
            "phase": r.phase.map(|p| p.as_str()),
            "patchHash": r.patch_hash,
            "scope": r.scope,
            "findings": r
                .findings
                .iter()
                .map(|f| serde_json::json!({
                    "severity": f.severity.as_str(),
                    "path": f.path,
                    "text": f.text,
                }))
                .collect::<Vec<_>>(),
        })
    };
    serde_json::json!({
        "change": change,
        "rounds": ticket.rounds.iter().map(round_json).collect::<Vec<_>>(),
        "lastRound": round_json(ticket.last_round()),
    })
}

/// `review prepare` 的唯一實作（local／remote 共用）：sidecar 全在本地
/// checkout，只有「是否已開工」這件事實由各自的 store 提供。
///
/// initial／kept 靜默（spec：stdout 為空）；late／unavailable 以 stderr 警告但
/// 仍 exit 0，讓 apply 可以繼續。
pub(crate) fn run_review_prepare(
    ws: &core::workspace::Workspace,
    change: &str,
    started: bool,
) -> Result<()> {
    match speclink_host::change_diff::prepare(ws, change, started)? {
        speclink_host::change_diff::PrepareOutcome::Captured(_)
        | speclink_host::change_diff::PrepareOutcome::KeptExisting(_) => {}
        speclink_host::change_diff::PrepareOutcome::Late(_) => eprintln!(
            "Warning: baseline for '{change}' was captured late (the change already started) \
             — review scope will need an explicit trusted --base fixed point"
        ),
        speclink_host::change_diff::PrepareOutcome::Unavailable(_) => eprintln!(
            "Warning: no git checkout found — baseline recorded as unavailable; review scope \
             will need an explicit trusted --base fixed point"
        ),
    }
    Ok(())
}

/// 工單各輪的 patchHash 鏈（新→舊）——validation 回走重建 adjacent 段的依據。
/// 末輪沒有 patchHash（legacy 輪）時回空鏈：驗證輪據此 fail closed，不拿更早
/// 一輪的快照冒充末輪。
pub(crate) fn patch_hash_chain<'a>(
    rounds: impl DoubleEndedIterator<Item = Option<&'a str>>,
) -> Vec<String> {
    let mut newest_first = rounds.rev();
    match newest_first.next() {
        Some(Some(last)) => {
            std::iter::once(last).chain(newest_first.flatten()).map(str::to_string).collect()
        }
        _ => Vec::new(),
    }
}

/// `review scope` 的請求組裝（local／remote 共用）：touched 記錄與重疊認領都是
/// host-local 事實，只有 change 清單與工單由各自的 store 提供。
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_scope_request(
    ws: &core::workspace::Workspace,
    change: String,
    other_change_names: Vec<String>,
    ticket: Option<speclink_host::change_diff::TicketBinding>,
    base: Option<String>,
    candidate_hash: Option<String>,
    include_hunks: Vec<String>,
    station: speclink_host::change_diff::StationNs,
) -> speclink_host::change_diff::ScopeRequest {
    let touched_paths = core::tasks::TouchedRecord::load(ws, &change).all_files();
    // 其他 active change 的 host-local touched 認領（overlap 守門）。
    let other_claims = other_change_names
        .into_iter()
        .filter(|name| *name != change)
        .filter_map(|name| {
            let paths = core::tasks::TouchedRecord::load(ws, &name).all_files();
            (!paths.is_empty())
                .then_some(speclink_host::change_diff::ActiveClaim { change: name, paths })
        })
        .collect();
    speclink_host::change_diff::ScopeRequest {
        change,
        touched_paths,
        other_claims,
        ticket,
        base_override: base,
        candidate_hash,
        include_hunks,
        station,
    }
}

/// scope 的解析與呈現（local／remote／兩站共用——resolved payload 逐位元同形的
/// 唯一保證）。needsInput 印 JSON（--json 時）後以非零收場。
pub(crate) fn run_station_scope(
    ws: &core::workspace::Workspace,
    st: &core::station::Station,
    req: &speclink_host::change_diff::ScopeRequest,
    json: bool,
) -> Result<()> {
    match speclink_host::change_diff::resolve_scope(ws, req)? {
        speclink_host::change_diff::ScopeOutcome::Resolved(r) => {
            if json {
                return print_json(&serde_json::json!({
                    "change": r.change,
                    "phase": r.phase.as_str(),
                    "state": "resolved",
                    "baseCommit": r.base_commit,
                    "candidateHash": r.candidate_hash,
                    "patchHash": r.patch_hash,
                    "paths": r.paths,
                    "files": r.files,
                    "patch": r.patch,
                    "outOfScopeChanged": r.out_of_scope_changed,
                }));
            }
            let hunk_count: usize = r.files.iter().map(|f| f.hunks.len()).sum();
            println!(
                "{} Frozen {} scope for change '{}'",
                color::green("✓"),
                r.phase.as_str(),
                r.change
            );
            println!("  Patch: {}", r.patch_hash);
            println!(
                "  Scope: {} file(s), {} hunk(s){}",
                r.paths.len(),
                hunk_count,
                attribution_breakdown(&r.files)
            );
            // 範圍外變動＝從未進本站檢查面的候選檔又動了：轉知使用者，不入檢查面。
            if !r.out_of_scope_changed.is_empty() {
                println!(
                    "  Changed outside the {} scope: {}",
                    st.noun_phrase,
                    r.out_of_scope_changed.join(", ")
                );
            }
            Ok(())
        }
        speclink_host::change_diff::ScopeOutcome::NeedsInput(n) => {
            if json {
                print_json(&serde_json::json!({
                    "change": n.change,
                    "phase": n.phase.as_str(),
                    "state": "needsInput",
                    "candidateHash": n.candidate_hash,
                    "ambiguousPaths": n.ambiguous_paths,
                    "files": n.files,
                }))?;
            }
            bail!("{}", scope_needs_input_message(st, &n));
        }
    }
}

/// 驗證輪計數行的三類出身補述（design D4）：discovery 沒有上輪可歸因，回空字串。
fn attribution_breakdown(files: &[speclink_host::change_diff::FileDelta]) -> String {
    use speclink_host::change_diff::Attribution;
    let count = |a: Attribution| files.iter().filter(|f| f.attribution == Some(a)).count();
    let (finding, adjacent, new) =
        (count(Attribution::Finding), count(Attribution::Adjacent), count(Attribution::New));
    if finding + adjacent + new == 0 {
        return String::new();
    }
    format!(" — {finding} finding, {adjacent} adjacent, {new} new")
}

/// stamp／discard 後清本站的 host-local snapshots（Apply baseline 與另一站的
/// snapshots 保留——design D8）。清除失敗僅警告——canonical 工單／metadata
/// mutation 已完成，不回滾。
fn clear_snapshots_warning(
    ws: &core::workspace::Workspace,
    st: &core::station::Station,
    ns: speclink_host::change_diff::StationNs,
    change: &str,
) {
    if let Err(e) = speclink_host::change_diff::clear_snapshots(ws, change, ns) {
        eprintln!("Warning: could not clear {} snapshots for '{change}': {e}", st.noun_phrase);
    }
}

/// needsInput 的 stderr 說明：原因、ambiguous paths 與三種處置（可信 --base、
/// hash-pinned --include-hunk、隔離 worktree）。
fn scope_needs_input_message(
    st: &core::station::Station,
    n: &speclink_host::change_diff::ScopeNeedsInput,
) -> String {
    use speclink_host::change_diff::AmbiguityReason;
    let mut lines = vec![format!("{} scope for '{}' needs input:", st.noun, n.change)];
    for reason in &n.reasons {
        lines.push(match reason {
            AmbiguityReason::BaselineMissing => {
                "  - no Apply baseline was captured for this change".to_string()
            }
            AmbiguityReason::BaselineLate => {
                "  - the baseline was captured late (change already started)".to_string()
            }
            AmbiguityReason::BaselineUnavailable => {
                "  - the baseline has no usable git fixed point".to_string()
            }
            AmbiguityReason::BaseUnresolvable(e) => format!("  - {e}"),
            AmbiguityReason::DirtyAtStart(paths) => {
                format!("  - touched paths were already dirty at start: {}", paths.join(", "))
            }
            AmbiguityReason::ActiveOverlap { change, paths } => format!(
                "  - active change '{change}' also claims: {}",
                paths.join(", ")
            ),
            AmbiguityReason::EmptyTouched => {
                "  - no touched files recorded — the whole worktree is never auto-reviewed"
                    .to_string()
            }
        });
    }
    if !n.ambiguous_paths.is_empty() {
        lines.push(format!("  ambiguous paths: {}", n.ambiguous_paths.join(", ")));
    }
    lines.push("resolve it explicitly by one of:".to_string());
    // Only discovery ever needs input: a validation round resolves its scope by
    // content movement against the frozen snapshot chain, so all three
    // disposals here address a discovery fixed point.
    lines.push("  1. pass a trusted fixed point with --base <rev>".to_string());
    lines.push(
        "  2. pin hunks with --candidate-hash <sha256> and --include-hunk <id> (repeatable)"
            .to_string(),
    );
    lines.push("  3. redo the work in an isolated worktree".to_string());
    lines.join("\n")
}

fn cmd_discuss(a: DiscussArgs) -> Result<()> {
    if let Some(ctx) = remote_ctx()? {
        return remote_discuss(&ctx, a);
    }
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    match a.command {
        DiscussCommands::New { topic, slug, kind, json } => {
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::DiscussNew { topic, slug, kind },
            )?;
            let core::command::CommandOutcome::DiscussNew(info) = outcome else {
                unreachable!("discuss new yields a discussion info");
            };
            render_discuss_new(&info, json)?;
        }
        DiscussCommands::List { archived, json } => {
            let outcome =
                run_command(store, Some(&ws), core::command::Command::DiscussList { archived })?;
            let core::command::CommandOutcome::DiscussList(items) = outcome else {
                unreachable!("discuss list yields a discussion list");
            };
            render_discuss_list(&items, archived, json)?;
        }
        DiscussCommands::Show { slug, json } => {
            let outcome =
                run_command(store, Some(&ws), core::command::Command::DiscussShow { slug })?;
            let core::command::CommandOutcome::DiscussShow(show) = outcome else {
                unreachable!("discuss show yields a discussion document");
            };
            render_discuss_show(&show, json)?;
        }
        DiscussCommands::Context { slug, stdin, json } => {
            let content = read_stdin_content(stdin);
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::DiscussContext { slug, content },
            )?;
            let core::command::CommandOutcome::DiscussContext(o) = outcome else {
                unreachable!("discuss context yields a subject outcome");
            };
            render_discuss_context(&o.slug, json)?;
        }
        DiscussCommands::AddRound { slug, mode, stdin, json } => {
            let content = read_stdin_content(stdin);
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::DiscussAddRound { slug, mode, content },
            )?;
            let core::command::CommandOutcome::DiscussAddRound(o) = outcome else {
                unreachable!("discuss add-round yields a round outcome");
            };
            render_discuss_add_round(&o.slug, o.round, &o.mode, json)?;
        }
        DiscussCommands::Conclude { slug, stdin, json } => {
            let content = read_stdin_content(stdin);
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::DiscussConclude { slug, content },
            )?;
            let core::command::CommandOutcome::DiscussConclude(o) = outcome else {
                unreachable!("discuss conclude yields a conclude outcome");
            };
            render_discuss_conclude(&o.slug, &o.restale_flagged, json)?;
        }
        DiscussCommands::Archive { slug, json } => {
            let outcome =
                run_command(store, Some(&ws), core::command::Command::DiscussArchive { slug })?;
            let core::command::CommandOutcome::DiscussArchive(o) = outcome else {
                unreachable!("discuss archive yields an archive outcome");
            };
            render_discuss_archive(
                &o.slug,
                &format!("discussions/archive/{}", o.archived_file),
                json,
            )?;
        }
        DiscussCommands::Discard { slug, force, json } => {
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::DiscussDiscard { slug, force },
            )?;
            let core::command::CommandOutcome::DiscussDiscard(o) = outcome else {
                unreachable!("discuss discard yields a subject outcome");
            };
            render_discuss_discard(&o.slug, json)?;
        }
        DiscussCommands::Promote { slug, name, json } => {
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::DiscussPromote { slug, name },
            )?;
            let core::command::CommandOutcome::DiscussPromote(o) = outcome else {
                unreachable!("discuss promote yields a promote outcome");
            };
            let shown = o.path.to_string_lossy();
            let wire = core::util::to_slash(&o.path);
            render_discuss_promote(
                &o.slug,
                &o.change,
                Some(PromotedPath { shown: &shown, wire: &wire }),
                json,
            )?;
        }
        DiscussCommands::Link { slug, change, json } => {
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::DiscussLink { slug, change },
            )?;
            let core::command::CommandOutcome::DiscussLink(o) = outcome else {
                unreachable!("discuss link yields a bind outcome");
            };
            render_discuss_bind(&o.slug, &o.change, DiscussBind::Link, json)?;
        }
        DiscussCommands::Seal { slug, change, json } => {
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::DiscussSeal { slug, change },
            )?;
            let core::command::CommandOutcome::DiscussSeal(o) = outcome else {
                unreachable!("discuss seal yields a bind outcome");
            };
            render_discuss_bind(&o.slug, &o.change, DiscussBind::Seal, json)?;
        }
    }
    Ok(())
}

// --- discuss 的共用渲染：每個子指令一支，fs 與 remote 餵同一份事實 ---

/// `link` 與 `seal` 的輸出只差動詞與尾註，共用一支渲染。
#[derive(Clone, Copy)]
enum DiscussBind {
    Link,
    Seal,
}

impl DiscussBind {
    fn status(self) -> &'static str {
        match self {
            DiscussBind::Link => "linked",
            DiscussBind::Seal => "sealed",
        }
    }

    fn line(self) -> (&'static str, &'static str) {
        match self {
            DiscussBind::Link => ("Linked", ""),
            DiscussBind::Seal => ("Sealed", " (marked promoted)"),
        }
    }
}

fn render_discuss_new(info: &core::discuss::DiscussionInfo, json: bool) -> Result<()> {
    if json {
        return print_json(info);
    }
    println!("{} Created discussion: {}", color::green("✓"), info.slug);
    println!("  Topic: {}", info.topic);
    println!("  Path: {}", info.path);
    Ok(())
}

fn render_discuss_list(
    items: &[core::discuss::DiscussionInfo],
    archived: bool,
    json: bool,
) -> Result<()> {
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
    for d in items {
        println!("  • {} [{}] ({} rounds) — {}", d.slug, d.status, d.rounds, d.topic);
    }
    Ok(())
}

fn render_discuss_show(show: &core::command::DiscussShowOutcome, json: bool) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({ "info": show.info, "content": show.content }));
    }
    print!("{}", show.content);
    Ok(())
}

fn render_discuss_context(slug: &str, json: bool) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({ "slug": slug, "context": "set" }));
    }
    println!("{} Set context for discussion '{slug}'", color::green("✓"));
    Ok(())
}

fn render_discuss_add_round(slug: &str, round: usize, mode: &str, json: bool) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({ "slug": slug, "round": round, "mode": mode }));
    }
    println!("{} Recorded round {round} ({mode}) to discussion '{slug}'", color::green("✓"));
    Ok(())
}

fn render_discuss_conclude(slug: &str, flagged: &[String], json: bool) -> Result<()> {
    if json {
        // Byte-identical to before when nothing was flagged (promoted_to empty);
        // the array appears only when a re-conclude actually staled changes.
        let mut payload = serde_json::json!({ "slug": slug, "status": "concluded" });
        if !flagged.is_empty() {
            payload["restaleFlagged"] = serde_json::json!(flagged);
        }
        return print_json(&payload);
    }
    println!("{} Concluded discussion '{slug}'", color::green("✓"));
    if !flagged.is_empty() {
        println!("  Flagged {} change(s) for re-ingest: {}", flagged.len(), flagged.join(", "));
    }
    Ok(())
}

fn render_discuss_archive(slug: &str, archived_to: &str, json: bool) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({ "slug": slug, "archived_to": archived_to }));
    }
    println!("{} Archived discussion: {slug} → {archived_to}", color::green("✓"));
    Ok(())
}

fn render_discuss_discard(slug: &str, json: bool) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({ "slug": slug, "status": "discarded" }));
    }
    println!("{} Discarded discussion: {slug}", color::green("✓"));
    Ok(())
}

/// The new change's directory in both spellings: the human line keeps the
/// platform separators the fs path was built with, while `--json` (and the
/// wire) is always slashed. On Unix the two are identical; on Windows they
/// are not, and the frozen fs output depends on the difference.
struct PromotedPath<'a> {
    shown: &'a str,
    wire: &'a str,
}

/// `path` absent is the remote mode's declared divergence (design D5): the new
/// change's directory is a store-side location with no meaning on the caller's
/// machine, so the Path line is dropped rather than invented. The follow-up
/// hint applies to both modes and always prints.
fn render_discuss_promote(
    slug: &str,
    change: &str,
    path: Option<PromotedPath<'_>>,
    json: bool,
) -> Result<()> {
    if json {
        let mut payload = serde_json::json!({
            "change": change,
            "slug": slug,
            "status": "promoted",
        });
        if let Some(path) = &path {
            payload["path"] = serde_json::json!(path.wire);
        }
        return print_json(&payload);
    }
    println!("{} Promoted discussion '{slug}' → change '{change}'", color::green("✓"));
    if let Some(path) = &path {
        println!("  Path: {}", path.shown);
        println!("  Proposal prefilled from the conclusion — run /speclink-propose to complete the artifacts");
    }
    Ok(())
}

fn render_discuss_bind(slug: &str, change: &str, bind: DiscussBind, json: bool) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({
            "change": change,
            "slug": slug,
            "status": bind.status(),
        }));
    }
    let (verb, suffix) = bind.line();
    println!("{} {verb} discussion '{slug}' → change '{change}'{suffix}", color::green("✓"));
    Ok(())
}

#[cfg(test)]
mod patch_hash_chain_tests {
    //! Ticket→TicketBinding 的 patchHash 鏈組裝（spec「frozen snapshot 綁定
    //! discovery 與 validation patch」的回走鏈輸入）：新→舊、legacy 空洞跳過、
    //! 末輪 legacy＝空鏈（驗證輪據此 fail closed，不拿舊輪冒充末輪）。
    use super::patch_hash_chain;

    #[test]
    fn newest_first_and_legacy_gaps_skipped() {
        let rounds = [Some("sha256:r1"), None, Some("sha256:r3")];
        assert_eq!(
            patch_hash_chain(rounds.iter().map(|h| *h)),
            vec!["sha256:r3".to_string(), "sha256:r1".to_string()],
            "鏈序新→舊，中段 legacy 輪跳過"
        );
    }

    #[test]
    fn legacy_last_round_yields_an_empty_chain_even_with_older_hashes() {
        let rounds = [Some("sha256:r1"), Some("sha256:r2"), None];
        assert_eq!(
            patch_hash_chain(rounds.iter().map(|h| *h)),
            Vec::<String>::new(),
            "末輪無 hash 時不得拿舊輪快照冒充末輪"
        );
        assert_eq!(patch_hash_chain([None].iter().map(|h| *h)), Vec::<String>::new());
        assert_eq!(
            patch_hash_chain(std::iter::empty::<Option<&str>>()),
            Vec::<String>::new(),
            "空工單＝空鏈"
        );
    }
}

#[cfg(test)]
mod init_tools_tests {
    //! `init` 的工具解析入口（spec「init 內建 Agent 工具選擇」、design「CLI 互動解析
    //! 停留在 speclink-cli」）。互動 prompt 需要真實終端才會在整合測試裡觸發，因此
    //! 單選／雙選／全否重試在此以注入的行讀寫 helper 覆蓋。
    use super::*;
    use speclink_core::skills::Tool;
    use std::io::{BufRead, Cursor};

    /// 任何讀取都 panic 的 stdin 替身——釘死「不把 redirected／piped stdin 當作答案」。
    struct NeverRead;

    impl std::io::Read for NeverRead {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            panic!("stdin must not be read")
        }
    }

    impl BufRead for NeverRead {
        fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
            panic!("stdin must not be read")
        }
        fn consume(&mut self, _: usize) {
            panic!("stdin must not be read")
        }
    }

    fn with_answers(
        spec: Option<&str>,
        interactive: bool,
        answers: &str,
    ) -> (Result<Vec<Tool>>, String) {
        let mut input = Cursor::new(answers.as_bytes().to_vec());
        let mut out: Vec<u8> = Vec::new();
        let got = resolve_init_tools(spec, interactive, &mut input, &mut out);
        (got, String::from_utf8(out).expect("prompts are utf-8"))
    }

    fn without_stdin(spec: Option<&str>, interactive: bool) -> (Result<Vec<Tool>>, String) {
        let mut out: Vec<u8> = Vec::new();
        let got = resolve_init_tools(spec, interactive, &mut NeverRead, &mut out);
        (got, String::from_utf8(out).expect("prompts are utf-8"))
    }

    fn single_line_error(got: Result<Vec<Tool>>) -> String {
        let message = got.expect_err("must fail").to_string();
        assert_eq!(message.lines().count(), 1, "must be a single line: {message}");
        message
    }

    #[test]
    fn explicit_tools_are_used_without_prompting() {
        let (got, prompts) = without_stdin(Some("claude,codex"), true);
        assert_eq!(got.expect("valid selection"), vec![Tool::Claude, Tool::Codex]);
        assert!(prompts.is_empty(), "顯式 --tools 不得詢問: {prompts}");
    }

    #[test]
    fn explicit_duplicates_collapse_to_one_entry() {
        let (got, _) = without_stdin(Some("codex, codex"), false);
        assert_eq!(got.expect("valid selection"), vec![Tool::Codex]);
    }

    #[test]
    fn explicit_empty_selection_is_rejected_naming_the_flag_and_values() {
        let (got, _) = without_stdin(Some("  ,  "), true);
        let message = single_line_error(got);
        for token in ["--tools", "claude", "codex"] {
            assert!(message.contains(token), "must mention {token}: {message}");
        }
    }

    #[test]
    fn explicit_unknown_tool_names_the_offender() {
        let (got, _) = without_stdin(Some("claude,vscode"), true);
        assert!(single_line_error(got).contains("vscode"));
    }

    #[test]
    fn non_interactive_without_tools_fails_without_reading_stdin() {
        let (got, prompts) = without_stdin(None, false);
        let message = single_line_error(got);
        for token in ["--tools", "claude", "codex"] {
            assert!(message.contains(token), "must mention {token}: {message}");
        }
        assert!(prompts.is_empty(), "非互動終端不得詢問: {prompts}");
    }

    #[test]
    fn interactive_yes_yes_selects_both() {
        let (got, prompts) = with_answers(None, true, "y\ny\n");
        assert_eq!(got.expect("selection"), vec![Tool::Claude, Tool::Codex]);
        assert!(prompts.contains("Claude") && prompts.contains("Codex"), "{prompts}");
    }

    #[test]
    fn interactive_yes_no_selects_claude_only() {
        let (got, _) = with_answers(None, true, "y\nn\n");
        assert_eq!(got.expect("selection"), vec![Tool::Claude]);
    }

    #[test]
    fn interactive_no_yes_selects_codex_only() {
        let (got, _) = with_answers(None, true, "n\ny\n");
        assert_eq!(got.expect("selection"), vec![Tool::Codex]);
    }

    #[test]
    fn interactive_all_no_reasks_until_a_tool_is_picked() {
        let (got, prompts) = with_answers(None, true, "n\nn\nn\ny\n");
        assert_eq!(got.expect("selection"), vec![Tool::Codex]);
        assert!(
            prompts.matches("Claude").count() >= 2,
            "兩者皆否必須重新詢問: {prompts}"
        );
    }

    #[test]
    fn interactive_invalid_answer_reasks_the_same_question() {
        let (got, prompts) = with_answers(None, true, "maybe\ny\nn\n");
        assert_eq!(got.expect("selection"), vec![Tool::Claude]);
        assert!(
            prompts.matches("Claude").count() >= 2,
            "無效輸入必須重問同一題: {prompts}"
        );
    }

    #[test]
    fn interactive_eof_is_a_single_line_error() {
        let (got, _) = with_answers(None, true, "");
        assert!(single_line_error(got).contains("--tools"));
    }

    #[test]
    fn prompts_carry_no_ansi_escape() {
        let (_, prompts) = with_answers(None, true, "y\nn\n");
        assert!(!prompts.contains('\x1b'), "prompt 不得含 ANSI: {prompts:?}");
    }
}
