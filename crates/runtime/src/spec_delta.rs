//! Spec delta merge：解析 `<change>/specs/<capability>/spec.md` 的 4 種 heading
//! （`## ADDED Requirements` / `## MODIFIED Requirements` / `## REMOVED Requirements` /
//! `## RENAMED Requirements`）並套用至主 spec 的純文字演算法。
//!
//! 演算法為純 in-memory 運算：input 為 delta 與既有主 spec 的 `&str`，output 為新主 spec 的
//! `String` + [`ApplySummary`]。不觸碰 filesystem，可被任何 provider 復用。

use thiserror::Error;

/// 解析後的 delta spec 結構。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedDelta {
    /// `## ADDED Requirements` 下的 requirement 區塊。
    pub added: Vec<RequirementBlock>,
    /// `## MODIFIED Requirements` 下的 requirement 區塊。
    pub modified: Vec<RequirementBlock>,
    /// `## REMOVED Requirements` 下的 requirement 區塊。
    pub removed: Vec<RequirementBlock>,
    /// `## RENAMED Requirements` 下的 rename 項目。
    pub renamed: Vec<RenamedEntry>,
}

/// 單一 `### Requirement: <name>` 區塊。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequirementBlock {
    /// Requirement 名稱（heading 後 trim 過的完整字串，可含 backtick）。
    pub name: String,
    /// 區塊完整文字（含 `### Requirement: <name>` 起始行至下個區塊邊界前）。
    pub content: String,
}

/// `## RENAMED Requirements` 的單一項目。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenamedEntry {
    /// 原 requirement 名稱（讀自 `**FROM:**` 行，trim）。
    pub from: String,
    /// 新 requirement 名稱（讀自 `**TO:**` 行，trim）。
    pub to: String,
    /// 區塊完整文字（保留以利 debug；apply 階段不使用）。
    pub content: String,
}

/// `apply_delta` 成功後回傳的套用摘要。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplySummary {
    /// 套用的 ADDED 區塊數量。
    pub added_count: usize,
    /// 套用的 MODIFIED 區塊數量。
    pub modified_count: usize,
    /// 套用的 REMOVED 區塊數量。
    pub removed_count: usize,
    /// 套用的 RENAMED 區塊數量。
    pub renamed_count: usize,
    /// 主 spec 是否為本次新建（main 為 `None` → `true`）。
    pub created_main_spec: bool,
}

/// Spec delta 套用的錯誤型別。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpecDeltaError {
    /// 解析錯誤：heading 重複、格式錯誤、缺 FROM/TO 等。
    #[error("spec delta parse error: {message}")]
    Parse {
        /// 解析失敗描述。
        message: String,
    },

    /// 套用錯誤：ADDED 名稱已存在、MODIFIED/REMOVED/RENAMED 名稱找不到。
    #[error("spec delta conflict ({operation}): requirement '{requirement}'")]
    Conflict {
        /// 觸發衝突的 requirement 名稱。
        requirement: String,
        /// 觸發衝突的 heading 操作。
        operation: &'static str,
    },
}

const HEAD_ADDED: &str = "## ADDED Requirements";
const HEAD_MODIFIED: &str = "## MODIFIED Requirements";
const HEAD_REMOVED: &str = "## REMOVED Requirements";
const HEAD_RENAMED: &str = "## RENAMED Requirements";
const REQ_PREFIX: &str = "### Requirement: ";

