//! The skills a Server gives a checkout (spec workspace-tools「remote 專案的技能取自
//! Server」): which tools to ask for, and what to do with the answer — install it, or
//! hand its digests to the engine's probe. The CLI's `update` and the desktop app share
//! these; each asks the Server through its own client ([`Client::skills`]).
//!
//! [`Client::skills`]: crate::client::Client::skills

use crate::RemoteError;
use speclink_core::config::Invocation;
use speclink_core::init::{RenderedSkill, RenderedTool, ServedSkills, SkillDigest, SyncPlan, ToolSelection, UpdateOutcome};
use speclink_protocol::skills::{CustomTarget, SkillTarget, SkillsResponse};
use std::path::Path;

/// The request targets of a checkout's tool selection, in the order the sync plan
/// writes them ([`ToolSelection::labels`]). The selection is the one the local sync
/// resolves from `.speclink.yaml`, so both paths agree on which tools a checkout has;
/// a descriptor that is not valid is no target, and the selection carries its error.
pub fn targets_of(selection: &ToolSelection) -> Vec<SkillTarget> {
    let invocation_of = |name: &str| {
        let custom = selection.customs.iter().find(|custom| custom.name == name)?;
        let invocation = match custom.invocation {
            Invocation::Cli => "cli",
            Invocation::ToolCall => "tool-call",
        };
        Some(CustomTarget { invocation: invocation.to_string() })
    };
    selection.labels().into_iter().map(|tool| SkillTarget { custom: invocation_of(&tool), tool }).collect()
}

/// What installing a Server's answer did.
#[derive(Debug)]
pub struct Installed {
    pub outcome: UpdateOutcome,
    /// The Server's skill version.
    pub version: String,
    /// The newer version the checkout had, when the Server's is older.
    pub replaced: Option<String>,
}

/// Install the skills of `response` — an answer asked for with contents — into the
/// checkout at `root`: the files become what the Server gave for each tool of
/// `selection`, and the other `speclink-` folders are removed. A tool the Server gives
/// no skill is left as it is, and named in the outcome's `unserved`. The downgrade guard
/// is not asked: a remote project's skills follow its Server. An answer that does not fit
/// the checkout (a skill without its text, a name that is no skill name, a tool that was
/// not asked for) is refused before any write.
pub fn install(root: &Path, selection: ToolSelection, response: SkillsResponse) -> anyhow::Result<Installed> {
    let version = response.version.clone();
    let plan = SyncPlan::from_rendered(root, selection, rendered(response)?)?;
    let replaced = plan.leading_version(&version);
    let outcome = plan.apply(root)?;
    Ok(Installed { outcome, version, replaced })
}

