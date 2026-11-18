//! The one place this crate builds an HTTP agent (client-os-trust-store).
//!
//! # Sharp-edges audit
//!
//! Checked against the audit checklist (`speclink instructions --skill audit`)
//! when this module was written:
//!
//! - **No path turns the certificate check off.** `dangerous()` is in this
//!   crate's source once, in [`trust_config`], and installs the operating
//!   system's verifier — rustls offers no other way to install a verifier.
//!   The crate has no verifier of its own. No flag, setting or environment
//!   variable of ours changes the trust source.
//! - **The fallback cannot be forced by a server.** [`builder_with`] falls
//!   back only when the trust settings cannot be built, once per process,
//!   before any request. A certificate refused on one request goes to
//!   [`translate_transport_error`]; nothing retries with other trust settings. The
//!   fallback itself still checks certificates, against ureq's built-in list
//!   of public CAs.
//! - **The message carries the name of the reason and nothing else.**
//!   [`reason_name`] keeps the ASCII letters and digits of the rustls name. No
//!   token, no URL or query string, and no text from the other side's
//!   certificate (which could hold terminal control characters) reaches it.
//! - **The login service's agent still follows no redirect.** `oauth::agent`
//!   adds `redirects(0)` on top of [`agent_builder`]; the oauth test
//!   `a_redirect_from_the_login_service_is_a_failure_not_a_success` holds it.
//! - **The safe way is the easy way.** [`agent_builder`] takes no argument, so
//!   a caller cannot pass weaker settings, and the guard tests below fail when
//!   another module builds an agent, uses a ureq shortcut that makes one, or
//!   translates a connection-layer failure without looking at the certificate.
//! - **Not ours to decide:** on Linux the library that reads the system's CA
//!   files also obeys `SSL_CERT_FILE` and `SSL_CERT_DIR`. Whoever sets the
//!   environment of the process already controls it.

use std::sync::{Arc, OnceLock};

use rustls::pki_types::CertificateDer;

use crate::RemoteError;

/// The builder every connection of this crate starts from: the operating
/// system's trust store decides which server certificates it accepts. A
/// caller adds its own timeouts and redirect rule, nothing else.
pub(crate) fn agent_builder() -> ureq::AgentBuilder {
    // Built once per process: on Linux it reads the system's CA files.
    static CONFIG: OnceLock<Result<Arc<rustls::ClientConfig>, rustls::Error>> = OnceLock::new();
    builder_with(CONFIG.get_or_init(|| trust_config(Vec::new())).clone())
}

/// Trust settings that could not be built mean a system with no CA at all (a
/// slim Linux container). The builder then keeps ureq's built-in list of
/// public CAs, as before this module existed. It says nothing: a builder
/// carries its trust settings on http connections too, so a failure here must
/// never reach a request.
fn builder_with(config: Result<Arc<rustls::ClientConfig>, rustls::Error>) -> ureq::AgentBuilder {
    match config {
        Ok(config) => ureq::AgentBuilder::new().tls_config(config),
        Err(_) => ureq::AgentBuilder::new(),
    }
}

/// TLS settings whose certificate check is the operating system's, plus
/// `extra_roots`. Only tests pass extra roots — the real trust store cannot be
/// changed from a test.
fn trust_config(
    extra_roots: Vec<CertificateDer<'static>>,
) -> Result<Arc<rustls::ClientConfig>, rustls::Error> {
    // The provider is named here, like ureq does, so that no other user of
    // rustls in the same process can change it.
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    // With no extra roots, `new` is the constructor to use. On Windows the
    // other one builds a second chain engine for the extra roots — one more
    // thing that can fail — and tries every refused chain against it again.
    let verifier = if extra_roots.is_empty() {
        rustls_platform_verifier::Verifier::new(provider.clone())?
    } else {
        rustls_platform_verifier::Verifier::new_with_extra_roots(extra_roots, provider.clone())?
    };
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()?
        // rustls has no other way to install a verifier of our choice. This
        // one is the operating system's own check.
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(verifier))
        .with_no_client_auth();
    Ok(Arc::new(config))
}

