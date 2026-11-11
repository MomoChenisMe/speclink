//! The client half of a browser login (client-browser-login D3): the login
//! service's metadata document (RFC 8414, with the OpenID Connect location as
//! the fallback), PKCE S256 (RFC 7636), the authorization URL and the token
//! requests. Speclink implements no server side of any of this.
//!
//! Every URL the login uses must be https, except on this machine (127.0.0.1,
//! ::1, localhost) for development. The metadata's `issuer` must equal the
//! declared issuer exactly before any endpoint in it is used.

use base64::Engine as _;
use serde::Deserialize;
use sha2::Digest as _;

/// The lifetime given to an access token whose response states none.
pub(crate) const DEFAULT_ACCESS_TOKEN_SECS: u64 = 300;

/// The endpoints a browser login uses, read from a checked metadata document.
#[derive(Debug, Clone)]
pub(crate) struct Endpoints {
    pub authorization: String,
    pub token: String,
    pub revocation: Option<String>,
}

/// The fields of a metadata document this client reads.
#[derive(Debug, Deserialize)]
struct Metadata {
    issuer: Option<String>,
    authorization_endpoint: Option<String>,
    token_endpoint: Option<String>,
    revocation_endpoint: Option<String>,
    code_challenge_methods_supported: Option<Vec<String>>,
}

/// The host of `url`, when the URL has a plain `scheme://host[:port]` form.
/// A userinfo part (`user@host`) is refused outright: it is the classic way to
/// make a remote host look like a local one.
fn scheme_and_host(url: &str) -> Option<(String, String)> {
    let (scheme, rest) = url.split_once("://")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    if authority.is_empty() || authority.contains('@') {
        return None;
    }
    let host = match authority.strip_prefix('[') {
        // `[v6]` may be followed by `:port` and nothing else.
        Some(v6) => {
            let (host, rest) = v6.split_once(']')?;
            if !(rest.is_empty() || rest.starts_with(':')) {
                return None;
            }
            host
        }
        None => authority.split(':').next()?,
    };
    Some((scheme.to_ascii_lowercase(), host.to_ascii_lowercase()))
}

/// https everywhere; plain http only for a login service on this machine.
pub(crate) fn require_secure(url: &str, what: &str) -> Result<(), String> {
    match scheme_and_host(url) {
        Some((scheme, _)) if scheme == "https" => Ok(()),
        Some((scheme, host))
            if scheme == "http" && matches!(host.as_str(), "127.0.0.1" | "::1" | "localhost") =>
        {
            Ok(())
        }
        Some(_) => Err(format!("the login service's {what} must use https: {url}")),
        None => Err(format!("the login service's {what} is not a valid URL: {url}")),
    }
}

/// The RFC 8414 location (the well-known segment goes between host and path)
/// and the OpenID Connect one (appended to the issuer).
fn metadata_urls(issuer: &str) -> (String, String) {
    let trimmed = issuer.trim_end_matches('/');
    let (scheme, rest) = trimmed.split_once("://").unwrap_or(("https", trimmed));
    let (authority, path) = match rest.find('/') {
        Some(i) => rest.split_at(i),
        None => (rest, ""),
    };
    (
        format!("{scheme}://{authority}/.well-known/oauth-authorization-server{path}"),
        format!("{trimmed}/.well-known/openid-configuration"),
    )
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(30))
        .redirects(0)
        .build()
}

/// One metadata GET: `Ok(None)` for a 404, so the caller can try the other
/// location.
fn fetch(url: &str) -> Result<Option<Metadata>, String> {
    match agent().get(url).call() {
        Ok(resp) => resp
            .into_json()
            .map(Some)
            .map_err(|_| format!("the login service's metadata at {url} is not valid JSON")),
        Err(ureq::Error::Status(404, _)) => Ok(None),
        Err(ureq::Error::Status(status, _)) => Err(format!(
            "could not read the login service's metadata at {url} (HTTP {status})"
        )),
        Err(ureq::Error::Transport(_)) => {
            Err(format!("could not reach the login service at {url}"))
        }
    }
}

