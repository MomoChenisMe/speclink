//! Output of the document verbs: `artifact cat`, `language show` and
//! `language replace`.

use serde_json::{json, Value};
use speclink_core::command::LanguageReplaceOutcome;

use super::{JsonForm, Paint, RenderOptions, Rendered, Text};

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

/// `language replace --json`: whether this call created the document.
pub fn replaced_json(o: &LanguageReplaceOutcome) -> Value {
    json!({ "created": o.created })
}

/// `language replace`: one success line, worded by what the call did.
pub fn replaced_human(o: &LanguageReplaceOutcome, paint: &Paint) -> Text {
    let mut t = Text::default();
    let did = if o.created { "Created" } else { "Replaced" };
    outln!(t, "{} {did} the LANGUAGE document", paint.green("✓"));
    t
}

/// `language replace`, from the engine's result.
pub fn replaced(o: &LanguageReplaceOutcome, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&replaced_json(o))), |paint| replaced_human(o, paint))
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
    fn language_replace_has_one_line_for_a_replace_and_one_for_a_create() {
        // spec verb-contract「詞彙文件的寫入動詞」的輸出：兩種成功行與 {created}。
        use speclink_core::command::LanguageReplaceOutcome;
        const PLAIN: RenderOptions = RenderOptions { json: false, color: false };
        const COLOR: RenderOptions = RenderOptions { json: false, color: true };
        const JSON: RenderOptions = RenderOptions { json: true, color: true };
        let replaced_doc = LanguageReplaceOutcome { created: false };
        let created_doc = LanguageReplaceOutcome { created: true };

        let out = replaced(&replaced_doc, PLAIN);
        assert_eq!((out.stdout.as_str(), out.stderr.as_str()), ("✓ Replaced the LANGUAGE document\n", ""));
        assert_eq!(out.json, Some(serde_json::json!({ "created": false })));
        assert_eq!(replaced(&created_doc, PLAIN).stdout, "✓ Created the LANGUAGE document\n");
        // --no-color 之外只多色碼，文字相同。
        assert_eq!(replaced(&replaced_doc, COLOR).stdout, "\x1b[32m✓\x1b[0m Replaced the LANGUAGE document\n");
        assert_eq!(replaced(&created_doc, COLOR).stdout, "\x1b[32m✓\x1b[0m Created the LANGUAGE document\n");

        assert_eq!(replaced(&created_doc, JSON).stdout, "{\n  \"created\": true\n}\n");
        assert_eq!(replaced(&replaced_doc, JSON).stdout, "{\n  \"created\": false\n}\n");
        assert_eq!(replaced(&created_doc, JSON).json, Some(serde_json::json!({ "created": true })));
    }

    #[test]
    fn document_has_no_json_form_even_when_asked() {
        let out = document("# Language\n", RenderOptions { json: true, color: true });
        assert_eq!(out.stdout, "# Language\n");
        assert_eq!(out.json, None);
    }
}
