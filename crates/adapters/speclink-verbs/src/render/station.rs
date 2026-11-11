//! Output of the two quality stations: review and verify. The stations share
//! every output; the station's own words come from its constants.

use serde_json::{json, Value};
use speclink_core::station::{Round, Station, Ticket};

use super::{JsonForm, Paint, RenderOptions, Rendered, Text};

/// The three verbs that end with one success line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StationAction {
    /// A round was recorded; the number is its index.
    AddRound(usize),
    Stamp,
    Discard,
}

/// The structured value of an action: `add-round` reports the round, `stamp`
/// the change. `discard` has none.
pub fn action_json(action: StationAction, change: &str) -> Option<Value> {
    match action {
        StationAction::AddRound(round) => Some(json!({ "change": change, "round": round })),
        StationAction::Stamp => Some(json!({ "change": change })),
        StationAction::Discard => None,
    }
}

/// `add-round`, `stamp` and `discard`: one success line in the station's
/// words.
pub fn action_human(st: &Station, action: StationAction, change: &str, paint: &Paint) -> Text {
    let mut t = Text::default();
    // The action phrase differs; the frame around it is the same.
    let did = match action {
        StationAction::AddRound(round) => format!("Recorded {} Round {round}", st.noun_phrase),
        StationAction::Stamp => format!("Stamped {}", st.noun_phrase),
        StationAction::Discard => format!("Discarded {} ticket", st.noun_phrase),
    };
    outln!(t, "{} {did} for change '{change}'", paint.green("✓"));
    t
}

pub fn action(st: &Station, action: StationAction, change: &str, options: RenderOptions) -> Rendered {
    let json = action_json(action, change).map(|value| JsonForm::pretty(&value));
    Rendered::of(options, json, |paint| action_human(st, action, change, paint))
}

/// `show --json`: the ticket's rounds. The field set and the null meanings
/// are a public contract; the ticket's text is not part of it.
pub fn ticket_json(change: &str, ticket: &Ticket) -> Value {
    json!({
        "change": change,
        "rounds": ticket.rounds.iter().map(Round::to_json).collect::<Vec<_>>(),
        "lastRound": ticket.rounds.last().map(Round::to_json),
    })
}

/// `show`: the ticket's text when there is one. Without it (an old server
/// that sends no text) a structured summary is printed instead.
pub fn show_human(st: &Station, change: &str, ticket: &Ticket, doc: Option<&str>) -> Text {
    if let Some(doc) = doc {
        return ticket_text_human(doc);
    }
    let mut t = Text::default();
    outln!(t, "{} — {change}", st.title);
    for r in &ticket.rounds {
        outln!(t, "\nRound {}", r.index);
        if let Some(phase) = &r.phase {
            outln!(t, "  Phase: {}", phase.as_str());
        }
        if let Some(hash) = &r.patch_hash {
            outln!(t, "  Patch: {hash}");
        }
        if !r.scope.is_empty() {
            outln!(t, "  Scope: {}", r.scope.join(", "));
        }
        for f in &r.findings {
            outln!(t, "  - [{}] {} — {}", f.severity.as_str(), f.path, f.text);
        }
    }
    t
}

pub fn show(
    st: &Station,
    change: &str,
    ticket: &Ticket,
    doc: Option<&str>,
    options: RenderOptions,
) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&ticket_json(change, ticket))), |_| {
        show_human(st, change, ticket, doc)
    })
}

fn ticket_text_human(doc: &str) -> Text {
    let mut t = Text::default();
    out!(t, "{doc}");
    t
}

