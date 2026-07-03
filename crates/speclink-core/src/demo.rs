//! Generate a demo change with sample data. The theme rotates randomly per run and the
//! name/adjective pools mirror Spectra's (probed from 2.3.1; change names use the slx-
//! prefix instead of spx-).

use crate::paths::Paths;
use crate::util;
use anyhow::Result;
use std::path::PathBuf;

/// (theme, proposal, design, tasks, spec) — one entry per Spectra demo theme.
const THEMES: &[(&str, &str, &str, &str, &str)] = &[
    (
        "access-control",
        include_str!("../assets/demo/access-control/proposal.md"),
        include_str!("../assets/demo/access-control/design.md"),
        include_str!("../assets/demo/access-control/tasks.md"),
        include_str!("../assets/demo/access-control/spec.md"),
    ),
    (
        "audit-trail",
        include_str!("../assets/demo/audit-trail/proposal.md"),
        include_str!("../assets/demo/audit-trail/design.md"),
        include_str!("../assets/demo/audit-trail/tasks.md"),
        include_str!("../assets/demo/audit-trail/spec.md"),
    ),
    (
        "batch-export",
        include_str!("../assets/demo/batch-export/proposal.md"),
        include_str!("../assets/demo/batch-export/design.md"),
        include_str!("../assets/demo/batch-export/tasks.md"),
        include_str!("../assets/demo/batch-export/spec.md"),
    ),
    (
        "keyboard-macros",
        include_str!("../assets/demo/keyboard-macros/proposal.md"),
        include_str!("../assets/demo/keyboard-macros/design.md"),
        include_str!("../assets/demo/keyboard-macros/tasks.md"),
        include_str!("../assets/demo/keyboard-macros/spec.md"),
    ),
    (
        "real-time-sync",
        include_str!("../assets/demo/real-time-sync/proposal.md"),
        include_str!("../assets/demo/real-time-sync/design.md"),
        include_str!("../assets/demo/real-time-sync/tasks.md"),
        include_str!("../assets/demo/real-time-sync/spec.md"),
    ),
    (
        "smart-search",
        include_str!("../assets/demo/smart-search/proposal.md"),
        include_str!("../assets/demo/smart-search/design.md"),
        include_str!("../assets/demo/smart-search/tasks.md"),
        include_str!("../assets/demo/smart-search/spec.md"),
    ),
    (
        "snapshot-restore",
        include_str!("../assets/demo/snapshot-restore/proposal.md"),
        include_str!("../assets/demo/snapshot-restore/design.md"),
        include_str!("../assets/demo/snapshot-restore/tasks.md"),
        include_str!("../assets/demo/snapshot-restore/spec.md"),
    ),
    (
        "theme-engine",
        include_str!("../assets/demo/theme-engine/proposal.md"),
        include_str!("../assets/demo/theme-engine/design.md"),
        include_str!("../assets/demo/theme-engine/tasks.md"),
        include_str!("../assets/demo/theme-engine/spec.md"),
    ),
];

const WORDS: &[&str] = &[
    "bright", "calm", "dark", "eager", "fast", "gentle", "happy", "keen", "light", "neat",
    "proud", "quick", "rare", "sharp", "tall", "vivid", "warm", "bold", "cool", "deep",
];
const POKEMON: &[&str] = &[
    "pikachu", "charmander", "bulbasaur", "squirtle", "eevee", "snorlax", "gengar",
    "jigglypuff", "mewtwo", "dragonite", "lucario", "gardevoir", "charizard", "gyarados",
    "arcanine", "lapras", "umbreon", "absol", "togekiss", "rayquaza",
];

pub struct DemoOutcome {
    pub name: String,
    pub theme: String,
    pub path: PathBuf,
}

pub fn generate(paths: &Paths) -> Result<DemoOutcome> {
    let (theme, proposal, design, tasks, spec) = THEMES[util::pseudo_random(THEMES.len())];
    let word = WORDS[util::pseudo_random(WORDS.len())];
    let mon = POKEMON[util::pseudo_random(POKEMON.len())];
    let name = format!("slx-{word}-{mon}");
    let dir = paths.change_dir(&name);

    let created = util::today();
    let identity = util::git_identity(&paths.root);
    let mut meta = format!("schema: spec-driven\ncreated: {created}\n");
    if let Some(id) = identity {
        meta.push_str(&format!("created_by: {id}\n"));
    }

    util::write_file(&dir.join(".openspec.yaml"), &meta)?;
    util::write_file(&dir.join("proposal.md"), proposal)?;
    util::write_file(&dir.join("design.md"), design)?;
    util::write_file(&dir.join("tasks.md"), tasks)?;
    util::write_file(&dir.join("specs").join(theme).join("spec.md"), spec)?;

    Ok(DemoOutcome {
        name,
        theme: theme.to_string(),
        path: dir,
    })
}
