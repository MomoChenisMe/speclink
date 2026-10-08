mod mode_resolution;
mod no_direct_fs;
mod no_process_env;
mod render_golden;
mod skill_authoring;
mod skill_verbization;

/// Each line of `text` (newline included) with its byte offset, skipping fenced code
/// blocks and their fence lines: the skill-source checks must not read code as prose or
/// as a heading.
fn lines_outside_fences(text: &str) -> Vec<(usize, &str)> {
    let mut kept = Vec::new();
    let mut offset = 0;
    let mut in_fence = false;
    for line in text.split_inclusive('\n') {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
        } else if !in_fence {
            kept.push((offset, line));
        }
        offset += line.len();
    }
    kept
}

/// A hard-coded `/speclink-<name>` skill reference, the name in group 1. The character
/// before it must not belong to a path, so `.claude/skills/speclink-apply` does not match.
fn slash_skill_reference() -> regex::Regex {
    regex::Regex::new(r"(?:^|[^A-Za-z0-9_./:])(/speclink-[a-z][a-z0-9-]*)").unwrap()
}
