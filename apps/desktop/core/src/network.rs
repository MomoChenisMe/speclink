//! Resolve the device preference snapshot before starting an operation.
use speclink_host::network::ProxyPolicy;
use std::path::Path;
pub fn policy_at(root: &Path) -> Result<ProxyPolicy, String> {
    ProxyPolicy::snapshot(crate::runtime_preferences::read(root)?.proxy)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_preferences::{self, ProxyMode, ProxyPreferences, RuntimePreferences};
    #[test]
    fn operation_keeps_its_proxy_snapshot_after_preference_change() {
        let root = tempfile::tempdir().unwrap();
        let value = RuntimePreferences {
            proxy: ProxyPreferences {
                mode: ProxyMode::Manual,
                url: Some("http://proxy:8080".into()),
            },
            ..Default::default()
        };
        runtime_preferences::write(root.path(), &value).unwrap();
        let before = policy_at(root.path()).unwrap();
        runtime_preferences::write(
            root.path(),
            &RuntimePreferences {
                proxy: ProxyPreferences {
                    mode: ProxyMode::Direct,
                    url: None,
                },
                ..Default::default()
            },
        )
        .unwrap();
        assert!(before.route("https://github.com/").unwrap().is_some());
        assert!(policy_at(root.path())
            .unwrap()
            .route("https://github.com/")
            .unwrap()
            .is_none());
    }
}