/// 解析 delta spec 字串為 [`ParsedDelta`]。
///
/// 規則（對應 spec `Delta heading recognition` 與 `Requirement block delimitation`）：
///
/// - 僅接受四種固定 `## ` heading；其他 `## ` 開頭一律 Parse error
/// - 每種 heading 至多出現一次；重複出現一律 Parse error
/// - leading whitespace 視為內容；heading 必須行首即出現
/// - 區塊邊界：下一個 `### Requirement:`、下一個 `## `、或 EOF
pub fn parse_delta(content: &str) -> Result<ParsedDelta, SpecDeltaError> {
    let mut parsed = ParsedDelta::default();
    let mut seen_added = false;
    let mut seen_modified = false;
    let mut seen_removed = false;
    let mut seen_renamed = false;
    let mut current_section: Option<Section> = None;
    let mut current_blocks: Vec<RequirementBlock> = Vec::new();
    let mut current_block_lines: Vec<String> = Vec::new();
    let mut current_block_name: Option<String> = None;

    for raw_line in content.lines() {
        if let Some(stripped) = trim_trailing(raw_line).strip_prefix("## ") {
            // 結束目前 block / section
            flush_current_block(
                &mut current_blocks,
                &mut current_block_lines,
                &mut current_block_name,
            );
            if let Some(sec) = current_section.take() {
                store_section(&mut parsed, sec, std::mem::take(&mut current_blocks))?;
            }

            let heading_full = format!("## {stripped}");
            let next_section = match heading_full.as_str() {
                HEAD_ADDED => {
                    if seen_added {
                        return Err(SpecDeltaError::Parse {
                            message: format!("duplicate heading: {heading_full}"),
                        });
                    }
                    seen_added = true;
                    Section::Added
                }
                HEAD_MODIFIED => {
                    if seen_modified {
                        return Err(SpecDeltaError::Parse {
                            message: format!("duplicate heading: {heading_full}"),
                        });
                    }
                    seen_modified = true;
                    Section::Modified
                }
                HEAD_REMOVED => {
                    if seen_removed {
                        return Err(SpecDeltaError::Parse {
                            message: format!("duplicate heading: {heading_full}"),
                        });
                    }
                    seen_removed = true;
                    Section::Removed
                }
                HEAD_RENAMED => {
                    if seen_renamed {
                        return Err(SpecDeltaError::Parse {
                            message: format!("duplicate heading: {heading_full}"),
                        });
                    }
                    seen_renamed = true;
                    Section::Renamed
                }
                other => {
                    return Err(SpecDeltaError::Parse {
                        message: format!("unrecognized heading: {other}"),
                    });
                }
            };
            current_section = Some(next_section);
            continue;
        }

        // 處理 `### Requirement:` 邊界
        if let Some(rest) = raw_line.strip_prefix(REQ_PREFIX) {
            // 新 requirement 邊界 — 先存上一個
            flush_current_block(
                &mut current_blocks,
                &mut current_block_lines,
                &mut current_block_name,
            );
            let name = rest.trim().to_string();
            if name.is_empty() {
                return Err(SpecDeltaError::Parse {
                    message: "requirement heading with empty name".to_string(),
                });
            }
            current_block_name = Some(name);
            current_block_lines.push(raw_line.to_string());
            continue;
        }

        // 普通內容行
        if current_block_name.is_some() {
            current_block_lines.push(raw_line.to_string());
        } else {
            // 在 heading 之外或 section 起頭尚未進入 requirement — 視為 section preamble，
            // 本實作捨棄（spec 不規定 preamble 用途）。
        }
    }

    // EOF：flush 最後 block + section
    flush_current_block(
        &mut current_blocks,
        &mut current_block_lines,
        &mut current_block_name,
    );
    if let Some(sec) = current_section.take() {
        store_section(&mut parsed, sec, std::mem::take(&mut current_blocks))?;
    }

    Ok(parsed)
}

#[derive(Debug, Clone, Copy)]
enum Section {
    Added,
    Modified,
    Removed,
    Renamed,
}