/// Translate the connection-layer error of a request. A certificate the trust
/// store refused gets its own line, because the fix is not the URL. Every
/// other failure (refused, timeout, DNS) keeps the line of
/// [`crate::translate_transport`] — which no request path calls directly: it
/// cannot see the error, so it cannot name a certificate.
pub(crate) fn translate_transport_error(error: &ureq::Transport) -> RemoteError {
    match certificate_rejection(error, "the server's") {
        Some(rejection) => RemoteError {
            message: format!("server unreachable — {rejection}"),
            reason: None,
            status: None,
            evidence: None,
        },
        None => crate::translate_transport(),
    }
}

/// When the certificate check is what broke the connection: the sentence that
/// names whose certificate it was (`owner`, for example `the server's`), the
/// reason and the fix.
pub(crate) fn certificate_rejection(error: &ureq::Transport, owner: &str) -> Option<String> {
    let mut cause = std::error::Error::source(error);
    while let Some(error) = cause {
        if let Some(rustls::Error::InvalidCertificate(reason)) = error.downcast_ref() {
            return Some(format!(
                "{owner} TLS certificate was rejected ({}); if your organisation's own CA issued it, add that CA to the operating system's trust store",
                reason_name(reason)
            ));
        }
        // An `io::Error` hands out the error it wraps through `get_ref`, not
        // through `source`.
        cause = match error.downcast_ref::<std::io::Error>().and_then(|io| io.get_ref()) {
            Some(wrapped) => Some(wrapped),
            None => error.source(),
        };
    }
    None
}