/// Read and check the login service's metadata document. Every check runs
/// before an endpoint from the document is used.
pub(crate) fn discover_endpoints(issuer: &str) -> Result<Endpoints, String> {
    require_secure(issuer, "issuer")?;
    let (rfc8414, oidc) = metadata_urls(issuer);
    let metadata = match fetch(&rfc8414)? {
        Some(metadata) => metadata,
        None => fetch(&oidc)?.ok_or_else(|| {
            format!("the login service publishes no metadata document at {rfc8414} or {oidc}")
        })?,
    };

    let claimed = metadata.issuer.unwrap_or_default();
    if claimed != issuer {
        return Err(format!(
            "the login service's metadata names issuer \"{claimed}\", not the declared \"{issuer}\""
        ));
    }
    let authorization = metadata
        .authorization_endpoint
        .ok_or("the login service's metadata has no authorization_endpoint")?;
    let token = metadata
        .token_endpoint
        .ok_or("the login service's metadata has no token_endpoint")?;
    require_secure(&authorization, "authorization_endpoint")?;
    require_secure(&token, "token_endpoint")?;
    if let Some(revocation) = &metadata.revocation_endpoint {
        require_secure(revocation, "revocation_endpoint")?;
    }
    // Absent is common and still gets S256; a list without it is a refusal.
    if let Some(methods) = &metadata.code_challenge_methods_supported {
        if !methods.iter().any(|m| m == "S256") {
            return Err(format!(
                "the login service does not support PKCE S256 (it lists: {})",
                methods.join(", ")
            ));
        }
    }
    Ok(Endpoints {
        authorization,
        token,
        revocation: metadata.revocation_endpoint,
    })
}

/// 32 bytes from the operating system's random source, base64url-encoded (43
/// characters): a PKCE verifier or a state.
pub(crate) fn random_token() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| format!("no secure random source available: {e}"))?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}

/// The S256 challenge for `verifier`.
pub(crate) fn challenge_of(verifier: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sha2::Sha256::digest(verifier.as_bytes()))
}

/// Percent-encode everything but RFC 3986's unreserved characters.
fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// The URL that starts the login in the browser.
pub(crate) fn authorization_url(
    endpoint: &str,
    client_id: &str,
    redirect_uri: &str,
    challenge: &str,
    state: &str,
    scopes: &[String],
) -> String {
    let scope = scopes.join(" ");
    let mut params = vec![
        ("response_type", "code"),
        ("client_id", client_id),
        ("redirect_uri", redirect_uri),
        ("code_challenge", challenge),
        ("code_challenge_method", "S256"),
        ("state", state),
    ];
    if !scope.is_empty() {
        params.push(("scope", &scope));
    }
    let query: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
        .collect();
    let sep = if endpoint.contains('?') { '&' } else { '?' };
    format!("{endpoint}{sep}{}", query.join("&"))
}

/// A token endpoint's answer.
#[derive(Deserialize)]
pub(crate) struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<u64>,
}

/// Printed without its tokens, so a `{:?}` in a log line cannot leak them.
impl std::fmt::Debug for TokenResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenResponse")
            .field("refresh_token", &self.refresh_token.as_ref().map(|_| "<redacted>"))
            .field("expires_in", &self.expires_in)
            .finish_non_exhaustive()
    }
}

/// Why a token request failed. Only `Rejected` says the credential is dead;
/// anything else leaves its fate unknown.
#[derive(Debug)]
pub(crate) enum GrantFailure {
    Rejected(String),
    Unavailable(String),
}

impl GrantFailure {
    pub fn message(self) -> String {
        match self {
            GrantFailure::Rejected(m) | GrantFailure::Unavailable(m) => m,
        }
    }
}

#[derive(Deserialize)]
struct ErrorBody {
    #[serde(default)]
    error: String,
}

/// POST a token request. A 400 `invalid_grant` or a 401 is the login service
/// refusing the grant; every other failure keeps the credential's fate open.
fn token_request(endpoint: &str, what: &str, form: &[(&str, &str)]) -> Result<TokenResponse, GrantFailure> {
    match agent().post(endpoint).send_form(form) {
        Ok(resp) => resp.into_json().map_err(|_| {
            GrantFailure::Unavailable(format!("the login service's {what} answer is not a token response"))
        }),
        Err(ureq::Error::Status(status, resp)) => {
            let error = resp
                .into_json::<ErrorBody>()
                .map(|b| b.error)
                .unwrap_or_default()
                .chars()
                .filter(|c| c.is_ascii_graphic())
                .take(64)
                .collect::<String>();
            let line = format!("the login service refused the {what} (HTTP {status} {error})");
            if status == 401 || (status == 400 && error == "invalid_grant") {
                Err(GrantFailure::Rejected(line))
            } else {
                Err(GrantFailure::Unavailable(line))
            }
        }
        Err(ureq::Error::Transport(_)) => {
            Err(GrantFailure::Unavailable("could not reach the login service".to_string()))
        }
    }
}

