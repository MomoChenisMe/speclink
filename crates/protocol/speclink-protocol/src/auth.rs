//! Login-method discovery (client-browser-login D1): what a server declares at
//! `GET /api/speclink/v1/auth-methods`, asked before any credential exists.
//!
//! The request carries no credential and no version header. A server without
//! the route (404/405) has declared nothing; clients keep their pre-discovery
//! behavior for it. `methods` stays a list of strings on the wire so a method a
//! client does not know is skipped by that client instead of failing the parse.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// `GET /api/speclink/v1/auth-methods` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthMethodsResponse {
    /// The login methods the server supports: `browser`, `device`, `token`.
    pub methods: Vec<String>,
    /// The login service behind `browser`; meaningful only when `methods`
    /// lists `browser`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser: Option<BrowserLogin>,
}

/// The login service a browser login goes to. Only the issuer travels: the
/// endpoints come from the service's own metadata document, so the two never
/// drift apart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BrowserLogin {
    /// The login service's issuer URL.
    pub issuer: String,
    /// The client identifier the host registered for speclink at its login
    /// service.
    pub client_id: String,
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use crate::auth::*;

    #[test]
    fn a_full_declaration_parses() {
        let resp: AuthMethodsResponse = serde_json::from_str(
            r#"{"methods":["browser","device"],"browser":{"issuer":"https://id.example.com","clientId":"speclink","scopes":["openid"]}}"#,
        )
        .unwrap();
        assert_eq!(resp.methods, vec!["browser".to_string(), "device".to_string()]);
        let browser = resp.browser.expect("browser settings present");
        assert_eq!(browser.issuer, "https://id.example.com");
        assert_eq!(browser.client_id, "speclink");
        assert_eq!(browser.scopes, vec!["openid".to_string()]);
    }

    #[test]
    fn absent_scopes_read_as_empty() {
        let resp: AuthMethodsResponse = serde_json::from_str(
            r#"{"methods":["browser"],"browser":{"issuer":"https://id.example.com","clientId":"speclink"}}"#,
        )
        .unwrap();
        assert!(resp.browser.expect("browser settings present").scopes.is_empty());
    }

    /// A method this client does not know must not break parsing: servers add
    /// methods, old clients skip them.
    #[test]
    fn an_unknown_method_is_not_a_parse_error() {
        let resp: AuthMethodsResponse =
            serde_json::from_str(r#"{"methods":["passkey","token"]}"#).unwrap();
        assert_eq!(resp.methods, vec!["passkey".to_string(), "token".to_string()]);
        assert!(resp.browser.is_none(), "an absent browser object reads as none");
    }

    #[test]
    fn serializes_camel_case_keys_with_their_types() {
        let resp = AuthMethodsResponse {
            methods: vec!["browser".into(), "token".into()],
            browser: Some(BrowserLogin {
                issuer: "https://id.example.com".into(),
                client_id: "speclink".into(),
                scopes: vec!["openid".into()],
            }),
        };
        let json = serde_json::to_value(&resp).unwrap();
        let mut top: Vec<&str> = json.as_object().unwrap().keys().map(String::as_str).collect();
        top.sort_unstable();
        assert_eq!(top, vec!["browser", "methods"], "top-level keys: {json}");
        assert!(json["methods"].is_array());
        let browser = json["browser"].as_object().expect("browser is an object");
        let mut keys: Vec<&str> = browser.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, vec!["clientId", "issuer", "scopes"], "browser keys: {json}");
        assert!(json["browser"]["issuer"].is_string());
        assert!(json["browser"]["clientId"].is_string());
        assert!(json["browser"]["scopes"].is_array());

        let back: AuthMethodsResponse = serde_json::from_value(json).unwrap();
        assert_eq!(back, resp);
    }

    #[test]
    fn auth_method_dtos_export_json_schema() {
        let text = serde_json::to_string(&schemars::schema_for!(AuthMethodsResponse)).unwrap();
        assert!(text.contains("properties"), "schema has properties: {text}");
        assert!(
            text.contains("methods") && text.contains("clientId") && text.contains("scopes"),
            "schema fields are camelCase and include the nested browser object: {text}"
        );
    }
}
