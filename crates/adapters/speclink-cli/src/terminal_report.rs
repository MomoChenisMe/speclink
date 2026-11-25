//! The terminal activity report (spec terminal-activity-report): in a Herdr
//! terminal, a verb that works on a change or a discussion labels its own
//! terminal, so the Speclink panel knows which agent does what without hooks.

use crate::Commands;
use serde::Serialize;
use speclink_verbs::families::discuss::DiscussCommands;
use speclink_verbs::families::new::NewCommands;
use speclink_verbs::families::progress::{InProgressCommands, TaskCommands};
use speclink_verbs::families::station::ReviewCommands;
use speclink_verbs::SpecVerb;

/// The two labels the report sets. An empty value removes the label.
#[derive(Debug, PartialEq, Serialize)]
pub(crate) struct Labels {
    #[serde(rename = "speclink.change")]
    change: String,
    #[serde(rename = "speclink.discussion")]
    discussion: String,
}

/// The labels for a verb that works on a change or a discussion; `None` for
/// every other verb. The table in spec terminal-activity-report decides the
/// verbs, read from the parsed arguments: read-only verbs and verbs that leave
/// the name to the engine do not count, and an archive of several changes
/// names no single one. It started from the Herdr mod's `commandLabels`, which
/// matches command text and differs on purpose: no `verify prepare` (the verb
/// does not exist), only a successful verb counts, and `--change` may come
/// anywhere after `task done`.
pub(crate) fn labels(command: &Commands) -> Option<Labels> {
    let Commands::Spec(verb) = command else { return None };
    let on_change = |name: &str| Labels { change: name.to_string(), discussion: String::new() };
    let on_discussion = |slug: &str| Labels { change: String::new(), discussion: slug.to_string() };
    match verb {
        SpecVerb::New(a) => match &a.command {
            NewCommands::Change(c) => Some(Labels {
                change: c.name.clone(),
                // The agent that writes a proposal from a discussion still belongs to it.
                discussion: c.from_discussion.clone().unwrap_or_default(),
            }),
            NewCommands::Artifact(_) => None,
        },
        SpecVerb::Instructions(a) if a.artifact.as_deref() == Some("apply") => {
            a.change.as_deref().map(on_change)
        }
        SpecVerb::Task(a) => match &a.command {
            TaskCommands::Done { change: Some(name), .. } => Some(on_change(name)),
            _ => None,
        },
        SpecVerb::InProgress(a) => match &a.command {
            InProgressCommands::Add { name } => Some(on_change(name)),
            InProgressCommands::Remove { .. } => None,
        },
        SpecVerb::Review(a) => match &a.command {
            ReviewCommands::Prepare { change, .. } => Some(on_change(change)),
            _ => None,
        },
        SpecVerb::Archive(a) => match a.changes.as_slice() {
            [name] if !a.all => Some(on_change(name)),
            _ => None,
        },
        SpecVerb::Discuss(a) => match &a.command {
            DiscussCommands::New { slug: Some(slug), .. }
            | DiscussCommands::Context { slug, .. }
            | DiscussCommands::AddRound { slug, .. }
            | DiscussCommands::Conclude { slug, .. } => Some(on_discussion(slug)),
            _ => None,
        },
        _ => None,
    }
}

/// After a verb succeeded: label this terminal when the verb counts and the
/// process runs in a Herdr terminal. `get` reads an environment variable; it
/// is not called for a verb that does not count.
#[cfg(unix)]
pub(crate) fn report(labels: Option<Labels>, get: impl Fn(&str) -> Option<String>) {
    let Some(labels) = labels else { return };
    let Some(target) = target(get) else { return };
    send(&target, &labels, DEADLINE);
}

/// The whole report — connect, send, wait for the reply — ends within this.
#[cfg(unix)]
const DEADLINE: std::time::Duration = std::time::Duration::from_millis(100);

