//! ANSI styling, decided by the caller. The codes are defined here and
//! nowhere else: the CLI's own families color through the same type.

/// Whether to color, as one value passed to every human-output function.
/// With `enabled` false every method returns its input unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Paint {
    pub enabled: bool,
}

impl Paint {
    pub const fn new(enabled: bool) -> Paint {
        Paint { enabled }
    }

    fn code(&self, code: &str, s: &str) -> String {
        if self.enabled {
            format!("\x1b[{code}m{s}\x1b[0m")
        } else {
            s.to_string()
        }
    }

    pub fn bold(&self, s: &str) -> String {
        self.code("1", s)
    }
    pub fn dim(&self, s: &str) -> String {
        self.code("2", s)
    }
    pub fn red(&self, s: &str) -> String {
        self.code("31", s)
    }
    pub fn green(&self, s: &str) -> String {
        self.code("32", s)
    }
    pub fn yellow(&self, s: &str) -> String {
        self.code("33", s)
    }
    pub fn cyan(&self, s: &str) -> String {
        self.code("36", s)
    }
    pub fn bold_red(&self, s: &str) -> String {
        self.code("1;31", s)
    }
    pub fn bold_green(&self, s: &str) -> String {
        self.code("1;32", s)
    }
    pub fn bold_yellow(&self, s: &str) -> String {
        self.code("1;33", s)
    }
    pub fn bold_cyan(&self, s: &str) -> String {
        self.code("1;36", s)
    }
}

#[cfg(test)]
mod tests {
    use super::Paint;

    type Method = fn(&Paint, &str) -> String;

    /// Each method with the SGR code the CLI's color module used for it.
    const PALETTE: [(&str, Method, &str); 10] = [
        ("bold", Paint::bold, "1"),
        ("dim", Paint::dim, "2"),
        ("red", Paint::red, "31"),
        ("green", Paint::green, "32"),
        ("yellow", Paint::yellow, "33"),
        ("cyan", Paint::cyan, "36"),
        ("bold_red", Paint::bold_red, "1;31"),
        ("bold_green", Paint::bold_green, "1;32"),
        ("bold_yellow", Paint::bold_yellow, "1;33"),
        ("bold_cyan", Paint::bold_cyan, "1;36"),
    ];

    #[test]
    fn disabled_paint_returns_the_input_unchanged() {
        let paint = Paint::new(false);
        for (name, method, _) in PALETTE {
            assert_eq!(method(&paint, "text"), "text", "{name}");
            assert_eq!(method(&paint, ""), "", "{name}");
        }
    }

    #[test]
    fn enabled_paint_wraps_the_input_in_the_frozen_codes() {
        let paint = Paint::new(true);
        for (name, method, code) in PALETTE {
            assert_eq!(method(&paint, "text"), format!("\x1b[{code}mtext\x1b[0m"), "{name}");
        }
    }

    // A change without a summary prints the empty pair in color mode — part
    // of the frozen output.
    #[test]
    fn enabled_paint_wraps_an_empty_string_too() {
        assert_eq!(Paint::new(true).dim(""), "\x1b[2m\x1b[0m");
    }
}