fn store_section(
    parsed: &mut ParsedDelta,
    sec: Section,
    blocks: Vec<RequirementBlock>,
) -> Result<(), SpecDeltaError> {
    match sec {
        Section::Added => parsed.added = blocks,
        Section::Modified => parsed.modified = blocks,
        Section::Removed => parsed.removed = blocks,
        Section::Renamed => {
            let mut entries = Vec::with_capacity(blocks.len());
            for b in blocks {
                let (from, to) =
                    extract_from_to(&b.content).ok_or_else(|| SpecDeltaError::Parse {
                        message: format!(
                            "RENAMED requirement '{}' missing **FROM:** or **TO:** line",
                            b.name
                        ),
                    })?;
                entries.push(RenamedEntry {
                    from,
                    to,
                    content: b.content,
                });
            }
            parsed.renamed = entries;
        }
    }
    Ok(())
}

fn flush_current_block(
    blocks: &mut Vec<RequirementBlock>,
    lines: &mut Vec<String>,
    name: &mut Option<String>,
) {
    if let Some(n) = name.take() {
        let content = lines.join("\n");
        blocks.push(RequirementBlock { name: n, content });
        lines.clear();
    } else {
        lines.clear();
    }
}

fn extract_from_to(content: &str) -> Option<(String, String)> {
    let mut from = None;
    let mut to = None;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("**FROM:**") {
            from = Some(rest.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix("**TO:**") {
            to = Some(rest.trim().to_string());
        }
    }
    match (from, to) {
        (Some(f), Some(t)) if !f.is_empty() && !t.is_empty() => Some((f, t)),
        _ => None,
    }
}

fn trim_trailing(s: &str) -> &str {
    s.trim_end_matches([' ', '\t', '\r'])
}

/// 套用 delta 至主 spec，回傳新主 spec 字串 + [`ApplySummary`]。
///
/// 套用順序固定（spec `Apply order across heading sections`）：RENAMED → REMOVED → MODIFIED → ADDED。
///
/// 當 `main == None`：視為主 spec 新建，初始為空字串；只接受 ADDED（其他 heading 找不到對應
/// 區塊 → Conflict）。
pub fn apply_delta(
    main: Option<&str>,
    delta: &ParsedDelta,
) -> Result<(String, ApplySummary), SpecDeltaError> {
    let created_main_spec = main.is_none();
    let mut current = main.unwrap_or("").to_string();

    // 1) RENAMED
    for entry in &delta.renamed {
        current = rename_requirement(&current, &entry.from, &entry.to)?;
    }

    // 2) REMOVED
    for block in &delta.removed {
        current = remove_requirement(&current, &block.name)?;
    }

    // 3) MODIFIED
    for block in &delta.modified {
        current = modify_requirement(&current, &block.name, &block.content)?;
    }

    // 4) ADDED
    for block in &delta.added {
        current = add_requirement(&current, &block.name, &block.content)?;
    }

    let summary = ApplySummary {
        added_count: delta.added.len(),
        modified_count: delta.modified.len(),
        removed_count: delta.removed.len(),
        renamed_count: delta.renamed.len(),
        created_main_spec,
    };
    Ok((current, summary))
}

/// 在 main spec 中找 `### Requirement: <name>` heading 行的索引。
fn find_requirement_line(main: &str, name: &str) -> Option<(usize, usize)> {
    let target = format!("{REQ_PREFIX}{name}");
    for (idx, line) in main.lines().enumerate() {
        if line.trim_end_matches([' ', '\t', '\r']) == target {
            return Some((idx, line.len()));
        }
    }
    None
}

/// 找出 requirement 區塊的起訖 byte offset（end 為 exclusive；包含尾隨單一空行）。
fn requirement_span(main: &str, name: &str) -> Option<(usize, usize)> {
    let target = format!("{REQ_PREFIX}{name}");
    // 用 byte offset 找 heading 起點
    let mut start: Option<usize> = None;
    let mut offset = 0usize;
    let bytes = main.as_bytes();
    for line in main.split_inclusive('\n') {
        let line_no_nl = line.strip_suffix('\n').unwrap_or(line);
        let trimmed = line_no_nl.trim_end_matches([' ', '\t', '\r']);
        if trimmed == target {
            start = Some(offset);
            break;
        }
        offset += line.len();
    }
    let start = start?;

    // 從 heading 之後找下個邊界：下一個 `### Requirement:`、下一個 `## ` heading、或 EOF。
    let after_heading = offset + bytes_to_next_newline(&bytes[start..]) + 1;
    let mut block_end = bytes.len();
    let mut scan_offset = after_heading;
    for line in main[after_heading.min(main.len())..].split_inclusive('\n') {
        let line_no_nl = line.strip_suffix('\n').unwrap_or(line);
        if line_no_nl.starts_with(REQ_PREFIX) || line_no_nl.starts_with("## ") {
            block_end = scan_offset;
            break;
        }
        scan_offset += line.len();
    }
    // 若 block_end 之前的 char 為 '\n' 且 block_end 之前再之前也是 '\n'（雙空行），保留兩個換行；
    // 若僅單一空行（即 block_end 之前一行是 ""），刪掉這一行。
    // 統一行為：將 block 視為「heading 起始至 block_end」即可；後續呼叫端處理銜接。
    Some((start, block_end))
}

fn bytes_to_next_newline(b: &[u8]) -> usize {
    b.iter().position(|&x| x == b'\n').unwrap_or(b.len() - 1)
}

fn rename_requirement(main: &str, from: &str, to: &str) -> Result<String, SpecDeltaError> {
    let target = format!("{REQ_PREFIX}{from}");
    // line-based 重建
    let mut found = false;
    let mut out = String::with_capacity(main.len() + to.len());
    let ends_with_nl = main.ends_with('\n');
    let mut iter = main.split('\n').peekable();
    while let Some(line) = iter.next() {
        let trimmed = line.trim_end_matches([' ', '\t', '\r']);
        if !found && trimmed == target {
            out.push_str(REQ_PREFIX);
            out.push_str(to);
            // 保留原行的 trailing whitespace 已被 trim — 重建時不還原（無語意差異）
            found = true;
        } else {
            out.push_str(line);
        }
        if iter.peek().is_some() {
            out.push('\n');
        }
    }
    if ends_with_nl && !out.ends_with('\n') {
        out.push('\n');
    }
    if !found {
        return Err(SpecDeltaError::Conflict {
            requirement: from.to_string(),
            operation: "RENAMED",
        });
    }
    Ok(out)
}

fn remove_requirement(main: &str, name: &str) -> Result<String, SpecDeltaError> {
    let (start, end) = requirement_span(main, name).ok_or_else(|| SpecDeltaError::Conflict {
        requirement: name.to_string(),
        operation: "REMOVED",
    })?;

    let mut result = String::with_capacity(main.len());
    result.push_str(&main[..start]);
    // 尾隨單一空行：若 main[end..] 以 "\n" 開頭（即 end 處正好是 newline），移除這一個 newline
    // 以避免雙空行；多個連續空行保留以上的空行（spec 規定僅刪「尾隨單一空行」）。
    let mut after = &main[end..];
    // 先檢查 result 結尾是否為 `\n` 且 after 起頭為 `\n` → 表示中間多出一個空行
    if result.ends_with('\n') && after.starts_with('\n') {
        // 再檢查 after 是否為「單一空行」(即 after 為 "\n" 後緊接非 newline)
        // 若 after 起頭是 `\n\n` → 多個空行，不剝除（保留語意）
        if !after.starts_with("\n\n") {
            after = &after[1..];
        }
    }
    result.push_str(after);
    Ok(result)
}

fn modify_requirement(main: &str, name: &str, new_content: &str) -> Result<String, SpecDeltaError> {
    let (start, end) = requirement_span(main, name).ok_or_else(|| SpecDeltaError::Conflict {
        requirement: name.to_string(),
        operation: "MODIFIED",
    })?;

    let mut result = String::with_capacity(main.len() + new_content.len());
    result.push_str(&main[..start]);
    result.push_str(new_content);
    // 確保 new_content 結束後有換行可供下一區塊延續
    if !new_content.ends_with('\n') {
        result.push('\n');
    }
    result.push_str(&main[end..]);
    Ok(result)
}

fn add_requirement(main: &str, name: &str, content: &str) -> Result<String, SpecDeltaError> {
    if find_requirement_line(main, name).is_some() {
        return Err(SpecDeltaError::Conflict {
            requirement: name.to_string(),
            operation: "ADDED",
        });
    }
    let mut out = String::with_capacity(main.len() + content.len() + 2);
    if main.is_empty() {
        out.push_str(content);
        if !content.ends_with('\n') {
            out.push('\n');
        }
        return Ok(out);
    }
    out.push_str(main);
    // 確保 main 結尾為 `\n\n`（單一空行分隔），但若 main 已以 `\n\n` 結束則不加；若以 `\n` 結束則補一個
    if main.ends_with("\n\n") {
        // already has blank-line separator
    } else if main.ends_with('\n') {
        out.push('\n');
    } else {
        out.push('\n');
        out.push('\n');
    }
    out.push_str(content);
    if !content.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsed_delta_default_empty() {
        let p = ParsedDelta::default();
        assert!(p.added.is_empty());
        assert!(p.modified.is_empty());
        assert!(p.removed.is_empty());
        assert!(p.renamed.is_empty());
    }

    #[test]
    fn parse_only_added_with_two_requirements() {
        let delta =
            "## ADDED Requirements\n\n### Requirement: A\nbody A\n\n### Requirement: B\nbody B\n";
        let p = parse_delta(delta).expect("ok");
        assert_eq!(p.added.len(), 2);
        assert_eq!(p.added[0].name, "A");
        assert_eq!(p.added[1].name, "B");
        assert!(p.modified.is_empty());
        assert!(p.removed.is_empty());
        assert!(p.renamed.is_empty());
    }

    #[test]
    fn parse_unrecognized_heading_rejected() {
        let delta = "## ADDED Requirements\n\n### Requirement: A\n\n## DEPRECATED Requirements\n";
        let err = parse_delta(delta).expect_err("err");
        match err {
            SpecDeltaError::Parse { message } => {
                assert!(
                    message.contains("DEPRECATED"),
                    "message should name the heading: {message}"
                );
            }
            _ => panic!("expected Parse error"),
        }
    }

    #[test]
    fn parse_duplicate_heading_rejected() {
        let delta = "## ADDED Requirements\n\n### Requirement: A\n\n## ADDED Requirements\n\n### Requirement: B\n";
        let err = parse_delta(delta).expect_err("err");
        assert!(matches!(err, SpecDeltaError::Parse { .. }));
    }

    #[test]
    fn parse_renamed_requires_from_and_to() {
        let delta_missing_from = "## RENAMED Requirements\n\n### Requirement: New\n**TO:** New\n";
        let err = parse_delta(delta_missing_from).expect_err("missing FROM");
        assert!(matches!(err, SpecDeltaError::Parse { .. }));

        let delta_missing_to = "## RENAMED Requirements\n\n### Requirement: New\n**FROM:** Old\n";
        let err2 = parse_delta(delta_missing_to).expect_err("missing TO");
        assert!(matches!(err2, SpecDeltaError::Parse { .. }));

        let delta_ok =
            "## RENAMED Requirements\n\n### Requirement: New\n**FROM:** Old\n**TO:** New\n";
        let p = parse_delta(delta_ok).expect("ok");
        assert_eq!(p.renamed.len(), 1);
        assert_eq!(p.renamed[0].from, "Old");
        assert_eq!(p.renamed[0].to, "New");
    }

    #[test]
    fn parse_delta_requirement_with_nested_scenarios() {
        let delta = "## ADDED Requirements\n\n### Requirement: A\n\n#### Scenario: One\n\nbody1\n\n#### Scenario: Two\n\nbody2\n\n#### Scenario: Three\n\nbody3\n\n### Requirement: B\n\nbody B\n";
        let p = parse_delta(delta).expect("ok");
        assert_eq!(p.added.len(), 2);
        let a = &p.added[0];
        assert_eq!(a.name, "A");
        // 三個 scenario 應全部在 A 的 content 中
        assert!(a.content.contains("Scenario: One"));
        assert!(a.content.contains("Scenario: Two"));
        assert!(a.content.contains("Scenario: Three"));
        // B 的 content 應該獨立
        assert!(!a.content.contains("Requirement: B"));
        assert!(p.added[1].content.contains("body B"));
    }

    #[test]
    fn parse_delta_requirement_name_with_backticks() {
        let delta =
            "## ADDED Requirements\n\n### Requirement: `artifact write` command surface\n\nbody\n";
        let p = parse_delta(delta).expect("ok");
        assert_eq!(p.added.len(), 1);
        assert_eq!(p.added[0].name, "`artifact write` command surface");
    }

    #[test]
    fn apply_added_to_nonexistent_main() {
        let delta =
            parse_delta("## ADDED Requirements\n\n### Requirement: User login\n\nbody\n").unwrap();
        let (out, summary) = apply_delta(None, &delta).expect("ok");
        assert!(out.contains("### Requirement: User login"));
        assert_eq!(summary.added_count, 1);
        assert!(summary.created_main_spec);
    }

    #[test]
    fn apply_added_existing_main_does_not_set_created_flag() {
        let delta =
            parse_delta("## ADDED Requirements\n\n### Requirement: New One\n\nbody\n").unwrap();
        let main = "### Requirement: Existing\n\nold body\n";
        let (out, summary) = apply_delta(Some(main), &delta).expect("ok");
        assert!(out.contains("### Requirement: Existing"));
        assert!(out.contains("### Requirement: New One"));
        assert!(!summary.created_main_spec);
    }

    #[test]
    fn apply_added_with_existing_requirement_conflicts() {
        let delta =
            parse_delta("## ADDED Requirements\n\n### Requirement: User login\n\nbody\n").unwrap();
        let main = "### Requirement: User login\n\nold body\n";
        let err = apply_delta(Some(main), &delta).expect_err("err");
        match err {
            SpecDeltaError::Conflict {
                requirement,
                operation,
            } => {
                assert_eq!(requirement, "User login");
                assert_eq!(operation, "ADDED");
            }
            _ => panic!("expected Conflict ADDED"),
        }
    }

    #[test]
    fn apply_modified_replaces_full_block() {
        let main = "### Requirement: Token rotation\n\n#### Scenario: Old one\n\nbody\n";
        let new_block = "### Requirement: Token rotation\n\n#### Scenario: S1\n\n#### Scenario: S2\n\n#### Scenario: S3\n";
        let delta = parse_delta(&format!("## MODIFIED Requirements\n\n{new_block}\n")).unwrap();
        let (out, summary) = apply_delta(Some(main), &delta).expect("ok");
        assert!(out.contains("#### Scenario: S1"));
        assert!(out.contains("#### Scenario: S2"));
        assert!(out.contains("#### Scenario: S3"));
        assert!(!out.contains("Old one"));
        assert_eq!(summary.modified_count, 1);
    }

    #[test]
    fn apply_modified_not_found_conflicts() {
        let main = "### Requirement: Other\n\nbody\n";
        let delta =
            parse_delta("## MODIFIED Requirements\n\n### Requirement: Missing Req\n\nnew body\n")
                .unwrap();
        let err = apply_delta(Some(main), &delta).expect_err("err");
        match err {
            SpecDeltaError::Conflict {
                requirement,
                operation,
            } => {
                assert_eq!(requirement, "Missing Req");
                assert_eq!(operation, "MODIFIED");
            }
            _ => panic!("expected Conflict MODIFIED"),
        }
    }

    #[test]
    fn apply_removed_deletes_target_block() {
        let main = "### Requirement: A\n\nbody A\n\n### Requirement: B\n\nbody B\n\n### Requirement: C\n\nbody C\n";
        let delta = parse_delta("## REMOVED Requirements\n\n### Requirement: B\n").unwrap();
        let (out, summary) = apply_delta(Some(main), &delta).expect("ok");
        assert!(out.contains("### Requirement: A"));
        assert!(!out.contains("### Requirement: B"));
        assert!(out.contains("### Requirement: C"));
        // A 與 C 之間不該有多餘空行（至多一個空行）
        assert!(!out.contains("\n\n\n\n"), "no triple+ blank lines");
        assert_eq!(summary.removed_count, 1);
    }

    #[test]
    fn apply_removed_with_reason_metadata_documents_only() {
        let main = "### Requirement: Old Token Flow\n\nbody\n";
        let delta = parse_delta("## REMOVED Requirements\n\n### Requirement: Old Token Flow\n**Reason**: Replaced\n**Migration**: Update clients\n").unwrap();
        let (out, _) = apply_delta(Some(main), &delta).expect("ok");
        assert!(!out.contains("Old Token Flow"));
        assert!(!out.contains("Replaced"));
        assert!(!out.contains("Update clients"));
    }

    #[test]
    fn apply_renamed_changes_heading_only() {
        let main = "### Requirement: User login\n\n#### Scenario: Email login\n\nbody\n";
        let delta = parse_delta(
            "## RENAMED Requirements\n\n### Requirement: Sign-in\n**FROM:** User login\n**TO:** Sign-in\n",
        )
        .unwrap();
        let (out, summary) = apply_delta(Some(main), &delta).expect("ok");
        assert!(out.contains("### Requirement: Sign-in"));
        assert!(out.contains("#### Scenario: Email login"));
        assert!(!out.contains("Requirement: User login"));
        assert_eq!(summary.renamed_count, 1);
    }

    #[test]
    fn apply_renamed_missing_from_in_main_conflicts() {
        let main = "### Requirement: Other\n";
        let delta = parse_delta(
            "## RENAMED Requirements\n\n### Requirement: Sign-in\n**FROM:** User login\n**TO:** Sign-in\n",
        )
        .unwrap();
        let err = apply_delta(Some(main), &delta).expect_err("err");
        assert!(matches!(
            err,
            SpecDeltaError::Conflict {
                operation: "RENAMED",
                ..
            }
        ));
    }

    #[test]
    fn apply_order_rename_then_modify() {
        let main = "### Requirement: A\n\nold body\n";
        let delta_text = "## MODIFIED Requirements\n\n### Requirement: B\n\nnew body\n\n## RENAMED Requirements\n\n### Requirement: B\n**FROM:** A\n**TO:** B\n";
        let delta = parse_delta(delta_text).expect("parse");
        let (out, _) = apply_delta(Some(main), &delta).expect("ok");
        assert!(out.contains("### Requirement: B"));
        assert!(out.contains("new body"));
        assert!(!out.contains("old body"));
    }

    #[test]
    fn apply_summary_counts_match_delta() {
        let delta_text = "## ADDED Requirements\n\n### Requirement: A1\n\nbody\n\n### Requirement: A2\n\nbody\n\n## MODIFIED Requirements\n\n### Requirement: M1\n\nnew\n\n## REMOVED Requirements\n\n### Requirement: R1\n";
        let delta = parse_delta(delta_text).expect("parse");
        let main = "### Requirement: M1\n\nold\n\n### Requirement: R1\n\nbody\n";
        let (_, summary) = apply_delta(Some(main), &delta).expect("ok");
        assert_eq!(summary.added_count, 2);
        assert_eq!(summary.modified_count, 1);
        assert_eq!(summary.removed_count, 1);
        assert_eq!(summary.renamed_count, 0);
        assert!(!summary.created_main_spec);
    }
}
