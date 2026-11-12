//! The skills a Server gives a checkout (spec workspace-tools「remote 專案的技能取自
//! Server」): which tools to ask for, the request, and the answer in the form the
//! engine's sync plan installs. The CLI's `update` and the desktop app share these.

use crate::client::Client;
use crate::RemoteError;
use speclink_core::config::Invocation;
use speclink_core::init::{RenderedTool, ServedSkills, ToolSelection};
use speclink_core::skills::Tool;
use speclink_protocol::skills::{CustomTarget, SkillTarget, SkillsRequest, SkillsResponse};

/// The request targets of a checkout's tool selection, in the order the sync plan
/// writes them: the built-ins (claude, codex, copilot, pi, cursor), then the
/// descriptors in list order. The selection is the one the local sync resolves from
/// `.speclink.yaml`, so both paths agree on which tools a checkout has; a descriptor
/// that is not valid is no target, and the selection carries its error.
pub fn targets_of(selection: &ToolSelection) -> Vec<SkillTarget> {
    let builtins = Tool::ALL
        .into_iter()
        .filter(|tool| selection.builtins.contains(tool))
        .map(|tool| SkillTarget { tool: tool.name().to_string(), custom: None });
    let customs = selection.customs.iter().map(|custom| SkillTarget {
        tool: custom.name.clone(),
        custom: Some(CustomTarget {
            invocation: match custom.invocation {
                Invocation::Cli => "cli",
                Invocation::ToolCall => "tool-call",
            }
            .to_string(),
        }),
    });
    builtins.chain(customs).collect()
}

/// Ask the Server for the skills of `targets`: digests only, or with each skill's
/// text when `contents` is true.
pub fn fetch(client: &Client, targets: &[SkillTarget], contents: bool) -> Result<SkillsResponse, RemoteError> {
    client.skills(&SkillsRequest { targets: targets.to_vec(), contents })
}

/// A response that was asked for with contents, as the sync plan takes it. A skill
/// without its text is refused: there is nothing to install for it.
pub fn rendered(response: SkillsResponse) -> Result<Vec<RenderedTool>, RemoteError> {
    response
        .targets
        .into_iter()
        .map(|target| {
            let skills = target
                .skills
                .into_iter()
                .map(|skill| match skill.content {
                    Some(content) => Ok((skill.name, content)),
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
            digests: target.skills.iter().map(|skill| (skill.name.clone(), skill.digest.clone())).collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_core::config::AppConfig;
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
    fn fetch_posts_the_targets_and_reads_the_answer() {
        let (server, base, sent) = serve(
            200,
            r#"{"version":"v1.58.0","targets":[{"tool":"claude","skills":[{"name":"apply","digest":"sha256:ab","content":"text\n"}]}]}"#,
        );
        let client = Client::new(&base, "tok", Some("backend"));
        let response = fetch(&client, &[target("claude", None)], true).expect("the skills");
        assert_eq!(response.version, "v1.58.0");
        assert_eq!(response.targets[0].skills[0].content.as_deref(), Some("text\n"));

        let (method, path, body, headers) = sent.lock().unwrap().last().cloned().expect("one request");
        assert_eq!((method.as_str(), path.as_str()), ("POST", "/api/speclink/v1/projects/demo/skills"));
        assert_eq!(body, r#"{"targets":[{"tool":"claude"}],"contents":true}"#);
        let header = |name: &str| headers.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str());
        assert_eq!(header("x-speclink-api-version"), Some(speclink_protocol::API_VERSION));
        assert_eq!(header("x-speclink-repo"), Some("backend"));

        fetch(&client, &[target("my-agent", Some("tool-call"))], false).expect("the digests");
        let (_, _, body, _) = sent.lock().unwrap().last().cloned().unwrap();
        assert_eq!(body, r#"{"targets":[{"tool":"my-agent","custom":{"invocation":"tool-call"}}]}"#);
        server.unblock();
    }

    #[test]
    fn fetch_gives_the_servers_refusal_as_a_remote_error() {
        let (server, base, _) = serve(500, "{}");
        let error = fetch(&Client::new(&base, "tok", None), &[target("claude", None)], true).expect_err("a 500");
        assert_eq!(error.status, Some(500));
        server.unblock();
    }

    fn entry(name: &str, content: Option<&str>) -> SkillEntry {
        SkillEntry { name: name.into(), digest: "sha256:00".into(), content: content.map(str::to_string) }
    }

    #[test]
    fn rendered_is_each_tools_names_and_texts() {
        let response = SkillsResponse {
            version: "v1.58.0".into(),
            targets: vec![
                TargetSkills { tool: "claude".into(), skills: vec![entry("apply", Some("a\n")), entry("discuss", Some("d\n"))] },
                TargetSkills { tool: "my-agent".into(), skills: vec![entry("apply", Some("n\n"))] },
            ],
        };
        assert_eq!(
            rendered(response).unwrap(),
            [
                RenderedTool { tool: "claude".into(), skills: vec![("apply".into(), "a\n".into()), ("discuss".into(), "d\n".into())] },
                RenderedTool { tool: "my-agent".into(), skills: vec![("apply".into(), "n\n".into())] },
            ]
        );
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
                digests: vec![("apply".into(), "sha256:00".into()), ("discuss".into(), "sha256:00".into())],
            }]
        );
    }

    #[test]
    fn rendered_refuses_a_skill_without_its_text() {
        let response = SkillsResponse {
            version: "v1.58.0".into(),
            targets: vec![TargetSkills { tool: "claude".into(), skills: vec![entry("apply", None)] }],
        };
        let error = rendered(response).expect_err("no text, nothing to install");
        assert!(error.message.contains("apply") && error.message.contains("claude"), "{}", error.message);
    }
}