/// The rustls name of a reason, and only the name. Some reasons carry a host
/// name, a time, or text the operating system took from the other side's
/// certificate; none of it reaches the message. The system verifier decides
/// the name, so one situation can read differently per platform, and a reason
/// rustls has no name for reads `Other`. rustls calls the variant that
/// carries data `…Context`; the name drops that ending.
fn reason_name(reason: &rustls::CertificateError) -> String {
    let debug = format!("{reason:?}");
    let name: String = debug.chars().take_while(char::is_ascii_alphanumeric).collect();
    name.trim_end_matches("Context").to_string()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::io::{Read, Write};

    use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};

    /// A test CA, and a server certificate it issued for `127.0.0.1` with its
    /// key.
    pub(crate) struct TestPki {
        ca: CertificateDer<'static>,
        leaf: CertificateDer<'static>,
        key: PrivateKeyDer<'static>,
    }

    /// The macOS verifier refuses a server certificate without the serverAuth
    /// usage, without a SAN, or valid for more than 825 days, so each of the
    /// three is set here (rcgen's default validity is about 2,000 years).
    pub(crate) fn test_pki() -> TestPki {
        use chrono::Datelike;
        let day = |offset: i64| {
            let date = (chrono::Utc::now() + chrono::Duration::days(offset)).date_naive();
            rcgen::date_time_ymd(date.year(), date.month() as u8, date.day() as u8)
        };

        let ca_key = rcgen::KeyPair::generate().unwrap();
        let mut ca = rcgen::CertificateParams::new(Vec::<String>::new()).unwrap();
        ca.distinguished_name.push(rcgen::DnType::CommonName, "speclink test CA");
        ca.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        ca.key_usages = vec![rcgen::KeyUsagePurpose::KeyCertSign, rcgen::KeyUsagePurpose::CrlSign];
        ca.not_before = day(-1);
        ca.not_after = day(30);
        let ca_cert = ca.self_signed(&ca_key).unwrap();

        let key = rcgen::KeyPair::generate().unwrap();
        let mut leaf = rcgen::CertificateParams::new(vec!["127.0.0.1".to_string()]).unwrap();
        leaf.distinguished_name.push(rcgen::DnType::CommonName, "127.0.0.1");
        leaf.key_usages = vec![rcgen::KeyUsagePurpose::DigitalSignature];
        leaf.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];
        leaf.use_authority_key_identifier_extension = true;
        leaf.not_before = day(-1);
        leaf.not_after = day(30);
        let leaf_cert = leaf.signed_by(&key, &rcgen::Issuer::from_params(&ca, &ca_key)).unwrap();

        TestPki {
            ca: ca_cert.der().clone(),
            leaf: leaf_cert.der().clone(),
            key: PrivatePkcs8KeyDer::from(key.serialize_der()).into(),
        }
    }

    const BODY: &str = r#"{"ok":true}"#;

    /// Read one request head and answer [`BODY`].
    fn answer(stream: &mut (impl Read + Write)) -> std::io::Result<()> {
        let mut head = Vec::new();
        let mut buf = [0u8; 1024];
        while !head.windows(4).any(|w| w == b"\r\n\r\n") {
            let n = stream.read(&mut buf)?;
            if n == 0 {
                return Ok(());
            }
            head.extend_from_slice(&buf[..n]);
        }
        write!(
            stream,
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{BODY}",
            BODY.len()
        )?;
        stream.flush()
    }

    /// An HTTPS server on a random local port whose chain makes a system
    /// that does not trust the test CA name an unknown issuer. Which chain
    /// that is differs: Windows names an unknown issuer only when it can
    /// build the chain up to the root, so there the root travels with the
    /// leaf. macOS names it only when the root is absent.
    pub(crate) fn https_server(pki: &TestPki) -> String {
        https_server_sending(pki, cfg!(windows))
    }

    /// An HTTPS server on a random local port that presents `pki`'s leaf,
    /// with or without the root behind it.
    fn https_server_sending(pki: &TestPki, root_too: bool) -> String {
        let mut chain = vec![pki.leaf.clone()];
        if root_too {
            chain.push(pki.ca.clone());
        }
        let config = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(chain, pki.key.clone_key())
        .unwrap();
        let config = Arc::new(config);
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for tcp in listener.incoming().flatten() {
                let conn = rustls::ServerConnection::new(config.clone()).unwrap();
                // A client that rejects the certificate ends the handshake,
                // and the error lands here.
                let _ = answer(&mut rustls::StreamOwned::new(conn, tcp));
            }
        });
        format!("https://127.0.0.1:{port}")
    }

    /// An HTTP server (no TLS) on a random local port.
    fn http_server() -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for mut tcp in listener.incoming().flatten() {
                let _ = answer(&mut tcp);
            }
        });
        format!("http://127.0.0.1:{port}")
    }

    fn get(builder: ureq::AgentBuilder, url: &str) -> Result<String, ureq::Error> {
        Ok(builder.build().get(url).call()?.into_string().unwrap())
    }

    #[test]
    fn a_ca_in_the_trust_roots_lets_the_request_through() {
        let pki = test_pki();
        let builder = builder_with(trust_config(vec![pki.ca.clone()]));
        assert_eq!(get(builder, &https_server(&pki)).unwrap(), BODY);
    }

    #[test]
    fn a_ca_outside_the_trust_roots_fails_the_request() {
        let builder = builder_with(trust_config(Vec::new()));
        assert!(get(builder, &https_server(&test_pki())).is_err());
    }

    /// The connection-layer error of a request that must fail.
    fn transport_error(builder: ureq::AgentBuilder, url: &str) -> ureq::Transport {
        match get(builder, url) {
            Err(ureq::Error::Transport(error)) => error,
            other => panic!("expected a connection-layer failure, got {other:?}"),
        }
    }

    /// A port with no process listening on it. Nothing holds the port after
    /// this returns, so a test that runs at the same time can, rarely, bind
    /// it: a caller gives its request a timeout.
    pub(crate) fn closed_port() -> u16 {
        std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
    }

    #[test]
    fn a_rejected_certificate_names_the_certificate_and_the_fix() {
        let error = transport_error(builder_with(trust_config(Vec::new())), &https_server(&test_pki()));
        let translated = translate_transport_error(&error);
        assert_eq!(
            translated.message,
            "server unreachable — the server's TLS certificate was rejected (UnknownIssuer); if your organisation's own CA issued it, add that CA to the operating system's trust store"
        );
        assert!(translated.status.is_none() && translated.reason.is_none() && translated.evidence.is_none());
    }

    /// The spec's example row for macOS: a root that travels with the leaf
    /// and is not trusted is a reason rustls has no name for.
    #[cfg(target_os = "macos")]
    #[test]
    fn macos_reads_other_when_the_untrusted_root_travels_with_the_leaf() {
        let url = https_server_sending(&test_pki(), true);
        let error = transport_error(builder_with(trust_config(Vec::new())), &url);
        assert_eq!(
            translate_transport_error(&error).message,
            "server unreachable — the server's TLS certificate was rejected (Other); if your organisation's own CA issued it, add that CA to the operating system's trust store"
        );
    }

    /// The text some reasons carry comes from the other side's certificate.
    #[test]
    fn a_reason_is_named_without_the_data_it_carries() {
        use rustls::CertificateError;
        let from_the_system = std::io::Error::other("“evil\u{1b}[31m” certificate is not trusted: -67843");
        let other = CertificateError::Other(rustls::OtherError(Arc::new(from_the_system)));
        assert_eq!(reason_name(&other), "Other");
        let wrong_host = CertificateError::NotValidForNameContext {
            expected: "speclink.example.com".try_into().unwrap(),
            presented: vec!["evil.example.com".to_string()],
        };
        assert_eq!(reason_name(&wrong_host), "NotValidForName");
        assert_eq!(reason_name(&CertificateError::UnknownIssuer), "UnknownIssuer");
    }

    #[test]
    fn a_refused_connection_keeps_the_existing_message() {
        let url = format!("https://127.0.0.1:{}", closed_port());
        let builder = agent_builder().timeout(std::time::Duration::from_secs(5));
        let translated = translate_transport_error(&transport_error(builder, &url));
        assert_eq!(translated.message, crate::translate_transport().message);
        assert!(!translated.message.contains("certificate"));
    }

    /// D3: a system with no CA at all. The builder then carries no settings
    /// of ours — it does not trust the test CA — and http still works.
    #[test]
    fn failed_trust_settings_fall_back_to_the_default_builder() {
        let no_ca = || Err(rustls::Error::General("No CA certificates were loaded from the system".into()));
        assert!(get(builder_with(no_ca()), &https_server(&test_pki())).is_err());
        assert_eq!(get(builder_with(no_ca()), &http_server()).unwrap(), BODY);
    }

    /// Every agent goes through this module, so a new connection cannot miss
    /// the shared trust settings.
    ///
    /// The same question for the whole workspace was answered by search, not
    /// by this test:
    ///
    /// ```text
    /// grep -rn --include=Cargo.toml -E "ureq|reqwest" crates apps/desktop
    /// grep -rnE "ureq::|reqwest::|use ureq|use reqwest" --include='*.rs' crates apps/desktop
    /// ```
    ///
    /// Result (2026-11-18): only `speclink-remote` depends on ureq in its
    /// `[dependencies]`. `speclink-server` names it under `[dev-dependencies]`
    /// and uses it in `tests/` only. No crate names reqwest; the desktop app
    /// gets it through the updater plugin, which talks to the release feed,
    /// not to a Speclink server.
    #[test]
    fn only_this_module_builds_an_agent() {
        // The builder, and the ureq shortcuts that make an agent with ureq's
        // own trust settings.
        const AGENT_MAKERS: [&str; 12] = [
            "AgentBuilder::new",
            "Agent::new(",
            "ureq::builder(",
            "ureq::agent(",
            "ureq::request(",
            "ureq::request_url(",
            "ureq::get(",
            "ureq::head(",
            "ureq::patch(",
            "ureq::post(",
            "ureq::put(",
            "ureq::delete(",
        ];
        let offenders = other_files_containing(&AGENT_MAKERS, &["http.rs"]);
        assert!(offenders.is_empty(), "these files build their own agent: {offenders:?}");
    }

    /// `translate_transport()` cannot see the error, so a request path that
    /// calls it loses the certificate's line without any test going red.
    /// `lib.rs` defines it; this module is its one caller.
    #[test]
    fn only_this_module_translates_without_the_error() {
        let offenders = other_files_containing(&["translate_transport()"], &["http.rs", "lib.rs"]);
        assert!(offenders.is_empty(), "these files call translate_transport() themselves: {offenders:?}");
    }

    /// The files of this crate's `src`, `skipped` left out, that hold one of
    /// `needles`.
    fn other_files_containing(needles: &[&str], skipped: &[&str]) -> Vec<String> {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut found: Vec<String> = std::fs::read_dir(&src)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| !skipped.contains(&name.as_str()))
            .filter(|name| {
                let text = std::fs::read_to_string(src.join(name)).unwrap();
                needles.iter().any(|needle| text.contains(needle))
            })
            .collect();
        found.sort();
        found
    }
}
