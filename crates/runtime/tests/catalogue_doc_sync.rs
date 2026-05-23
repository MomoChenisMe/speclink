//! CI snapshot test：守 `crates/runtime/src/catalogue/` 與
//! `doc/protocol/operations.md` 的 Index 表同步。
//!
//! 每次新增 / 修改 op 必須同步兩處：catalogue 是 single source of truth，
//! Index 表是 human-readable mirror。drift 由本 test 在 CI 即時擋下。

use speclink_runtime::catalogue::Catalogue;

const OPERATIONS_MD: &str = include_str!("../../../doc/protocol/operations.md");

#[derive(Debug, PartialEq, Eq)]
struct IndexRow {
    id: String,
    category: String,
    cli: String,
    tool_binding: String,
    mvp: bool,
    destructive: bool,
}

/// 從 `doc/protocol/operations.md` 抽出 `## Index` 後第一張 markdown table 的資料列。
fn parse_index_table(md: &str) -> Vec<IndexRow> {
    let after_index = md
        .split_once("\n## Index\n")
        .map(|(_, after)| after)
        .expect("operations.md missing `## Index` heading");

    let mut rows = Vec::new();
    let mut seen_separator = false;

    for line in after_index.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("## ") {
            break;
        }
        if !trimmed.starts_with('|') {
            continue;
        }

        if !seen_separator {
            if trimmed.starts_with("|---") || trimmed.starts_with("| ---") {
                seen_separator = true;
            }
            continue;
        }

        let cells: Vec<&str> = trimmed
            .trim_start_matches('|')
            .trim_end_matches('|')
            .split('|')
            .map(str::trim)
            .collect();

        if cells.len() < 9 {
            continue;
        }

        let id = strip_backticks(cells[1]);
        let category = cells[2].to_string();
        let cli = strip_backticks(cells[3]);
        let tool_binding = strip_backticks(cells[4]);
        let mvp = parse_mvp(cells[5]);
        let destructive = parse_destructive(cells[8]);

        rows.push(IndexRow {
            id,
            category,
            cli,
            tool_binding,
            mvp,
            destructive,
        });
    }

    rows
}

fn strip_backticks(s: &str) -> String {
    s.replace('`', "")
}

/// `✓` 或 `✓ (...)` 後綴註記（如 `✓ (slice-A)`）→ true；`[deferred]` → false；其他值 panic。
fn parse_mvp(s: &str) -> bool {
    if s.starts_with('✓') {
        true
    } else if s == "[deferred]" {
        false
    } else {
        panic!("operations.md Index MVP cell has unexpected value `{s}`")
    }
}

/// `⚠` → true；`—` (U+2014 EM DASH) → false；其他值 panic。
fn parse_destructive(s: &str) -> bool {
    match s {
        "⚠" => true,
        "—" => false,
        other => panic!("operations.md Index Destructive cell has unexpected value `{other}`"),
    }
}

#[test]
fn index_table_row_count_matches_catalogue_len() {
    let rows = parse_index_table(OPERATIONS_MD);
    let catalogue_len = Catalogue::all().len();
    assert_eq!(
        rows.len(),
        catalogue_len,
        "operations.md Index has {} rows, catalogue has {} entries — they must match",
        rows.len(),
        catalogue_len
    );
    assert_eq!(rows.len(), 37, "expected exactly 37 ops in Index");
}

#[test]
fn index_table_ids_match_catalogue_ids() {
    let rows = parse_index_table(OPERATIONS_MD);
    let doc_ids: std::collections::HashSet<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    let catalogue_ids: std::collections::HashSet<&str> =
        Catalogue::all().iter().map(|op| op.id).collect();

    let missing_in_catalogue: Vec<_> = doc_ids.difference(&catalogue_ids).collect();
    let extra_in_catalogue: Vec<_> = catalogue_ids.difference(&doc_ids).collect();

    assert!(
        missing_in_catalogue.is_empty() && extra_in_catalogue.is_empty(),
        "catalogue ↔ operations.md drift: missing_in_catalogue={missing_in_catalogue:?}, extra_in_catalogue={extra_in_catalogue:?}",
    );
}

#[test]
fn index_table_per_row_fields_match_catalogue() {
    let rows = parse_index_table(OPERATIONS_MD);

    for row in &rows {
        let op =
            Catalogue::get(&row.id).unwrap_or_else(|| panic!("catalogue missing id `{}`", row.id));

        assert_eq!(
            op.category, row.category,
            "category mismatch for `{}`",
            op.id
        );
        assert_eq!(op.cli, row.cli, "cli mismatch for `{}`", op.id);
        assert_eq!(
            op.tool_binding, row.tool_binding,
            "tool_binding mismatch for `{}`",
            op.id
        );
        assert_eq!(op.mvp, row.mvp, "mvp mismatch for `{}`", op.id);
        assert_eq!(
            op.destructive, row.destructive,
            "destructive mismatch for `{}`",
            op.id
        );
    }
}
