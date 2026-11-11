//! The process-level color switch, with anstream-compatible enablement semantics
//! (probed: CLICOLOR_FORCE overrides NO_COLOR; a piped stdout disables color).
//!
//! The ANSI codes themselves live in `speclink_verbs::render::Paint`. The shared
//! verbs get the switch's value as a parameter (`paint()`); the helpers below
//! color the CLI's own families through the same `Paint` (only the colors
//! those families use).

use speclink_verbs::render::Paint;
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

/// The switch as the value the shared output functions take.
pub fn paint() -> Paint {
    Paint::new(on())
}

pub fn red(s: &str) -> String {
    paint().red(s)
}
pub fn green(s: &str) -> String {
    paint().green(s)
}
