//! Shared device routing and structured diagnostics. No project or desktop I/O.
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyMode {
    #[default]
    System,
    Manual,
    Direct,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProxyPreferences {
    pub mode: ProxyMode,
    pub url: Option<String>,
}
impl Default for ProxyPreferences {
    fn default() -> Self {
        Self {
            mode: ProxyMode::System,
            url: None,
        }
    }
}
pub fn validate_proxy_url(raw: &str) -> Result<url::Url, String> {
    let parsed = url::Url::parse(raw).map_err(|_| "proxy_invalid")?;
    let explicit_port = raw
        .strip_prefix("http://")
        .and_then(|s| s.trim_end_matches('/').rsplit_once(':'))
        .and_then(|(_, p)| p.parse::<u16>().ok())
        .is_some_and(|p| p > 0);
    if raw.len() > 2048
        || parsed.scheme() != "http"
        || parsed.host_str().is_none()
        || !explicit_port
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.path() != "/"
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err("proxy_invalid".into());
    }
    Ok(parsed)
}
#[derive(Clone)]
pub struct ProxyPolicy {
    pub settings: ProxyPreferences,
    client_route_failed: Arc<AtomicBool>,
    system: Arc<hyper_util::client::proxy::matcher::Matcher>,
}
impl ProxyPolicy {
    pub fn snapshot(settings: ProxyPreferences) -> Result<Self, String> {
        match settings.mode {
            ProxyMode::Manual => {
                validate_proxy_url(settings.url.as_deref().unwrap_or(""))?;
            }
            _ if settings.url.is_some() => return Err("proxy_invalid".into()),
            _ => {}
        }
        Ok(Self {
            settings,
            client_route_failed: Arc::new(AtomicBool::new(false)),
            system: Arc::new(hyper_util::client::proxy::matcher::Matcher::from_system()),
        })
    }
    pub fn configure_client(
        &self,
        builder: reqwest::ClientBuilder,
    ) -> Result<reqwest::ClientBuilder, String> {
        for uri in [
            "https://speclink-proxy-validation.invalid/",
            "http://speclink-proxy-validation.invalid/",
        ] {
            self.route(uri)?;
        }
        let _ = ureq::rustls::crypto::ring::default_provider().install_default();
        let policy = self.clone();
        Ok(builder.no_proxy().proxy(reqwest::Proxy::custom(move |url| {
            policy.client_route(url.as_str())
        })))
    }
    fn client_route(&self, destination: &str) -> Option<url::Url> {
        match self.route(destination) {
            Ok(route) => route,
            Err(_) => {
                self.client_route_failed.store(true, Ordering::Release);
                // reqwest's selector cannot return an error. A closed local port prevents
                // its None => direct behavior; callers surface the latched policy error.
                Some(url::Url::parse("http://127.0.0.1:0").expect("fixed blocking route"))
            }
        }
    }
    pub fn client_error(&self) -> Option<&'static str> {
        self.client_route_failed
            .load(Ordering::Acquire)
            .then_some("proxy_unsupported")
    }
    pub fn route(&self, destination: &str) -> Result<Option<url::Url>, String> {
        match self.settings.mode {
            ProxyMode::Direct => Ok(None),
            ProxyMode::Manual => {
                validate_proxy_url(self.settings.url.as_deref().unwrap_or("")).map(Some)
            }
            ProxyMode::System => {
                let destination: http::Uri = destination.parse().map_err(|_| "proxy_invalid")?;
                let Some(route) = self.system.intercept(&destination) else {
                    return Ok(None);
                };
                if route.basic_auth().is_some() || route.raw_auth().is_some() {
                    return Err("proxy_unsupported".into());
                }
                let mut proxy =
                    url::Url::parse(&route.uri().to_string()).map_err(|_| "proxy_unsupported")?;
                if proxy.scheme() != "http" || proxy.host_str().is_none() {
                    return Err("proxy_unsupported".into());
                }
                if proxy.port().is_none() {
                    let _ = proxy.set_port(Some(80));
                }
                Ok(Some(proxy))
            }
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolvedRoute {
    Direct,
    HttpProxy,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticArea {
    Frontend,
    Updater,
    SecurityData,
    NetworkProbe,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticAction {
    Started,
    Completed,
    Failed,
    RenderError,
    UnhandledError,
    UnhandledRejection,
    Policy,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ErrorDetail {
    pub kind: String,
    pub code: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkDiagnostic {
    pub format_version: u8,
    pub timestamp: String,
    pub operation_id: String,
    pub area: DiagnosticArea,
    pub event: DiagnosticAction,
    pub duration_ms: Option<u64>,
    pub destination: Option<String>,
    pub proxy_mode: Option<ProxyMode>,
    pub route: Option<ResolvedRoute>,
    pub proxy_endpoint: Option<String>,
    pub http_status: Option<u16>,
    pub error_code: Option<String>,
    pub error_chain: Vec<ErrorDetail>,
    pub dropped_count: u64,
}
impl NetworkDiagnostic {
    pub fn set_route(&mut self, policy: &ProxyPolicy, destination: &str) -> Result<(), String> {
        self.proxy_mode = Some(policy.settings.mode.clone());
        let route = policy.route(destination)?;
        self.route = Some(if route.is_some() {
            ResolvedRoute::HttpProxy
        } else {
            ResolvedRoute::Direct
        });
        self.proxy_endpoint = route.map(|url| url.to_string());
        if policy.settings.mode == ProxyMode::System && self.proxy_endpoint.is_none() {
            self.error_code = Some("system-no-static-route".into());
        }
        Ok(())
    }
    pub fn new(area: DiagnosticArea, event: DiagnosticAction, operation_id: String) -> Self {
        Self {
            format_version: 1,
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation_id,
            area,
            event,
            duration_ms: None,
            destination: None,
            proxy_mode: None,
            route: None,
            proxy_endpoint: None,
            http_status: None,
            error_code: None,
            error_chain: Vec::new(),
            dropped_count: 0,
        }
    }
}
pub type DiagnosticSink = Arc<dyn Fn(NetworkDiagnostic) + Send + Sync>;

pub fn request_error(error: &reqwest::Error) -> (String, Option<u16>, Vec<ErrorDetail>) {
    let status = error.status().map(|status| status.as_u16());
    let initial = if status == Some(407) {
        "proxy-auth"
    } else if error.is_timeout() {
        "timeout"
    } else if error.is_decode() {
        "parse"
    } else if status.is_some() {
        "http"
    } else if error.is_connect() {
        "connect"
    } else {
        "unknown"
    };
    let (code, chain) = error_details(error, initial);
    (code, status, chain)
}

/// Retain typed cause information, never third-party messages or URL-bearing Debug output.
pub fn error_details(
    error: &(dyn std::error::Error + 'static),
    initial: &str,
) -> (String, Vec<ErrorDetail>) {
    let mut kind = initial.to_owned();
    let mut chain = Vec::new();
    let mut current = Some(error);
    for _ in 0..16 {
        let Some(error) = current else {
            break;
        };
        // hyper-util 0.1.20 does not export TunnelError. Match only its exact,
        // fixed CONNECT authentication cause; never retain third-party text or
        // infer an HTTP status that reqwest does not expose.
        if error.source().is_none()
            && error.to_string() == "tunnel error: proxy authorization required"
        {
            kind = "proxy-auth".into();
            chain.push(ErrorDetail {
                kind: kind.clone(),
                code: kind.clone(),
            });
        }
        if let Some(io) = error.downcast_ref::<std::io::Error>() {
            if io.kind() == std::io::ErrorKind::TimedOut {
                kind = "timeout".into();
            }
            if let Some(code) = io.raw_os_error() {
                chain.push(ErrorDetail {
                    kind: kind.clone(),
                    code: format!("os-{code}"),
                });
            }
            if let Some(tls) = io
                .get_ref()
                .and_then(|e| e.downcast_ref::<ureq::rustls::Error>())
            {
                kind = "tls".into();
                chain.push(ErrorDetail {
                    kind: kind.clone(),
                    code: match tls {
                        ureq::rustls::Error::InvalidCertificate(
                            ureq::rustls::CertificateError::UnknownIssuer,
                        ) => "unknown-issuer",
                        _ => "invalid-certificate",
                    }
                    .into(),
                });
            }
        }
        if error.is::<ureq::rustls::Error>() {
            kind = "tls".into();
            chain.push(ErrorDetail {
                kind: kind.clone(),
                code: "invalid-certificate".into(),
            });
        }
        current = error.source();
    }
    if chain.is_empty() {
        chain.push(ErrorDetail {
            kind: kind.clone(),
            code: kind.clone(),
        });
    }
    (kind, chain)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn malformed_update_response_keeps_parse_code_without_response_body() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let _ = stream.read(&mut [0; 4096]);
            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 14\r\nConnection: close\r\n\r\nsecret-invalid").unwrap();
        });
        let policy = ProxyPolicy::snapshot(ProxyPreferences {
            mode: ProxyMode::Direct,
            url: None,
        })
        .unwrap();
        let client = policy
            .configure_client(reqwest::Client::builder())
            .unwrap()
            .build()
            .unwrap();
        let error = client
            .get(format!("http://{address}"))
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap_err();
        let (code, status, chain) = request_error(&error);
        assert_eq!(code, "parse");
        assert_eq!(status, None);
        assert!(!serde_json::to_string(&chain).unwrap().contains("secret"));
        server.join().unwrap();
    }
    #[test]
    fn unsupported_redirect_route_is_blocked_instead_of_panicking_or_going_direct() {
        let mut policy = ProxyPolicy::snapshot(ProxyPreferences::default()).unwrap();
        policy.system = Arc::new(
            hyper_util::client::proxy::matcher::Matcher::builder()
                .https("socks5://proxy.example:1080")
                .no("speclink-proxy-validation.invalid")
                .build(),
        );
        assert!(policy.configure_client(reqwest::Client::builder()).is_ok());
        assert!(policy.client_route("https://github.com/").is_some());
        assert_eq!(policy.client_error(), Some("proxy_unsupported"));
    }
    #[test]
    fn unsupported_route_is_logged_before_any_request_is_sent() {
        use crate::security_data::{DataJob, DataTransport, OfficialTransport};
        let mut policy = ProxyPolicy::snapshot(ProxyPreferences::default()).unwrap();
        policy.system = Arc::new(
            hyper_util::client::proxy::matcher::Matcher::builder()
                .https("http://user:secret@proxy:8080")
                .build(),
        );
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = events.clone();
        let job = DataJob::new(Arc::new(AtomicBool::new(false)));
        let transport = OfficialTransport::with_policy(
            &job,
            policy,
            Arc::new(move |event| sink.lock().unwrap().push(event)),
        );
        assert!(transport.fetch("https://github.com/", &job).is_err());
        let events = events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].error_code.as_deref(), Some("proxy_unsupported"));
        assert!(!serde_json::to_string(&events[0])
            .unwrap()
            .contains("secret"));
    }
    #[test]
    fn diagnostic_schema_rejects_unknown_routes() {
        let event = NetworkDiagnostic::new(
            DiagnosticArea::Frontend,
            DiagnosticAction::RenderError,
            ulid::Ulid::new().to_string(),
        );
        let mut value = serde_json::to_value(event).unwrap();
        value["route"] = "arbitrary-text".into();
        assert!(serde_json::from_value::<NetworkDiagnostic>(value).is_err());
    }
    #[test]
    fn typed_tls_failures_retain_certificate_evidence_without_messages() {
        let error = std::io::Error::other(ureq::rustls::Error::InvalidCertificate(
            ureq::rustls::CertificateError::UnknownIssuer,
        ));
        let (kind, chain) = error_details(&error, "connect");
        assert_eq!(kind, "tls");
        assert!(chain.iter().any(|detail| detail.code == "unknown-issuer"));
        let mut policy = ProxyPolicy::snapshot(ProxyPreferences::default()).unwrap();
        policy.system = Arc::new(hyper_util::client::proxy::matcher::Matcher::builder().build());
        let mut event = NetworkDiagnostic::new(
            DiagnosticArea::Updater,
            DiagnosticAction::Started,
            ulid::Ulid::new().to_string(),
        );
        event.set_route(&policy, "https://github.com/").unwrap();
        assert_eq!(event.error_code.as_deref(), Some("system-no-static-route"));
        assert_eq!(event.route, Some(ResolvedRoute::Direct));
    }
    #[test]
    fn manual_proxy_validation_and_routing() {
        for raw in [
            "https://proxy:8080",
            "http://u:secret@proxy:8080",
            "http://proxy",
            "http://proxy:0",
            "http://proxy:80/path",
            "http://proxy:80?token=secret",
        ] {
            assert!(validate_proxy_url(raw).is_err(), "{raw}");
        }
        for raw in [
            "http://127.0.0.1:8080",
            "http://[::1]:80",
            "http://proxy.example:80",
        ] {
            let policy = ProxyPolicy::snapshot(ProxyPreferences {
                mode: ProxyMode::Manual,
                url: Some(raw.into()),
            })
            .unwrap();
            assert!(policy.route("https://github.com/").unwrap().is_some());
        }
    }
    #[test]
    fn direct_and_system_no_proxy_snapshot() {
        let policy = ProxyPolicy::snapshot(ProxyPreferences {
            mode: ProxyMode::Direct,
            url: None,
        })
        .unwrap();
        assert!(policy.route("https://github.com").unwrap().is_none());
        let mut policy = ProxyPolicy::snapshot(ProxyPreferences::default()).unwrap();
        policy.system = Arc::new(
            hyper_util::client::proxy::matcher::Matcher::builder()
                .https("http://proxy.example:8080")
                .no("internal.example")
                .build(),
        );
        assert!(policy.route("https://github.com").unwrap().is_some());
        assert!(policy.route("https://internal.example").unwrap().is_none());
    }
    #[test]
    fn system_proxy_auth_and_unsupported_scheme_fail_loudly() {
        let mut policy = ProxyPolicy::snapshot(ProxyPreferences::default()).unwrap();
        for raw in ["http://user:secret@proxy:8080", "socks5://proxy:8080"] {
            policy.system = Arc::new(
                hyper_util::client::proxy::matcher::Matcher::builder()
                    .https(raw)
                    .build(),
            );
            assert!(policy.route("https://github.com").is_err());
        }
    }
}