/// A response that was asked for with contents, as the sync plan takes it. A skill
/// without its text is refused: there is nothing to install for it.
fn rendered(response: SkillsResponse) -> Result<Vec<RenderedTool>, RemoteError> {
    response
        .targets
        .into_iter()
        .map(|target| {
            let skills = target
                .skills
                .into_iter()
                .map(|skill| match skill.content {
                    Some(content) => Ok(RenderedSkill { name: skill.name, content }),
                    None => Err(RemoteError {
                        message: format!(
                            "unexpected server response — skill '{}' of tool '{}' came without its text",
                            skill.name, target.tool
                        ),
                        reason: None,
                        status: None,
                        evidence: None,
                    }),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(RenderedTool { tool: target.tool, skills })
        })
        .collect()
}

/// A response by digest, as the engine's probe of a remote checkout takes it.
pub fn digests(response: &SkillsResponse) -> Vec<ServedSkills> {
    response
        .targets
        .iter()
        .map(|target| ServedSkills {
            tool: target.tool.clone(),
            digests: target
                .skills
                .iter()
                .map(|skill| SkillDigest { name: skill.name.clone(), digest: skill.digest.clone() })
                .collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::Client;
    use speclink_core::config::AppConfig;
    use speclink_protocol::skills::SkillsRequest;
    use speclink_protocol::skills::{SkillEntry, TargetSkills};
    use std::sync::{Arc, Mutex};

    /// The selection of a checkout whose `.speclink.yaml` is `config`.
    fn selection(dir: &std::path::Path, config: &str) -> ToolSelection {
        let path = dir.join(".speclink.yaml");
        std::fs::write(&path, config).unwrap();
        ToolSelection::resolve(dir, &AppConfig::load(&path).expect("the config parses"))
    }

    fn target(tool: &str, invocation: Option<&str>) -> SkillTarget {
        SkillTarget { tool: tool.into(), custom: invocation.map(|i| CustomTarget { invocation: i.into() }) }
    }

    #[test]
    fn targets_are_the_selections_builtins_then_its_descriptors() {
        let dir = tempfile::tempdir().unwrap();
        let sel = selection(
            dir.path(),
            "tools:\n  - name: my-agent\n    skills_dir: .my-agent/skills\n    invocation: tool-call\n  - agents\n  - claude\n  - name: plain\n    skills_dir: .plain/skills\n",
        );
        assert_eq!(
            targets_of(&sel),
            [
                target("claude", None),
                target("codex", None),
                target("my-agent", Some("tool-call")),
                target("plain", Some("cli")),
            ],
            "the plan's order; `agents` is codex; a descriptor without invocation is cli"
        );
    }

    #[test]
    fn a_descriptor_that_is_not_valid_is_no_target_and_the_selection_reports_it() {
        let dir = tempfile::tempdir().unwrap();
        let sel = selection(
            dir.path(),
            "tools:\n  - claude\n  - name: Bad Name\n    skills_dir: .bad/skills\n  - name: good\n    skills_dir: .good/skills\n",
        );
        let error = sel.descriptor_error.as_deref().expect("the selection carries the error");
        assert!(error.contains("Bad Name"), "{error}");
        assert_eq!(targets_of(&sel), [target("claude", None), target("good", Some("cli"))]);
    }

    #[test]
    fn without_a_tools_list_the_targets_follow_the_local_fallback() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(targets_of(&selection(dir.path(), "remote:\n  url: http://x/api/speclink/v1/projects/p\n")), []);
        std::fs::create_dir(dir.path().join(".claude")).unwrap();
        assert_eq!(
            targets_of(&selection(dir.path(), "remote:\n  url: http://x/api/speclink/v1/projects/p\n")),
            [target("claude", None)]
        );
    }

    /// A server that answers every request with `status` and `body`, and keeps what it
    /// was sent: (method, path, body, headers with lower-case names).
    type Sent = Arc<Mutex<Vec<(String, String, String, Vec<(String, String)>)>>>;

    fn serve(status: u16, body: &'static str) -> (Arc<tiny_http::Server>, String, Sent) {
        let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
        let base = format!("http://{}/api/speclink/v1/projects/demo", server.server_addr());
        let sent: Sent = Arc::default();
        let (looper, sink) = (server.clone(), sent.clone());
        std::thread::spawn(move || {
            for mut req in looper.incoming_requests() {
                let mut text = String::new();
                let _ = req.as_reader().read_to_string(&mut text);
                let headers =
                    req.headers().iter().map(|h| (h.field.to_string().to_ascii_lowercase(), h.value.to_string())).collect();
                sink.lock().unwrap().push((req.method().to_string(), req.url().to_string(), text, headers));
                let _ = req.respond(tiny_http::Response::from_string(body).with_status_code(status));
            }
        });
        (server, base, sent)
    }

    #[test]
    fn the_client_posts_the_targets_and_reads_the_answer() {
        let (server, base, sent) = serve(
            200,
            r#"{"version":"v1.58.0","targets":[{"tool":"claude","skills":[{"name":"apply","digest":"sha256:ab","content":"text\n"}]}]}"#,
        );
        let client = Client::new(&base, "tok", Some("backend"));
        let ask = |targets: Vec<SkillTarget>, contents| client.skills(&SkillsRequest { targets, contents });
        let response = ask(vec![target("claude", None)], true).expect("the skills");
        assert_eq!(response.version, "v1.58.0");
        assert_eq!(response.targets[0].skills[0].content.as_deref(), Some("text\n"));

        let (method, path, body, headers) = sent.lock().unwrap().last().cloned().expect("one request");
        assert_eq!((method.as_str(), path.as_str()), ("POST", "/api/speclink/v1/projects/demo/skills"));
        assert_eq!(body, r#"{"targets":[{"tool":"claude"}],"contents":true}"#);
        let header = |name: &str| headers.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str());
        assert_eq!(header("x-speclink-api-version"), Some(speclink_protocol::API_VERSION));
        assert_eq!(header("x-speclink-repo"), Some("backend"));

        ask(vec![target("my-agent", Some("tool-call"))], false).expect("the digests");
        let (_, _, body, _) = sent.lock().unwrap().last().cloned().unwrap();
        assert_eq!(body, r#"{"targets":[{"tool":"my-agent","custom":{"invocation":"tool-call"}}]}"#);
        server.unblock();
    }

    #[test]
    fn the_client_gives_the_servers_refusal_as_a_remote_error() {
        let (server, base, _) = serve(500, "{}");
        let request = SkillsRequest { targets: vec![target("claude", None)], contents: true };
        let error = Client::new(&base, "tok", None).skills(&request).expect_err("a 500");
        assert_eq!(error.status, Some(500));
        server.unblock();
    }

    fn entry(name: &str, content: Option<&str>) -> SkillEntry {
        SkillEntry { name: name.into(), digest: "sha256:00".into(), content: content.map(str::to_string) }
    }

    /// A remote checkout with claude, its skills rendered by the local templates.
    fn remote_checkout(dir: &std::path::Path) -> ToolSelection {
        let url = "https://team.example.test/api/speclink/v1/projects/acme";
        speclink_core::init::init_remote(dir, &[speclink_core::skills::Tool::Claude], true, url, None).unwrap();
        ToolSelection::resolve(dir, &AppConfig::load(&dir.join(".speclink.yaml")).unwrap())
    }

    fn served_text(name: &str, version: &str) -> String {
        format!("---\nname: speclink-{name}\nmetadata:\n  version: \"{version}\"\n---\n\nfrom the server\n")
    }

    fn answer(version: &str, skills: Vec<SkillEntry>) -> SkillsResponse {
        SkillsResponse { version: version.into(), targets: vec![TargetSkills { tool: "claude".into(), skills }] }
    }

    fn skill_file(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
        dir.join(".claude").join("skills").join(format!("speclink-{name}")).join("SKILL.md")
    }

    #[test]
    fn install_writes_the_servers_texts_and_removes_what_it_does_not_give() {
        let dir = tempfile::tempdir().unwrap();
        let selection = remote_checkout(dir.path());
        assert!(skill_file(dir.path(), "discuss").exists(), "the local templates have discuss");
        let texts = ["apply", "propose"].map(|name| (name, served_text(name, "v0.9.0")));
        let skills = texts.iter().map(|(name, text)| entry(name, Some(text))).collect();

        let installed = install(dir.path(), selection, answer("v0.9.0", skills)).expect("installed");
        assert_eq!(installed.outcome.updated, ["claude"]);
        assert_eq!(installed.version, "v0.9.0");
        assert_eq!(
            installed.replaced.as_deref(),
            Some(speclink_core::init::ASSET_VERSION),
            "the checkout had this engine's version, which is newer"
        );
        for (name, text) in &texts {
            assert_eq!(&std::fs::read_to_string(skill_file(dir.path(), name)).unwrap(), text);
        }
        assert!(!skill_file(dir.path(), "discuss").exists(), "a skill the Server does not give is removed");
    }

    #[test]
    fn install_leaves_a_tool_without_skills_alone_and_names_it() {
        let dir = tempfile::tempdir().unwrap();
        let selection = remote_checkout(dir.path());
        let before = std::fs::read_to_string(skill_file(dir.path(), "discuss")).unwrap();
        let installed = install(dir.path(), selection, answer("v1.58.0", Vec::new())).expect("nothing to install");
        assert_eq!(installed.outcome.unserved, ["claude"]);
        assert_eq!(installed.outcome.updated, Vec::<String>::new());
        assert_eq!(installed.replaced, None);
        assert_eq!(std::fs::read_to_string(skill_file(dir.path(), "discuss")).unwrap(), before);
    }

    #[test]
    fn install_names_no_replaced_version_when_the_servers_is_not_older() {
        let dir = tempfile::tempdir().unwrap();
        let selection = remote_checkout(dir.path());
        let text = served_text("apply", "v999.0.0");
        let installed = install(dir.path(), selection, answer("v999.0.0", vec![entry("apply", Some(&text))])).unwrap();
        assert_eq!(installed.replaced, None);
    }

    #[test]
    fn install_refuses_an_answer_that_does_not_fit_and_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let before = std::fs::read_to_string(skill_file(dir.path(), "propose")).ok();
        for (skills, names_it) in [
            (vec![entry("apply", None)], "without its text"),
            (vec![entry("../escape", Some("x"))], "skill name"),
        ] {
            let selection = remote_checkout(dir.path());
            let kept = std::fs::read_to_string(skill_file(dir.path(), "propose")).unwrap();
            let error = install(dir.path(), selection, answer("v1.58.0", skills)).expect_err("refused");
            assert!(error.to_string().contains(names_it), "{names_it}: {error}");
            assert_eq!(std::fs::read_to_string(skill_file(dir.path(), "propose")).unwrap(), kept, "{names_it}: zero writes");
        }
        assert_eq!(before, None, "the checkout started empty");
        assert!(!dir.path().join("escape").exists());
    }

    #[test]
    fn digests_are_each_tools_names_and_digests() {
        let response = SkillsResponse {
            version: "v1.58.0".into(),
            targets: vec![TargetSkills { tool: "claude".into(), skills: vec![entry("apply", None), entry("discuss", Some("d\n"))] }],
        };
        assert_eq!(
            digests(&response),
            [ServedSkills {
                tool: "claude".into(),
                digests: ["apply", "discuss"]
                    .map(|name| SkillDigest { name: name.into(), digest: "sha256:00".into() })
                    .to_vec(),
            }]
        );
    }

}
