//! `anyhow::Error` → `(ExitCode, ErrorCode)` 的決定性分類。
//!
//! 對應 spec `cli-machine-interface` 的 Stable exit-code table 與 Failure mapping。

use provider::config::ConfigError;
use provider::error::{ProviderError, ResolutionError};
use provider_local::error::LocalProviderError;
use runtime::propose::RuntimeError;

/// Process exit code newtype。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExitCode(pub u8);

impl From<u8> for ExitCode {
    fn from(v: u8) -> Self {
        Self(v)
    }
}

impl ExitCode {
    /// 取得 `u8` 值。
    pub fn as_u8(self) -> u8 {
        self.0
    }
}

/// 點分隔 error code newtype。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ErrorCode(pub &'static str);

impl ErrorCode {
    /// 取得內部 `&'static str`。
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

/// 沿 anyhow 因果鏈尋找已知 domain error 型別，並映射到 `(ExitCode, ErrorCode)`。
///
/// 沒有命中時回退為 `(1, "internal.error")`。
pub fn classify(err: &anyhow::Error) -> (ExitCode, ErrorCode) {
    for cause in err.chain() {
        if let Some(p) = cause.downcast_ref::<ProviderError>() {
            return classify_provider(p);
        }
        if let Some(r) = cause.downcast_ref::<RuntimeError>() {
            return classify_runtime(r);
        }
        if let Some(l) = cause.downcast_ref::<LocalProviderError>() {
            return classify_local(l);
        }
        if let Some(res) = cause.downcast_ref::<ResolutionError>() {
            return classify_resolution(res);
        }
        if cause.downcast_ref::<ConfigError>().is_some() {
            return (ExitCode(2), ErrorCode("input.invalid"));
        }
    }
    (ExitCode(1), ErrorCode("internal.error"))
}

fn classify_provider(p: &ProviderError) -> (ExitCode, ErrorCode) {
    match p {
        ProviderError::NotAuthenticated { .. } => {
            (ExitCode(6), ErrorCode("provider.not_authenticated"))
        }
        ProviderError::Unavailable { .. } => (ExitCode(5), ErrorCode("provider.unavailable")),
        ProviderError::ChangeAlreadyExists { .. } => {
            (ExitCode(1), ErrorCode("change.already_exists"))
        }
        ProviderError::ChangeNotFound { .. } => (ExitCode(1), ErrorCode("change.not_found")),
        ProviderError::InvalidChangeId { .. } => (ExitCode(2), ErrorCode("change.invalid_id")),
        ProviderError::ArtifactAlreadyExists { .. } => {
            (ExitCode(1), ErrorCode("artifact.already_exists"))
        }
        ProviderError::MissingCapability => (ExitCode(2), ErrorCode("artifact.missing_capability")),
        ProviderError::InvalidCapability { .. } => {
            (ExitCode(2), ErrorCode("artifact.invalid_capability"))
        }
        ProviderError::ChangeNotArchivable { .. } => {
            (ExitCode(1), ErrorCode("archive.change_not_archivable"))
        }
        ProviderError::SpecDeltaConflict { .. } => (ExitCode(7), ErrorCode("spec.delta_conflict")),
        ProviderError::SpecDeltaParseError { .. } => {
            (ExitCode(2), ErrorCode("spec.delta_parse_error"))
        }
        ProviderError::Internal { .. } => (ExitCode(1), ErrorCode("internal.error")),
    }
}

fn classify_runtime(r: &RuntimeError) -> (ExitCode, ErrorCode) {
    match r {
        RuntimeError::Provider(p) => classify_provider(p),
        RuntimeError::InvalidInput { .. } => (ExitCode(2), ErrorCode("input.invalid")),
    }
}

fn classify_local(l: &LocalProviderError) -> (ExitCode, ErrorCode) {
    match l {
        LocalProviderError::InvalidChangeId { .. } => (ExitCode(2), ErrorCode("change.invalid_id")),
        LocalProviderError::ChangeAlreadyExists { .. } => {
            (ExitCode(1), ErrorCode("change.already_exists"))
        }
        LocalProviderError::ChangeNotFound { .. } => (ExitCode(1), ErrorCode("change.not_found")),
        LocalProviderError::ArtifactAlreadyExists { .. } => {
            (ExitCode(1), ErrorCode("artifact.already_exists"))
        }
        LocalProviderError::MissingCapability => {
            (ExitCode(2), ErrorCode("artifact.missing_capability"))
        }
        LocalProviderError::InvalidCapability { .. } => {
            (ExitCode(2), ErrorCode("artifact.invalid_capability"))
        }
        LocalProviderError::ChangeNotArchivable { .. } => {
            (ExitCode(1), ErrorCode("archive.change_not_archivable"))
        }
        LocalProviderError::SpecDeltaConflict { .. } => {
            (ExitCode(7), ErrorCode("spec.delta_conflict"))
        }
        LocalProviderError::SpecDeltaParseError { .. } => {
            (ExitCode(2), ErrorCode("spec.delta_parse_error"))
        }
        LocalProviderError::Io(_)
        | LocalProviderError::Json(_)
        | LocalProviderError::StateDb(_)
        | LocalProviderError::RollbackFailed { .. }
        | LocalProviderError::Internal { .. } => (ExitCode(1), ErrorCode("internal.error")),
    }
}

fn classify_resolution(r: &ResolutionError) -> (ExitCode, ErrorCode) {
    match r {
        ResolutionError::AuthRequiredNoFallback { .. } => {
            (ExitCode(6), ErrorCode("provider.not_authenticated"))
        }
        ResolutionError::InvalidConfig { .. } => (ExitCode(2), ErrorCode("input.invalid")),
    }
}

#[cfg(test)]
mod tests {
    use crate::exit_code::{ExitCode, classify};
    use provider::error::ProviderError;
    use provider::model::ChangeId;
    use provider_local::error::LocalProviderError;
    use runtime::propose::RuntimeError;

