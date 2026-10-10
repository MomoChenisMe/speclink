use super::init_tools::TempEnv;

#[test]
fn copilot_shared_update_and_switch_keep_real_names_and_user_files() {
    let env = TempEnv::new("copilot-shared-lifecycle");
    assert!(env
        .run(&["init", "--tools", "codex,copilot"])
        .status
        .success());
    let path = env.dir.join(".agents/skills/speclink-propose/SKILL.md");
    let before = std::fs::read(&path).unwrap();
    std::fs::create_dir_all(env.dir.join(".agents/skills/my-skill")).unwrap();
    std::fs::write(env.dir.join(".agents/skills/my-skill/SKILL.md"), "user").unwrap();
    let out = env.run(&["--no-color", "update"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("Updated skill files for: codex, copilot")
    );
    assert!(!out.stdout.contains(&0x1b) && !out.stderr.contains(&0x1b));
    for tools in ["copilot", "codex", "copilot,codex"] {
        assert!(env
            .run(&["init", "--force", "--tools", tools])
            .status
            .success());
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
    assert_eq!(env.builtins(), ["copilot", "codex"]);
    assert!(env
        .run(&["init", "--force", "--tools", "claude"])
        .status
        .success());
    assert!(!path.exists());
    assert_eq!(
        std::fs::read_to_string(env.dir.join(".agents/skills/my-skill/SKILL.md")).unwrap(),
        "user"
    );
}

#[test]
fn copilot_guard_refuses_without_writes_and_allows_explicit_downgrade() {
    let env = TempEnv::new("copilot-guard-cli");
    assert!(env.run(&["init", "--tools", "copilot"]).status.success());
    let path = env.dir.join(".agents/skills/speclink-propose/SKILL.md");
    let text = std::fs::read_to_string(&path).unwrap();
    let version = speclink_core::init::ASSET_VERSION;
    std::fs::write(&path, text.replace(version, "v99999.0.0")).unwrap();
    let before = env.snapshot();
    let out = env.run(&["update"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("--allow-downgrade"));
    assert_eq!(env.snapshot(), before);
    assert!(env
        .run(&["--no-color", "update", "--allow-downgrade"])
        .status
        .success());
    assert_eq!(std::fs::read_to_string(path).unwrap(), text);
}

#[test]
fn copilot_remote_init_generates_shared_skills_without_local_specs() {
    let env = TempEnv::new("copilot-remote");
    let out = env.run(&[
        "init",
        "--store",
        "remote",
        "--url",
        "https://team.example.com/speclink/projects/foo",
        "--repo",
        "backend",
        "--tools",
        "copilot",
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(env.builtins(), ["copilot"]);
    assert!(env.exists(".agents/skills/speclink-propose/SKILL.md"));
    assert!(!env.exists("openspec") && !env.exists(".github") && !env.exists("AGENTS.md"));
}
