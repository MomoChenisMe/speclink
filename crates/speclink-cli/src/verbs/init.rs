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
use crate::remote_base::validate_or_defer;

#[derive(Args)]
pub(crate) struct InitArgs {
    /// Project path (defaults to current directory)
    path: Option<String>,
    /// AI tools to generate files for (e.g., claude, codex)
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
pub(crate) fn cmd_update(a: UpdateArgs) -> Result<()> {
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
    validate_or_defer(root, url, a.repo.as_deref())
}