    #[test]
    fn provider_not_authenticated_maps_to_6() {
        let err = anyhow::Error::from(ProviderError::NotAuthenticated {
            provider_name: "acme".to_string(),
        });
        let (code, ec) = classify(&err);
        assert_eq!(code, ExitCode::from(6));
        assert_eq!(ec.as_str(), "provider.not_authenticated");
    }

    #[test]
    fn provider_unavailable_maps_to_5() {
        let err = anyhow::Error::from(ProviderError::Unavailable {
            provider_name: "acme".to_string(),
        });
        let (code, ec) = classify(&err);
        assert_eq!(code, ExitCode::from(5));
        assert_eq!(ec.as_str(), "provider.unavailable");
    }

    #[test]
    fn change_already_exists_maps_to_1() {
        let err = anyhow::Error::from(ProviderError::ChangeAlreadyExists {
            change_id: ChangeId::from("demo"),
        });
        let (code, ec) = classify(&err);
        assert_eq!(code, ExitCode::from(1));
        assert_eq!(ec.as_str(), "change.already_exists");
    }

    #[test]
    fn provider_invalid_change_id_maps_to_2() {
        let err = anyhow::Error::from(ProviderError::InvalidChangeId {
            change_id: "Add-Feature".to_string(),
        });
        let (code, ec) = classify(&err);
        assert_eq!(code, ExitCode::from(2));
        assert_eq!(ec.as_str(), "change.invalid_id");
    }

    #[test]
    fn runtime_invalid_input_maps_to_2() {
        let err = anyhow::Error::from(RuntimeError::InvalidInput {
            reason: "summary empty".to_string(),
        });
        let (code, ec) = classify(&err);
        assert_eq!(code, ExitCode::from(2));
        assert_eq!(ec.as_str(), "input.invalid");
    }

    #[test]
    fn local_provider_invalid_change_id_maps_to_2() {
        let err = anyhow::Error::from(LocalProviderError::InvalidChangeId {
            change_id: "1bad".to_string(),
        });
        let (code, ec) = classify(&err);
        assert_eq!(code, ExitCode::from(2));
        assert_eq!(ec.as_str(), "change.invalid_id");
    }

