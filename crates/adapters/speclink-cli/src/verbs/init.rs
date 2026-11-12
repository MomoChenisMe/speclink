//! Project setup verbs: init and update.
//!
//! ModeFree: both write the local workspace scaffolding. `init --store remote`
//! still writes locally — it records the connection rather than consuming it.

use anyhow::{bail, Result};
use clap::Args;
use speclink_core as core;
use std::io::{BufRead, IsTerminal, Write};
use std::path::PathBuf;

use crate::color;
use crate::remote_base::{connect, validate_or_defer};
use core::init::{SyncPlan, ToolSelection, UpdateOutcome, ASSET_VERSION};
use speclink_remote::skills as remote_skills;

#[derive(Args)]
pub(crate) struct InitArgs {
    /// Project path (defaults to current directory)
    path: Option<String>,
    /// AI tools to generate files for (e.g., claude, codex, copilot, pi, cursor)
    #[arg(long)]
    tools: Option<String>,
    /// Overwrite existing files
    #[arg(long)]
    force: bool,
    /// Custom openspec directory path (default: openspec)
    #[arg(long)]
    dir: Option<String>,
    /// Store backend: fs (default) or remote
    #[arg(long, default_value = "fs")]
    store: String,
    /// Remote store connection URL (required with --store remote)
    #[arg(long)]
    url: Option<String>,
    /// This repo's registered name in the remote project
    #[arg(long)]
    repo: Option<String>,
}
#[derive(Args)]
pub(crate) struct UpdateArgs {
    /// Project path (defaults to current directory)
    path: Option<PathBuf>,
    /// Overwrite existing files
    #[arg(long)]
    force: bool,
    /// Rewrite managed skill files even when the workspace is newer than this engine
    #[arg(long)]
    allow_downgrade: bool,
}
pub(crate) fn cmd_init(a: InitArgs) -> Result<()> {
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
    let spec_dir = a.dir.clone().unwrap_or_else(|| core::workspace::DEFAULT_SPEC_DIR.to_string());
    let outcome = core::init::init(&root, &tools, a.force, &spec_dir)?;
    if let Some(report) = &outcome.imported {
        println!("{} Imported the OpenSpec project: {}", color::green("✓"), moved_summary(report));
    }
    // A legacy project's --force re-init stays in openspec/: name the folder init used.
    let folder = outcome.spec_dir_abs.strip_prefix(&root).unwrap_or(&outcome.spec_dir_abs);
    println!(
        "{} Initialized at {display_base}{}{}",
        color::green("✓"),
        std::path::MAIN_SEPARATOR,
        folder.display()
    );
    let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
    println!("Generated files for: {}", names.join(", "));
    if outcome.imported.is_some() {
        println!("! Commit the import: speclink/ (added), openspec/ (deleted), .speclink.yaml, .gitignore and the generated skill files");
        println!("! The OpenSpec CLI no longer sees this project — the move is one-way");
        println!("! OpenSpec's own skills and command files were not moved or deleted — remove them yourself if you no longer use OpenSpec");
    }
    Ok(())
}
/// The single line every missing/empty selection ends on — it names the flag and every
/// valid tool, so a failed non-interactive run is self-correcting.
const TOOLS_HINT: &str = "no AI tool selected — pass --tools with one or more of claude, \
codex, copilot, pi, cursor (comma-separated, e.g. --tools claude,cursor)";
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
/// Ask for Claude, Codex, Copilot, Pi and Cursor in turn, repeating the round until at
/// least one is picked — an empty selection is not an answer. Plain text only: nothing here is
/// styled, so `--no-color` changes nothing about the prompts.
fn prompt_for_tools(
    input: &mut impl BufRead,
    out: &mut impl Write,
) -> Result<Vec<core::skills::Tool>> {
    use core::skills::Tool;
    loop {
        let mut picked = Vec::new();
        for (tool, label) in [
            (Tool::Claude, "Claude"),
            (Tool::Codex, "Codex"),
            (Tool::Copilot, "Copilot"),
            (Tool::Pi, "Pi"),
            (Tool::Cursor, "Cursor"),
        ] {
            if ask_yes_no(input, out, label)? {
                picked.push(tool);
            }
        }
        if !picked.is_empty() {
            return Ok(picked);
        }
        writeln!(
            out,
            "Pick at least one tool: claude, codex, copilot, pi, cursor, or any combination."
        )?;
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
pub(crate) fn cmd_update(a: UpdateArgs) -> Result<()> {
    let root = a
        .path
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let root = if root.is_absolute() {
        root
    } else {
        std::env::current_dir()?.join(root)
    };
    if !core::workspace::is_project_root(&root) {
        bail!("Not initialized. Run 'speclink init' to initialize.");
    }
    let _ = a.force;
    // 降級守門在引擎的 update 本體（唯一實作落點）；--allow-downgrade 是唯一越過。
    // remote 專案的技能跟著 Server：Server 有給就不經這道守門。
    let local = || core::init::update(&root, a.allow_downgrade);
    let (outcome, source) = match skills_from_server(&root)? {
        FromServer::NotRemote => (local()?, None),
        FromServer::NotGiven(reason) => {
            eprintln!("speclink: skills generated from this CLI's templates — {reason}");
            (local()?, Some(format!("local ({ASSET_VERSION})")))
        }
        FromServer::Installed { outcome, version, replaced } => {
            if let Some(had) = replaced {
                println!("! Skills replaced with the server's version {version} (this checkout had {had})");
            }
            (outcome, Some(format!("server ({version})")))
        }
    };
    print_migration(&outcome.migration, &outcome.migrated);
    for note in &outcome.notes {
        println!("! {note}");
    }
    // 棄用提示走 stderr：不是錯誤、不影響 exit code，也不該混進 stdout 的機器面。
    for note in &outcome.deprecations {
        eprintln!("! {note}");
    }
    if outcome.updated.is_empty() && outcome.pruned.is_empty() && outcome.stripped.is_empty() {
        println!("! No AI tool configurations found. Use 'speclink init --tools' to set up.");
    } else {
        if !outcome.updated.is_empty() {
            println!("{} Updated skill files for: {}", color::green("✓"), outcome.updated.join(", "));
        }
        if !outcome.pruned.is_empty() {
            println!(
                "! Pruned generated files for deselected tool: {}",
                outcome.pruned.join(", ")
            );
        }
    }
    if let Some(source) = source {
        println!("  Skills source: {source}");
    }
    // 遺留剝除的摘要（design D2）：舊版引擎注入的區塊被移除時明說改了哪些檔案，
    // 使用者才知道 diff 裡多出的變動從何而來。
    if !outcome.stripped.is_empty() {
        println!(
            "! Stripped legacy Speclink blocks from: {}",
            outcome.stripped.join(", ")
        );
    }
    Ok(())
}
/// What asking the Server for a project's skills came to.
enum FromServer {
    /// A local project, or a remote one with no tool to ask for: no Server is asked.
    NotRemote,
    /// The Server gives no skills this run, for this reason (one line): it declares
    /// none, or the handshake did not succeed. The local templates stand in.
    NotGiven(String),
    /// The Server's skills are in the checkout. `replaced` is the newer version the
    /// checkout had, when the Server's is older.
    Installed { outcome: UpdateOutcome, version: String, replaced: Option<String> },
}

/// Take a remote project's skills from its Server (spec workspace-tools「remote 專案的
/// 技能取自 Server」). Once the handshake declares skills there is no way back to the
/// local templates: a request that fails then is an error, and nothing is written —
/// the Server said it has the skills, and other text in their place could tell the
/// agent to use verbs this Server does not have.
fn skills_from_server(root: &std::path::Path) -> Result<FromServer> {
    let app = core::config::AppConfig::load(&root.join(".speclink.yaml"))?;
    if app.remote.is_none() {
        return Ok(FromServer::NotRemote);
    }
    let selection = ToolSelection::resolve(root, &app);
    // 壞描述子＝零寫入、零請求：和本機的同步同一條規則。
    if let Some(message) = &selection.descriptor_error {
        bail!("{message}");
    }
    let targets = remote_skills::targets_of(&selection);
    if targets.is_empty() {
        return Ok(FromServer::NotRemote);
    }
    let Some(ws) = core::workspace::Workspace::discover(root)? else {
        return Ok(FromServer::NotRemote);
    };
    let conn = match speclink_host::context::resolve_store_mode(&ws) {
        Ok(core::workspace::ModeResolution { mode: core::workspace::StoreMode::Remote(conn), .. }) => conn,
        Ok(_) => return Ok(FromServer::NotRemote),
        Err(e) => return Ok(FromServer::NotGiven(e.to_string())),
    };
    let connected = match connect(&conn) {
        Ok(connected) => connected,
        Err(line) => return Ok(FromServer::NotGiven(line)),
    };
    if connected.binding.capabilities.skills.is_none() {
        return Ok(FromServer::NotGiven("this server does not provide skills".to_string()));
    }

    let refused = |e: &dyn std::fmt::Display| anyhow::anyhow!("could not get the skills from the server: {e}");
    let response = remote_skills::fetch(&connected.client, &targets, true).map_err(|e| refused(&e))?;
    let version = response.version.clone();
    let rendered = remote_skills::rendered(response).map_err(|e| refused(&e))?;
    let plan = SyncPlan::from_rendered(root, selection, rendered).map_err(|e| refused(&e))?;
    let replaced = plan.leading_version(&version);
    let outcome = plan.apply(root)?;
    Ok(FromServer::Installed { outcome, version, replaced })
}

/// The spec directory migration's lines (spec-dir-migration D5): what moved and
/// what to do next, or why it waits.
fn print_migration(plan: &core::migrate::MigrationPlan, report: &core::migrate::MigrationReport) {
    use core::migrate::MigrationPlan;
    match plan {
        MigrationPlan::Nothing => {}
        MigrationPlan::MoveFolder => {
            println!("{} Migrated the spec folder: {}", color::green("✓"), moved_summary(report));
            println!("! Commit the move: speclink/ (added) and openspec/ (deleted)");
            println!("! Teammates must upgrade speclink before pulling that commit — older versions see an empty project");
            println!("! Update your own scripts, CI settings and .gitignore entries that still name openspec/");
        }
        MigrationPlan::RenameLeftovers => {
            println!(
                "{} Renamed {} change data file(s) in speclink/ from .openspec.yaml to .meta.yaml — commit them",
                color::green("✓"),
                report.renamed
            );
        }
        MigrationPlan::Deferred(reason) => {
            println!("! Spec folder migration skipped: {reason}");
        }
    }
}

/// The folder move's result, shared by `update`'s migration and `init`'s import.
fn moved_summary(report: &core::migrate::MigrationReport) -> String {
    format!(
        "moved openspec/ to speclink/ and renamed {} change data file(s) from .openspec.yaml to .meta.yaml",
        report.renamed
    )
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

    /// 缺選集的單行訊息：`pi` 是 `copilot` 的子字串，逐 token 比對測不到漏列，故比全文。
    const FIVE_TOOLS_HINT: &str = "no AI tool selected — pass --tools with one or more of claude, codex, copilot, pi, cursor (comma-separated, e.g. --tools claude,cursor)";

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
        assert_eq!(single_line_error(got), FIVE_TOOLS_HINT);
    }

    #[test]
    fn explicit_unknown_tool_names_the_offender() {
        let (got, _) = without_stdin(Some("claude,vscode"), true);
        assert!(single_line_error(got).contains("vscode"));
    }

    #[test]
    fn explicit_tools_accept_copilot_in_any_case_and_combination() {
        // Example「--tools 值的解析」的接受列。
        let cases: [(&str, &[Tool]); 5] = [
            ("copilot", &[Tool::Copilot]),
            ("Copilot", &[Tool::Copilot]),
            ("claude,copilot", &[Tool::Claude, Tool::Copilot]),
            ("copilot,copilot", &[Tool::Copilot]),
            (
                "claude,codex,copilot",
                &[Tool::Claude, Tool::Codex, Tool::Copilot],
            ),
        ];
        for (spec, want) in cases {
            let (got, prompts) = without_stdin(Some(spec), true);
            assert_eq!(got.expect(spec), want, "--tools {spec}");
            assert!(prompts.is_empty(), "顯式 --tools 不得詢問: {prompts}");
        }
    }

    #[test]
    fn explicit_tools_accept_pi_and_cursor_in_any_case_and_combination() {
        // Example「--tools 值的解析」的 pi、cursor 接受列，加上五個工具全選。
        let cases: [(&str, &[Tool]); 7] = [
            ("pi", &[Tool::Pi]),
            ("Pi", &[Tool::Pi]),
            ("Cursor", &[Tool::Cursor]),
            ("CURSOR", &[Tool::Cursor]),
            ("cursor,cursor", &[Tool::Cursor]),
            ("codex,pi", &[Tool::Codex, Tool::Pi]),
            (
                "claude,codex,copilot,pi,cursor",
                &[Tool::Claude, Tool::Codex, Tool::Copilot, Tool::Pi, Tool::Cursor],
            ),
        ];
        for (spec, want) in cases {
            let (got, prompts) = without_stdin(Some(spec), true);
            assert_eq!(got.expect(spec), want, "--tools {spec}");
            assert!(prompts.is_empty(), "顯式 --tools 不得詢問: {prompts}");
        }
    }

    #[test]
    fn explicit_unknown_tool_lists_the_supported_set() {
        // Example「--tools 值的解析」的拒絕列：gh skill 的 agent id、Cursor CLI 的程式名
        // 都不是別名。輸入本身就含工具名：要比對完整的支援清單，單看 token 測不到漏列。
        for unknown in ["github-copilot", "cursor-agent", "pi-coding-agent"] {
            let (got, _) = without_stdin(Some(unknown), true);
            assert_eq!(
                single_line_error(got),
                format!("unknown tool: {unknown} (supported: claude, codex, copilot, pi, cursor)")
            );
        }
    }

    #[test]
    fn non_interactive_without_tools_fails_without_reading_stdin() {
        let (got, prompts) = without_stdin(None, false);
        assert_eq!(single_line_error(got), FIVE_TOOLS_HINT);
        assert!(prompts.is_empty(), "非互動終端不得詢問: {prompts}");
    }

    #[test]
    fn interactive_yes_yes_selects_both() {
        // Scenario「互動終端選擇 Claude 與 Codex」：五題依 Claude、Codex、Copilot、Pi、
        // Cursor 順序問。
        let (got, prompts) = with_answers(None, true, "y\ny\nn\nn\nn\n");
        assert_eq!(got.expect("selection"), vec![Tool::Claude, Tool::Codex]);
        let questions: Vec<&str> = prompts.split("Generate Speclink files for ").skip(1).collect();
        assert_eq!(
            questions,
            ["Claude", "Codex", "Copilot", "Pi", "Cursor"].map(|label| format!("{label}? (y/n): ")),
            "{prompts}"
        );
    }

    #[test]
    fn interactive_single_selections_follow_the_question_order() {
        let cases: [(&str, Tool); 5] = [
            ("y\nn\nn\nn\nn\n", Tool::Claude),
            ("n\ny\nn\nn\nn\n", Tool::Codex),
            ("n\nn\ny\nn\nn\n", Tool::Copilot),
            ("n\nn\nn\ny\nn\n", Tool::Pi),
            ("n\nn\nn\nn\ny\n", Tool::Cursor),
        ];
        for (answers, want) in cases {
            let (got, _) = with_answers(None, true, answers);
            assert_eq!(got.expect("selection"), vec![want], "{answers:?}");
        }
    }

    #[test]
    fn interactive_all_no_reasks_until_a_tool_is_picked() {
        // Scenario「互動終端不得提交空選集」：五題皆否就印一行提示、從 Claude 重問。
        let (got, prompts) = with_answers(None, true, "n\nn\nn\nn\nn\nn\ny\nn\nn\nn\n");
        assert_eq!(got.expect("selection"), vec![Tool::Codex]);
        assert_eq!(
            prompts.matches("for Claude?").count(),
            2,
            "五題皆否必須從 Claude 重新詢問: {prompts}"
        );
        assert!(
            prompts.contains(
                "(y/n): Pick at least one tool: claude, codex, copilot, pi, cursor, or any combination.\n"
            ),
            "{prompts}"
        );
    }

    #[test]
    fn interactive_invalid_answer_reasks_the_same_question() {
        let (got, prompts) = with_answers(None, true, "maybe\ny\nn\nn\nn\nn\n");
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
        let (_, prompts) = with_answers(None, true, "y\nn\nn\nn\nn\n");
        assert!(!prompts.contains('\x1b'), "prompt 不得含 ANSI: {prompts:?}");
    }
}
/// `tools` arrives already resolved and validated by `cmd_init` — filesystem and remote
/// init share that one entry, so a remote checkout is bootstrapped from the same selection
/// rules (and the same non-empty guarantee) as a local one.
fn cmd_init_remote(
    a: &InitArgs,
    root: &std::path::Path,
    display_base: &str,
    tools: &[core::skills::Tool],
) -> Result<()> {
    let Some(url) = a.url.as_deref() else {
        bail!("--store remote requires --url <project-scoped url>");
    };
    if a.dir.is_some() {
        bail!("--dir has no meaning with --store remote (documents live on the server)");
    }
    core::init::init_remote(root, tools, a.force, url, a.repo.as_deref())?;
    println!("{} Initialized at {display_base} (remote store)", color::green("✓"));
    let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
    println!("Generated files for: {}", names.join(", "));
    // Validate the declared repo now when credentials exist; defer otherwise
    // (offline init must not block — the first verb still validates).
    validate_or_defer(root, url, a.repo.as_deref())?;
    // 這裡產生的技能來自本機的範本（還沒登入的人也要有技能可用）。
    println!("! After you log in (`speclink auth login`), run `speclink update` to get the skills this server provides");
    Ok(())
}