/// The human form of `show` for a caller that holds the ticket's text and
/// does not parse the ticket: the text as it is stored.
pub fn ticket_text(doc: &str, options: RenderOptions) -> Rendered {
    Rendered::of(options, None, |_| ticket_text_human(doc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use speclink_core::station::{Finding, Round, RoundPhase, Severity, REVIEW, VERIFY};

    const PLAIN: Paint = Paint::new(false);
    const COLOR: Paint = Paint::new(true);
    const JSON: RenderOptions = RenderOptions { json: true, color: false };

    #[test]
    fn action_human_uses_the_station_words() {
        let rows = [
            (&REVIEW, StationAction::AddRound(1), "✓ Recorded review Round 1 for change 'ready'\n"),
            (&REVIEW, StationAction::Stamp, "✓ Stamped review for change 'ready'\n"),
            (&REVIEW, StationAction::Discard, "✓ Discarded review ticket for change 'ready'\n"),
            (&VERIFY, StationAction::AddRound(3), "✓ Recorded verification Round 3 for change 'ready'\n"),
            (&VERIFY, StationAction::Stamp, "✓ Stamped verification for change 'ready'\n"),
            (&VERIFY, StationAction::Discard, "✓ Discarded verification ticket for change 'ready'\n"),
        ];
        for (st, action, want) in rows {
            assert_eq!(action_human(st, action, "ready", &PLAIN).stdout, want);
        }
        assert_eq!(
            action_human(&REVIEW, StationAction::Stamp, "ready", &COLOR).stdout,
            "\x1b[32m✓\x1b[0m Stamped review for change 'ready'\n"
        );
    }

    #[test]
    fn action_json_reports_the_round_or_the_change() {
        assert_eq!(action_json(StationAction::AddRound(1), "beta"), Some(json!({ "change": "beta", "round": 1 })));
        assert_eq!(action_json(StationAction::Stamp, "beta"), Some(json!({ "change": "beta" })));
        assert_eq!(action_json(StationAction::Discard, "beta"), None);
        let stamped = action(&VERIFY, StationAction::Stamp, "beta", RenderOptions::default());
        assert_eq!(stamped.stdout, "✓ Stamped verification for change 'beta'\n");
        assert_eq!(stamped.json, Some(json!({ "change": "beta" })));
        let discarded = action(&REVIEW, StationAction::Discard, "beta", JSON);
        assert_eq!(discarded.stdout, "✓ Discarded review ticket for change 'beta'\n");
        assert_eq!(discarded.json, None);
    }

    fn ticket() -> Ticket {
        let finding = |severity: Severity, text: &str| Finding {
            severity,
            path: "src/auth.rs".to_string(),
            text: text.to_string(),
        };
        Ticket {
            rounds: vec![
                Round {
                    index: 1,
                    phase: Some(RoundPhase::Discovery),
                    patch_hash: Some("sha256:abc".to_string()),
                    rules: None,
                    scope: vec!["src/auth.rs".to_string(), "src/billing.rs".to_string()],
                    findings: vec![
                        finding(Severity::Critical, "unwrap on user input"),
                        finding(Severity::Suggestion, "rename helper"),
                    ],
                },
                Round { index: 2, phase: None, patch_hash: None, rules: None, scope: Vec::new(), findings: Vec::new() },
            ],
        }
    }

    #[test]
    fn show_human_is_the_ticket_text_when_there_is_one() {
        let doc = "# Review — ready\n\n## Round 1\n\n**Scope**: src/auth.rs\n";
        assert_eq!(show_human(&REVIEW, "ready", &ticket(), Some(doc)).stdout, doc);
        let text_only = ticket_text(doc, JSON);
        assert_eq!(text_only.stdout, doc);
        assert_eq!(text_only.json, None);
    }

    #[test]
    fn show_human_without_the_text_is_a_structured_summary() {
        assert_eq!(
            show_human(&VERIFY, "ready", &ticket(), None).stdout,
            "Verify — ready\n\nRound 1\n  Phase: discovery\n  Patch: sha256:abc\n  Scope: src/auth.rs, src/billing.rs\n  - [CRITICAL] src/auth.rs — unwrap on user input\n  - [SUGGESTION] src/auth.rs — rename helper\n\nRound 2\n"
        );
    }

    #[test]
    fn ticket_json_carries_every_round_and_the_last_one() {
        let last = json!({ "index": 2, "phase": null, "patchHash": null, "rules": null, "scope": [], "findings": [] });
        let value = ticket_json("ready", &ticket());
        assert_eq!(value["change"], "ready");
        assert_eq!(value["lastRound"], last);
        assert_eq!(value["rounds"][1], last);
        assert_eq!(
            value["rounds"][0],
            json!({
                "index": 1, "phase": "discovery", "patchHash": "sha256:abc", "rules": null,
                "scope": ["src/auth.rs", "src/billing.rs"],
                "findings": [
                    { "severity": "CRITICAL", "path": "src/auth.rs", "text": "unwrap on user input" },
                    { "severity": "SUGGESTION", "path": "src/auth.rs", "text": "rename helper" },
                ],
            })
        );
        let out = show(&REVIEW, "ready", &ticket(), Some("# Review — ready\n"), JSON);
        assert!(out.stdout.starts_with("{\n  \"change\": \"ready\",\n  \"lastRound\": {\n"), "{}", out.stdout);
        assert_eq!(out.json, Some(value));
    }
}