    #[test]
    fn provider_change_not_archivable_maps_to_1() {
        let err = anyhow::Error::from(ProviderError::ChangeNotArchivable {
            reason: "already archived".to_string(),
        });
        let (code, ec) = classify(&err);
        assert_eq!(code, ExitCode::from(1));
        assert_eq!(ec.as_str(), "archive.change_not_archivable");
    }

    #[test]
    fn provider_spec_delta_conflict_maps_to_7() {
        let err = anyhow::Error::from(ProviderError::SpecDeltaConflict {
            capability: "auth".to_string(),
            requirement: "User login".to_string(),
            operation: "ADDED",
        });
        let (code, ec) = classify(&err);
        assert_eq!(code, ExitCode::from(7));
        assert_eq!(ec.as_str(), "spec.delta_conflict");
    }

    #[test]
    fn provider_spec_delta_parse_error_maps_to_2() {
        let err = anyhow::Error::from(ProviderError::SpecDeltaParseError {
            capability: "auth".to_string(),
            message: "unknown heading".to_string(),
        });
        let (code, ec) = classify(&err);
        assert_eq!(code, ExitCode::from(2));
        assert_eq!(ec.as_str(), "spec.delta_parse_error");
    }

    #[test]
    fn local_change_not_archivable_maps_to_1() {
        let err = anyhow::Error::from(LocalProviderError::ChangeNotArchivable {
            reason: "exists".to_string(),
        });
        let (code, ec) = classify(&err);
        assert_eq!(code, ExitCode::from(1));
        assert_eq!(ec.as_str(), "archive.change_not_archivable");
    }

    #[test]
    fn local_spec_delta_conflict_maps_to_7() {
        let err = anyhow::Error::from(LocalProviderError::SpecDeltaConflict {
            capability: "auth".to_string(),
            requirement: "User login".to_string(),
            operation: "MODIFIED",
        });
        let (code, ec) = classify(&err);
        assert_eq!(code, ExitCode::from(7));
        assert_eq!(ec.as_str(), "spec.delta_conflict");
    }

    #[test]
    fn local_spec_delta_parse_error_maps_to_2() {
        let err = anyhow::Error::from(LocalProviderError::SpecDeltaParseError {
            capability: "auth".to_string(),
            message: "bad heading".to_string(),
        });
        let (code, ec) = classify(&err);
        assert_eq!(code, ExitCode::from(2));
        assert_eq!(ec.as_str(), "spec.delta_parse_error");
    }

    #[test]
    fn fallthrough_internal_error() {
        let err = anyhow::anyhow!("some random failure");
        let (code, ec) = classify(&err);
        assert_eq!(code, ExitCode::from(1));
        assert_eq!(ec.as_str(), "internal.error");
    }

    #[test]
    fn deterministic_classify_for_identical_input() {
        let err1 = anyhow::Error::from(ProviderError::NotAuthenticated {
            provider_name: "acme".to_string(),
        });
        let err2 = anyhow::Error::from(ProviderError::NotAuthenticated {
            provider_name: "acme".to_string(),
        });
        assert_eq!(classify(&err1), classify(&err2));
    }

    #[test]
    fn all_error_codes_match_naming_regex() {
        // Spec: `^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$`
        let codes = [
            "provider.not_authenticated",
            "provider.unavailable",
            "change.already_exists",
            "change.not_found",
            "change.invalid_id",
            "input.invalid",
            "internal.error",
            "artifact.already_exists",
            "artifact.missing_capability",
            "artifact.invalid_capability",
            "archive.change_not_archivable",
            "spec.delta_conflict",
            "spec.delta_parse_error",
        ];
        for c in codes {
            assert!(matches_naming(c), "code does not match naming regex: {c}");
        }
    }

    fn matches_naming(c: &str) -> bool {
        let parts: Vec<&str> = c.splitn(2, '.').collect();
        if parts.len() != 2 {
            return false;
        }
        for p in &parts {
            if p.is_empty() {
                return false;
            }
            let bytes = p.as_bytes();
            if !bytes[0].is_ascii_lowercase() {
                return false;
            }
            for &b in &bytes[1..] {
                if !(b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_') {
                    return false;
                }
            }
        }
        true
    }
}
