//! Output of the document-read verbs: `artifact cat` and `language show`.

use super::{RenderOptions, Rendered, Text};

/// Both verbs print the document as it is stored: no newline added, no color.
pub fn document_human(content: &str) -> Text {
    let mut t = Text::default();
    out!(t, "{content}");
    t
}

/// Neither verb has a JSON form.
pub fn document(content: &str, options: RenderOptions) -> Rendered {
    Rendered::of(options, None, |_| document_human(content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_human_is_the_content_as_stored() {
        for content in ["## Why\n\nSign-in needs a second rule.\n", "no final newline", ""] {
            let text = document_human(content);
            assert_eq!(text.stdout, content);
            assert_eq!(text.stderr, "");
        }
    }

    #[test]
    fn document_has_no_json_form_even_when_asked() {
        let out = document("# Language\n", RenderOptions { json: true, color: true });
        assert_eq!(out.stdout, "# Language\n");
        assert_eq!(out.json, None);
    }
}
