//! Generate a demo change with sample data (batch-export theme).

use crate::paths::Paths;
use crate::util;
use anyhow::Result;
use std::path::PathBuf;

const DEMO_PROPOSAL: &str = include_str!("../assets/demo/proposal.md");
const DEMO_DESIGN: &str = include_str!("../assets/demo/design.md");
const DEMO_TASKS: &str = include_str!("../assets/demo/tasks.md");
const DEMO_SPEC: &str = include_str!("../assets/demo/specs/batch-export/spec.md");

const WORDS: &[&str] = &[
    "cool", "tall", "brave", "swift", "calm", "bright", "keen", "bold", "wise", "quick",
];
const POKEMON: &[&str] = &[
    "bulbasaur", "charmander", "squirtle", "pikachu", "eevee", "gyarados", "snorlax", "gengar",
    "lapras", "dragonite",
];

pub struct DemoOutcome {
    pub name: String,
    pub theme: String,
    pub path: PathBuf,
}

pub fn generate(paths: &Paths) -> Result<DemoOutcome> {
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
    util::write_file(&dir.join("proposal.md"), DEMO_PROPOSAL)?;
    util::write_file(&dir.join("design.md"), DEMO_DESIGN)?;
    util::write_file(&dir.join("tasks.md"), DEMO_TASKS)?;
    util::write_file(&dir.join("specs").join("batch-export").join("spec.md"), DEMO_SPEC)?;

    Ok(DemoOutcome {
        name,
        theme: "batch-export".to_string(),
        path: dir,
    })
}
