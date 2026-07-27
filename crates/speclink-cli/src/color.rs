//! ANSI styling with the frozen palette and anstream-compatible enablement semantics
//! (probed: CLICOLOR_FORCE overrides NO_COLOR; a piped stdout disables color).
//!
//! Styling goes through tiny helpers so plain mode stays byte-identical: every helper
//! returns the input unchanged when color is off, and `dim("")` in color mode emits the
//! empty `\x1b[2m\x1b[0m` pair for a change without a summary — the frozen output shape.

use std::io::IsTerminal;
use std::sync::OnceLock;

static ENABLED: OnceLock<bool> = OnceLock::new();

/// Decide color once per process. Precedence: `--no-color` flag, then CLICOLOR_FORCE
/// (any value but "0" forces ON even under NO_COLOR), then NO_COLOR / CLICOLOR=0
/// (force OFF), then whether stdout is a terminal.
pub fn init(no_color_flag: bool) {
    let on = if no_color_flag {
        false
    } else if std::env::var("CLICOLOR_FORCE").map(|v| v != "0").unwrap_or(false) {
        true
    } else if std::env::var_os("NO_COLOR").is_some()
        || std::env::var("CLICOLOR").map(|v| v == "0").unwrap_or(false)
    {
        false
    } else {
        std::io::stdout().is_terminal()
    };
    let _ = ENABLED.set(on);
}

fn on() -> bool {
    *ENABLED.get().unwrap_or(&false)
}

fn paint(code: &str, s: &str) -> String {
    if on() {
        format!("\x1b[{code}m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

pub fn bold(s: &str) -> String {
    paint("1", s)
}
pub fn dim(s: &str) -> String {
    paint("2", s)
}
pub fn red(s: &str) -> String {
    paint("31", s)
}
pub fn green(s: &str) -> String {
    paint("32", s)
}
pub fn yellow(s: &str) -> String {
    paint("33", s)
}
pub fn cyan(s: &str) -> String {
    paint("36", s)
}
pub fn bold_red(s: &str) -> String {
    paint("1;31", s)
}
pub fn bold_green(s: &str) -> String {
    paint("1;32", s)
}
pub fn bold_yellow(s: &str) -> String {
    paint("1;33", s)
}
pub fn bold_cyan(s: &str) -> String {
    paint("1;36", s)
}