/// Send one `terminal.label` request to the target's core and wait for its
/// reply until `deadline`. Every failure is silent. Only the socket is
/// touched: no `hc`, no other program, so a missing core stays missing.
///
/// A worker thread does the exchange because a Unix socket has no connect
/// timeout (on Linux a full backlog blocks the connect). `recv_timeout` is the
/// only limit: after the deadline the caller stops waiting, and the thread
/// ends with the process.
#[cfg(unix)]
fn send(target: &Target, labels: &Labels, deadline: std::time::Duration) {
    // One JSON-RPC message per line; herdr-core needs no `hello` for it.
    #[derive(Serialize)]
    struct Request<'a> {
        jsonrpc: &'static str,
        id: u32,
        method: &'static str,
        params: Params<'a>,
    }
    #[derive(Serialize)]
    struct Params<'a> {
        terminal: &'a str,
        labels: &'a Labels,
    }
    let params = Params { terminal: &target.terminal, labels };
    let request = Request { jsonrpc: "2.0", id: 1, method: "terminal.label", params };
    let Ok(mut line) = serde_json::to_string(&request) else { return };
    line.push('\n');
    let socket = target.socket.clone();
    let (done, finished) = std::sync::mpsc::channel();
    let worker = std::thread::Builder::new().spawn(move || {
        let _ = exchange(&socket, &line);
        let _ = done.send(());
    });
    if worker.is_ok() {
        let _ = finished.recv_timeout(deadline);
    }
}

/// Write `line` and read the reply line, which is not looked at: it only
/// tells that the core handled the request before the connection closes.
#[cfg(unix)]
fn exchange(socket: &std::path::Path, line: &str) -> std::io::Result<()> {
    use std::io::{BufRead, Read, Write};
    let mut stream = std::os::unix::net::UnixStream::connect(socket)?;
    stream.write_all(line.as_bytes())?;
    let mut reply = String::new();
    std::io::BufReader::new((&stream).take(64 * 1024)).read_line(&mut reply)?;
    Ok(())
}

/// The standard library has no Unix domain socket on other platforms: no report.
#[cfg(not(unix))]
pub(crate) fn report(_labels: Option<Labels>, _get: impl Fn(&str) -> Option<String>) {}

/// Where the report goes: this terminal and its core's socket.
#[cfg(unix)]
#[derive(Debug, PartialEq)]
struct Target {
    terminal: String,
    socket: std::path::PathBuf,
}

/// The target, when both Herdr variables have a value and
/// `SPECLINK_TERMINAL_REPORT` does not turn the report off.
/// The socket is never guessed: without `HC_TERMINAL_SOCKET` there is no
/// target, because asking hc for its default socket can start a new core.
#[cfg(unix)]
fn target(get: impl Fn(&str) -> Option<String>) -> Option<Target> {
    let set = |key| get(key).filter(|value: &String| !value.is_empty());
    let terminal = set("HC_TERMINAL_ID")?;
    let socket = set("HC_TERMINAL_SOCKET")?;
    // Read like `parse_env_bool` in speclink-core: only `true` / `false` mean
    // anything, so only `false` turns the report off.
    let off = get("SPECLINK_TERMINAL_REPORT").is_some_and(|value| value.trim().eq_ignore_ascii_case("false"));
    (!off).then(|| Target { terminal, socket: socket.into() })
}

#[cfg(all(test, unix))]
mod target_tests {
    use super::*;
    use std::collections::HashMap;

    fn target_of(vars: &[(&str, &str)]) -> Option<Target> {
        let vars: HashMap<String, String> = vars.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        target(|key| vars.get(key).cloned())
    }

    fn t2p() -> Option<Target> {
        Some(Target { terminal: "t2p".into(), socket: "/tmp/x.sock".into() })
    }

    const HERDR: [(&str, &str); 2] = [("HC_TERMINAL_ID", "t2p"), ("HC_TERMINAL_SOCKET", "/tmp/x.sock")];

    #[test]
    fn a_herdr_terminal_is_a_target() {
        assert_eq!(target_of(&HERDR), t2p());
    }

    #[test]
    fn one_missing_or_empty_herdr_variable_means_no_target() {
        let cases: &[&[(&str, &str)]] = &[
            &[],
            &[("HC_TERMINAL_ID", "t2p")],
            &[("HC_TERMINAL_SOCKET", "/tmp/x.sock")],
            &[("HC_TERMINAL_ID", ""), ("HC_TERMINAL_SOCKET", "/tmp/x.sock")],
            &[("HC_TERMINAL_ID", "t2p"), ("HC_TERMINAL_SOCKET", "")],
        ];
        for vars in cases {
            assert_eq!(target_of(vars), None, "{vars:?}");
        }
    }

    // The spec's table: only `false` turns the report off, like the other
    // boolean variables of Speclink; any other value counts as not set.
    #[test]
    fn speclink_terminal_report_false_turns_the_report_off() {
        let cases: &[(Option<&str>, bool)] = &[
            (None, true),
            (Some("false"), false),
            (Some("FALSE"), false),
            (Some(" false "), false),
            (Some("true"), true),
            (Some("0"), true),
            (Some("no"), true),
            (Some(""), true),
        ];
        for (value, reports) in cases {
            let mut vars = HERDR.to_vec();
            vars.extend(value.map(|v| ("SPECLINK_TERMINAL_REPORT", v)));
            assert_eq!(target_of(&vars).is_some(), *reports, "{value:?}");
        }
    }