use serde::Serialize;
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeEndpointResult {
    pub destination: String,
    pub http_status: Option<u16>,
    pub error_code: Option<String>,
    pub duration_ms: u64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    pub operation_id: String,
    pub results: Vec<ProbeEndpointResult>,
}
pub fn prepare_probe(
    runtime: &crate::runtime_preferences::DeviceRuntime,
) -> Result<ProxyPolicy, String> {
    let preferences = crate::runtime_preferences::read(&runtime.root)?;
    if !preferences.debug_enabled {
        return Err("debug_disabled".into());
    }
    ProxyPolicy::snapshot(preferences.proxy)
}
pub async fn probe_update(
    runtime: &crate::runtime_preferences::DeviceRuntime,
    policy: ProxyPolicy,
    endpoints: Vec<String>,
) -> Result<ProbeResult, String> {
    let operation_id = ulid::Ulid::new().to_string();
    let mut results = Vec::new();
    let started = std::time::Instant::now();
    for endpoint in endpoints {
        if !url::Url::parse(&endpoint)
            .is_ok_and(|u| u.scheme() == "https" && u.host_str().is_some())
        {
            return Err("untrusted_source".into());
        }
        if started.elapsed() >= std::time::Duration::from_secs(20) {
            return Err("budget_exceeded".into());
        }
        let mut event = speclink_host::network::NetworkDiagnostic::new(
            speclink_host::network::DiagnosticArea::NetworkProbe,
            speclink_host::network::DiagnosticAction::Started,
            operation_id.clone(),
        );
        event.destination = Some(endpoint.clone());
        event.set_route(&policy, &endpoint)?;
        let _ = runtime.diagnostics.record(event.clone());
        let (mut result, chain) = probe_endpoint_with_timeout(
            &policy,
            &endpoint,
            std::time::Duration::from_secs(20).saturating_sub(started.elapsed()),
        )
        .await;
        event.event = if result.error_code.is_some() {
            speclink_host::network::DiagnosticAction::Failed
        } else {
            speclink_host::network::DiagnosticAction::Completed
        };
        event.http_status = result.http_status;
        event.duration_ms = Some(result.duration_ms);
        event.error_code = result.error_code.clone();
        event.error_chain = chain;
        let direct_system = policy.settings.mode == crate::runtime_preferences::ProxyMode::System
            && event.route == Some(speclink_host::network::ResolvedRoute::Direct);
        let _ = runtime.diagnostics.record(event);
        if direct_system && result.error_code.is_none() {
            result.error_code = Some("system-no-static-route".into());
        }
        results.push(result);
    }
    Ok(ProbeResult {
        operation_id,
        results,
    })
}
#[cfg(test)]
async fn probe_endpoint(
    policy: &ProxyPolicy,
    url: &str,
) -> (
    ProbeEndpointResult,
    Vec<speclink_host::network::ErrorDetail>,
) {
    probe_endpoint_with_timeout(policy, url, std::time::Duration::from_secs(20)).await
}
async fn probe_endpoint_with_timeout(
    policy: &ProxyPolicy,
    url: &str,
    timeout: std::time::Duration,
) -> (
    ProbeEndpointResult,
    Vec<speclink_host::network::ErrorDetail>,
) {
    let start = std::time::Instant::now();
    let mut result = ProbeEndpointResult {
        destination: url::Url::parse(url)
            .map(|u| u.origin().ascii_serialization())
            .unwrap_or_default(),
        http_status: None,
        error_code: None,
        duration_ms: 0,
    };
    let run = async {
        let builder = reqwest::Client::builder()
            .timeout(timeout)
            .user_agent("tauri-plugin-updater/2.10.1")
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if !probe_redirect_allowed(attempt.url().scheme(), attempt.previous().len()) {
                    attempt.error("untrusted_source")
                } else {
                    attempt.follow()
                }
            }));
        let builder = policy
            .configure_client(builder)
            .map_err(|_| ("proxy_unsupported".to_owned(), Vec::new()))?;
        let client = builder.build().map_err(probe_error)?;
        let mut response = client.get(url).send().await.map_err(probe_error)?;
        result.http_status = Some(response.status().as_u16());
        if !response.status().is_success() {
            return Err((
                if response.status().as_u16() == 407 {
                    "proxy-auth"
                } else {
                    "http"
                }
                .into(),
                Vec::new(),
            ));
        }
        let mut bytes = 0usize;
        while let Some(chunk) = response.chunk().await.map_err(probe_error)? {
            bytes += chunk.len();
            if bytes > 2 * 1024 * 1024 {
                return Err(("size_limit".into(), Vec::new()));
            }
        }
        Ok(())
    };
    let mut chain = match run.await {
        Ok(()) => Vec::new(),
        Err((code, chain)) => {
            result.error_code = Some(code);
            chain
        }
    };
    if let Some(code) = policy.client_error() {
        result.error_code = Some(code.into());
        chain = vec![speclink_host::network::ErrorDetail {
            kind: "unknown".into(),
            code: code.into(),
        }];
    }
    result.duration_ms = start.elapsed().as_millis() as u64;
    (result, chain)
}
fn probe_redirect_allowed(scheme: &str, redirects: usize) -> bool {
    scheme == "https" && redirects <= 3
}
fn probe_error(error: reqwest::Error) -> (String, Vec<speclink_host::network::ErrorDetail>) {
    speclink_host::network::error_details(
        &error,
        if error.is_timeout() {
            "timeout"
        } else if error.is_connect() {
            "connect"
        } else {
            "unknown"
        },
    )
}
#[cfg(test)]
mod probe_tests {
    use super::*;
    #[tokio::test]
    async fn connect_tunnel_success_does_not_disable_tls_or_hide_disconnects() {
        use std::io::{Read, Write};
        for reply in [
            &b"HTTP/1.1 200 Connection established\r\n\r\nnot-a-tls-record"[..],
            &b"HTTP/1.1 200 Connection established\r\n\r\n"[..],
            &b"broken-proxy-response\r\n\r\n"[..],
        ] {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let server = std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                stream.set_read_timeout(Some(std::time::Duration::from_secs(2))).unwrap();
                let mut request = [0; 4096];
                let count = stream.read(&mut request).unwrap();
                let _ = stream.write_all(reply);
                String::from_utf8_lossy(&request[..count]).into_owned()
            });
            let policy = ProxyPolicy::snapshot(crate::runtime_preferences::ProxyPreferences {
                mode: crate::runtime_preferences::ProxyMode::Manual, url: Some(format!("http://{address}")),
            }).unwrap();
            let (result, _) = probe_endpoint(&policy, "https://github.com/").await;
            assert!(server.join().unwrap().starts_with("CONNECT github.com:443"));
            assert!(result.error_code.is_some(), "a tunnel is not a successful TLS response");
            assert_eq!(result.http_status, None);
        }
    }
    #[test]
    fn probe_redirect_limit_preserves_https_and_three_hops() {
        assert!(probe_redirect_allowed("https", 3));
        assert!(!probe_redirect_allowed("https", 4));
        assert!(!probe_redirect_allowed("http", 1));
    }
    #[tokio::test]
    async fn probe_bounds_actual_response_body_and_timeout() {
        use std::io::{Read, Write};
        let policy = ProxyPolicy::snapshot(crate::runtime_preferences::ProxyPreferences {
            mode: crate::runtime_preferences::ProxyMode::Direct,
            url: None,
        })
        .unwrap();
        for (bytes, delay, expected) in [
            (2 * 1024 * 1024, 0, None),
            (2 * 1024 * 1024 + 1, 0, Some("size_limit")),
            (0, 100, Some("timeout")),
        ] {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let endpoint = format!("http://{}", listener.local_addr().unwrap());
            let server = std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0; 4096];
                let _ = stream.read(&mut request);
                std::thread::sleep(std::time::Duration::from_millis(delay));
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Length: {bytes}\r\nConnection: close\r\n\r\n"
                );
                let _ = stream.write_all(&vec![b'x'; bytes]);
            });
            let timeout = if delay > 0 {
                std::time::Duration::from_millis(20)
            } else {
                std::time::Duration::from_secs(20)
            };
            let (result, _) = probe_endpoint_with_timeout(&policy, &endpoint, timeout).await;
            assert_eq!(result.error_code.as_deref(), expected);
            server.join().unwrap();
        }
    }
    #[tokio::test]
    async fn explicit_probe_reports_http_407_and_debug_off_sends_nothing() {
        let root = tempfile::tempdir().unwrap();
        let runtime = crate::runtime_preferences::DeviceRuntime::new(root.path().to_path_buf());
        assert_eq!(prepare_probe(&runtime).err().unwrap(), "debug_disabled");
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            use std::io::{Read, Write};
            let (mut stream, _) = listener.accept().unwrap();
            let mut b = [0; 2048];
            let _ = stream.read(&mut b);
            stream
                .write_all(
                    b"HTTP/1.1 407 Proxy Authentication Required\r\nContent-Length: 0\r\n\r\n",
                )
                .unwrap();
        });
        let policy = ProxyPolicy::snapshot(crate::runtime_preferences::ProxyPreferences {
            mode: crate::runtime_preferences::ProxyMode::Direct,
            url: None,
        })
        .unwrap();
        let (result, _) = probe_endpoint(&policy, &url).await;
        assert_eq!(result.http_status, Some(407));
        assert_eq!(result.error_code.as_deref(), Some("proxy-auth"));
    }

    #[tokio::test]
    async fn https_connect_407_retains_proxy_auth_status_and_diagnostics() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                .unwrap();
            let mut request = [0; 4096];
            let count = stream.read(&mut request).unwrap();
            stream.write_all(b"HTTP/1.1 407 Proxy Authentication Required\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
            String::from_utf8_lossy(&request[..count]).into_owned()
        });
        let root = tempfile::tempdir().unwrap();
        let runtime = crate::runtime_preferences::DeviceRuntime::new(root.path().to_owned());
        runtime
            .write(crate::runtime_preferences::RuntimePreferences {
                debug_enabled: true,
                proxy: crate::runtime_preferences::ProxyPreferences {
                    mode: crate::runtime_preferences::ProxyMode::Manual,
                    url: Some(format!("http://{address}")),
                },
                ..Default::default()
            })
            .unwrap();
        let policy = prepare_probe(&runtime).unwrap();
        let result = probe_update(&runtime, policy, vec!["https://github.com/".into()])
            .await
            .unwrap();
        assert!(server.join().unwrap().starts_with("CONNECT github.com:443"));
        assert_eq!(result.results[0].http_status, None);
        assert_eq!(result.results[0].error_code.as_deref(), Some("proxy-auth"));
        runtime.diagnostics.flush();
        let log = std::fs::read_to_string(root.path().join("diagnostics/events.jsonl")).unwrap();
        let event: serde_json::Value = log
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .find(|entry| entry["event"] == "failed")
            .unwrap();
        assert_eq!(event["operationId"], result.operation_id);
        assert!(event["httpStatus"].is_null());
        assert_eq!(event["errorCode"], "proxy-auth");
    }

    #[tokio::test]
    async fn prepared_probe_uses_its_snapshot_without_reading_preferences_again() {
        let root = tempfile::tempdir().unwrap();
        let runtime = crate::runtime_preferences::DeviceRuntime::new(root.path().to_owned());
        runtime
            .write(crate::runtime_preferences::RuntimePreferences {
                debug_enabled: true,
                proxy: crate::runtime_preferences::ProxyPreferences {
                    mode: crate::runtime_preferences::ProxyMode::Direct,
                    url: None,
                },
                ..Default::default()
            })
            .unwrap();
        let policy = prepare_probe(&runtime).unwrap();
        std::fs::write(root.path().join("runtime-preferences.json"), "invalid").unwrap();
        assert!(policy.route("https://github.com/").unwrap().is_none());
        assert!(probe_update(&runtime, policy, Vec::new()).await.is_ok());
    }
}
