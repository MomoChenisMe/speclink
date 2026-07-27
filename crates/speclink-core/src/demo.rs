//! Generate a demo change with sample data. The theme rotates randomly per run and the
//! name/adjective pools are frozen (change names use the slx- prefix).

use crate::store::Store;
use crate::util;
use anyhow::Result;
use std::path::PathBuf;

/// (theme, proposal, design, tasks, spec) — one entry per demo theme.
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

/// `actor` is the Host-resolved display identity — None stamps no created_by.
pub fn generate(store: &dyn Store, actor: Option<&str>) -> Result<DemoOutcome> {
    let (theme, proposal, design, tasks, spec) = THEMES[util::pseudo_random(THEMES.len())];
    let word = WORDS[util::pseudo_random(WORDS.len())];
    let mon = POKEMON[util::pseudo_random(POKEMON.len())];
    let name = format!("slx-{word}-{mon}");

    let created = util::today();
    let mut meta = format!("schema: spec-driven\ncreated: {created}\n");
    if let Some(id) = actor {
        meta.push_str(&format!("created_by: {id}\n"));
    }

    let dir = store.create_change(&name, &meta)?;
    store.write_artifact(&name, "proposal.md", proposal)?;
    store.write_artifact(&name, "design.md", design)?;
    store.write_artifact(&name, "tasks.md", tasks)?;
    store.write_artifact(&name, &format!("specs/{theme}/spec.md"), spec)?;

    Ok(DemoOutcome {
        name,
        theme: theme.to_string(),
        path: dir,
    })
}