    #[test]
    fn a_verb_that_does_not_count_reads_no_variable() {
        let reads = std::cell::Cell::new(0);
        report(None, |_| {
            reads.set(reads.get() + 1);
            None
        });
        assert_eq!(reads.get(), 0);
    }
}

#[cfg(all(test, unix))]
mod send_tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    /// A throwaway directory with a short path: macOS caps a socket path at
    /// 104 bytes. Removed on drop.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> TempDir {
            static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let dir = std::env::temp_dir().join(format!("sl-tr-{}-{n}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }

        fn join(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A fake core: it takes one connection within a second, reads one line,
    /// answers `reply` (or never answers, holding the connection for a second)
    /// and gives back the line, or `None` when nobody connected. Socket errors
    /// after the accept are ignored: on a busy machine the sender may already
    /// have given up, and the line it wrote is still there to read.
    fn fake_core(socket: &Path, reply: Option<&'static str>) -> std::thread::JoinHandle<Option<String>> {
        let listener = UnixListener::bind(socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        std::thread::spawn(move || {
            let start = Instant::now();
            let stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(_) if start.elapsed() < Duration::from_secs(1) => std::thread::sleep(Duration::from_millis(5)),
                    Err(_) => return None,
                }
            };
            let _ = stream.set_nonblocking(false);
            let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
            let mut line = String::new();
            let _ = BufReader::new(&stream).read_line(&mut line);
            match reply {
                Some(reply) => {
                    let _ = (&stream).write_all(reply.as_bytes());
                }
                None => std::thread::sleep(Duration::from_secs(1)),
            }
            Some(line)
        })
    }

    const OK: &str = "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\n";

    fn add_auth() -> Labels {
        Labels { change: "add-auth".into(), discussion: String::new() }
    }

    fn to(terminal: &str, socket: &Path) -> Target {
        Target { terminal: terminal.into(), socket: socket.into() }
    }

    #[test]
    fn the_core_gets_one_terminal_label_request_line() {
        let dir = TempDir::new();
        let socket = dir.join("c.sock");
        let core = fake_core(&socket, Some(OK));
        send(&to("t2p", &socket), &add_auth(), Duration::from_millis(100));
        let line = core.join().unwrap().expect("the core got a request");
        assert!(line.ends_with('\n') && !line.trim_end().contains('\n'), "{line:?}");
        let request: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "terminal.label");
        assert_eq!(
            request["params"],
            serde_json::json!({"terminal": "t2p", "labels": {"speclink.change": "add-auth", "speclink.discussion": ""}})
        );
    }

    #[test]
    fn label_values_with_chinese_and_quotes_stay_valid_json() {
        let dir = TempDir::new();
        let socket = dir.join("c.sock");
        let core = fake_core(&socket, Some(OK));
        let labels = Labels { change: String::new(), discussion: "登入\"流程\"".into() };
        send(&to("t2p", &socket), &labels, Duration::from_millis(100));
        let line = core.join().unwrap().expect("the core got a request");
        let request: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(request["params"]["labels"]["speclink.discussion"], "登入\"流程\"");
    }

    // Sharp edge: the request is one line, so a newline in a value must not
    // start a second request (say `terminal.kill`) on the same connection.
    #[test]
    fn a_newline_in_the_terminal_id_cannot_add_a_second_request() {
        let dir = TempDir::new();
        let socket = dir.join("c.sock");
        let core = fake_core(&socket, Some(OK));
        let terminal = "t2p\"}}\n{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"terminal.kill\",\"params\":{\"terminal\":\"t2p\"}}";
        send(&to(terminal, &socket), &add_auth(), Duration::from_millis(100));
        let line = core.join().unwrap().expect("the core got a request");
        let request: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(request["method"], "terminal.label");
        assert_eq!(request["params"]["terminal"], terminal);
    }

    #[test]
    fn a_core_that_never_answers_costs_no_more_than_the_deadline() {
        let dir = TempDir::new();
        let socket = dir.join("c.sock");
        let core = fake_core(&socket, None);
        let start = Instant::now();
        send(&to("t2p", &socket), &add_auth(), Duration::from_millis(50));
        assert!(start.elapsed() < Duration::from_millis(500), "{:?}", start.elapsed());
        assert!(core.join().unwrap().is_some(), "the core got a request");
    }

    #[test]
    fn an_error_or_a_reply_that_is_not_json_changes_nothing() {
        for reply in [
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"error\":{\"code\":-32000,\"message\":\"not_found\",\"data\":{\"kind\":\"not_found\"}}}\n",
            "not json\n",
        ] {
            let dir = TempDir::new();
            let socket = dir.join("c.sock");
            let core = fake_core(&socket, Some(reply));
            send(&to("t2p", &socket), &add_auth(), Duration::from_millis(100));
            assert!(core.join().unwrap().is_some(), "{reply:?}");
        }
    }

    #[test]
    fn a_missing_socket_creates_nothing() {
        let dir = TempDir::new();
        let socket = dir.join("c.sock");
        send(&to("t2p", &socket), &add_auth(), Duration::from_millis(100));
        assert!(!socket.exists());
    }

    #[test]
    fn a_plain_file_at_the_socket_path_stays_as_it_is() {
        let dir = TempDir::new();
        let socket = dir.join("c.sock");
        std::fs::write(&socket, "keep").unwrap();
        send(&to("t2p", &socket), &add_auth(), Duration::from_millis(100));
        assert_eq!(std::fs::read_to_string(&socket).unwrap(), "keep");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::FromArgMatches;

    fn labels_of(args: &[&str]) -> Option<Labels> {
        let argv = std::iter::once("speclink").chain(args.iter().copied());
        let matches = crate::cli_command().try_get_matches_from(argv).expect("the arguments parse");
        let cli = crate::Cli::from_arg_matches(&matches).expect("the arguments parse");
        labels(&cli.command)
    }

    fn change(name: &str, discussion: &str) -> Option<Labels> {
        Some(Labels { change: name.into(), discussion: discussion.into() })
    }

    fn discussion(slug: &str) -> Option<Labels> {
        Some(Labels { change: String::new(), discussion: slug.into() })
    }

    #[test]
    fn verbs_that_work_on_a_change_label_it() {
        let cases: &[&[&str]] = &[
            &["new", "change", "add-auth"],
            &["instructions", "apply", "--change", "add-auth"],
            &["instructions", "apply", "--change", "add-auth", "--json"],
            // The spec's table counts `instructions apply --change` as written.
            &["instructions", "apply", "--change", "add-auth", "--skill", "speclink-apply"],
            &["task", "done", "1", "--change", "add-auth"],
            &["task", "done", "--change", "add-auth", "2", "--json"],
            &["in-progress", "add", "add-auth"],
            &["review", "prepare", "add-auth"],
            &["archive", "add-auth"],
        ];
        for args in cases {
            assert_eq!(labels_of(args), change("add-auth", ""), "{args:?}");
        }
    }

    #[test]
    fn a_change_from_a_discussion_keeps_the_discussion_label() {
        assert_eq!(
            labels_of(&["new", "change", "add-auth", "--from-discussion", "auth-flow"]),
            change("add-auth", "auth-flow")
        );
    }

    #[test]
    fn verbs_that_work_on_a_discussion_label_it_and_clear_the_change() {
        let cases: &[&[&str]] = &[
            &["discuss", "new", "登入流程", "--slug", "auth-flow"],
            &["discuss", "context", "auth-flow", "--stdin"],
            &["discuss", "add-round", "auth-flow", "--stdin"],
            &["discuss", "conclude", "auth-flow", "--stdin"],
        ];
        for args in cases {
            assert_eq!(labels_of(args), discussion("auth-flow"), "{args:?}");
        }
    }

    #[test]
    fn names_go_out_as_written() {
        assert_eq!(labels_of(&["discuss", "add-round", "登入-流程"]), discussion("登入-流程"));
    }

    #[test]
    fn read_only_verbs_and_verbs_without_a_name_label_nothing() {
        let cases: &[&[&str]] = &[
            &["show", "add-auth"],
            &["status", "--change", "add-auth", "--json"],
            &["list", "--json"],
            &["artifact", "cat", "proposal", "--change", "add-auth"],
            &["instructions", "proposal", "--change", "add-auth", "--json"],
            &["instructions", "apply"],
            &["task", "done", "1"],
            &["task", "undone", "1", "--change", "add-auth"],
            &["discuss", "new", "登入流程"],
            &["discuss", "show", "auth-flow"],
            &["archive", "add-auth", "add-login"],
            &["archive", "--all"],
            &["archive"],
            &["plan", "--json"],
            &["review", "scope", "add-auth"],
            &["init"],
        ];
        for args in cases {
            assert_eq!(labels_of(args), None, "{args:?}");
        }
    }
}