/// Trade the authorization code for tokens. A public client: no secret.
pub(crate) fn exchange_code(
    token_endpoint: &str,
    client_id: &str,
    code: &str,
    redirect_uri: &str,
    verifier: &str,
) -> Result<TokenResponse, GrantFailure> {
    token_request(
        token_endpoint,
        "sign-in",
        &[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("client_id", client_id),
            ("code_verifier", verifier),
        ],
    )
}

/// Spend a refresh token for a fresh access token (and, when the login service
/// rotates, a fresh refresh token).
pub(crate) fn refresh_grant(
    token_endpoint: &str,
    client_id: &str,
    refresh_token: &str,
) -> Result<TokenResponse, GrantFailure> {
    token_request(
        token_endpoint,
        "token refresh",
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", client_id),
        ],
    )
}

/// Revoke a refresh token at the login service (RFC 7009). Best effort: the
/// caller clears the local credential whatever this returns.
pub(crate) fn revoke(revocation_endpoint: &str, client_id: &str, refresh_token: &str) -> Result<(), String> {
    match agent().post(revocation_endpoint).send_form(&[
        ("token", refresh_token),
        ("token_type_hint", "refresh_token"),
        ("client_id", client_id),
    ]) {
        Ok(_) => Ok(()),
        Err(ureq::Error::Status(status, _)) => Err(format!("HTTP {status}")),
        Err(ureq::Error::Transport(_)) => Err("could not reach the login service".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc8414_inserts_the_well_known_segment_between_host_and_path() {
        let (rfc, oidc) = metadata_urls("https://id.example.com/realms/dev");
        assert_eq!(rfc, "https://id.example.com/.well-known/oauth-authorization-server/realms/dev");
        assert_eq!(oidc, "https://id.example.com/realms/dev/.well-known/openid-configuration");
        let (rfc, _) = metadata_urls("https://id.example.com");
        assert_eq!(rfc, "https://id.example.com/.well-known/oauth-authorization-server");
    }

    #[test]
    fn only_this_machine_may_skip_https() {
        for ok in ["https://id.example.com", "http://127.0.0.1:9000/x", "http://[::1]:9000", "http://LOCALHOST"] {
            assert!(require_secure(ok, "issuer").is_ok(), "{ok}");
        }
        for bad in [
            "http://id.example.com",
            "http://127.0.0.1@id.example.com",
            "http://localhost.id.example.com",
            "http://127.0.0.1.id.example.com",
            "http://[::1].id.example.com",
            "http://[::1]x:9000",
            "ftp://127.0.0.1",
            "not a url",
        ] {
            assert!(require_secure(bad, "issuer").is_err(), "{bad}");
        }
    }

    /// RFC 7636 appendix B's test vector.
    #[test]
    fn the_s256_challenge_matches_the_rfc_example() {
        assert_eq!(
            challenge_of("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn random_tokens_are_43_characters_and_differ() {
        let a = random_token().unwrap();
        let b = random_token().unwrap();
        assert_eq!(a.len(), 43);
        assert_ne!(a, b);
    }

    /// A token response printed with `{:?}` must not carry its tokens.
    #[test]
    fn a_token_response_debug_print_hides_the_tokens() {
        let resp = TokenResponse {
            access_token: "at-secret".into(),
            refresh_token: Some("rt-secret".into()),
            expires_in: Some(60),
        };
        let printed = format!("{resp:?}");
        assert!(!printed.contains("at-secret") && !printed.contains("rt-secret"), "{printed}");
    }

    #[test]
    fn the_authorization_url_encodes_its_parameters() {
        let url = authorization_url(
            "https://id.example.com/authorize?tenant=a",
            "speclink",
            "http://127.0.0.1:5000/callback",
            "chal",
            "st",
            &["openid".into(), "offline_access".into()],
        );
        assert_eq!(
            url,
            "https://id.example.com/authorize?tenant=a&response_type=code&client_id=speclink&redirect_uri=http%3A%2F%2F127.0.0.1%3A5000%2Fcallback&code_challenge=chal&code_challenge_method=S256&state=st&scope=openid%20offline_access"
        );
    }
}
