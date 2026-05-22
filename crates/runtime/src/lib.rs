//! SpecLink runtime：負責 bootstrap orchestration（git check、prepare-then-commit
//! pattern、`.gitignore` 寫入）、路徑解析（`git rev-parse --git-common-dir`）、
//! 以及 provider 串接。

#![allow(clippy::doc_markdown)]

pub mod artifact_ops;
pub mod bootstrap;
pub mod change_ops;
pub mod error;
pub mod git;
pub mod gitignore;
pub mod ops;
pub mod paths;

pub use artifact_ops::ArtifactOperations;
pub use bootstrap::Bootstrap;
pub use change_ops::{ArtifactRef, ChangeOperations, DEFAULT_SCHEMA_ID, ShowChangeData};
pub use error::{RuntimeError, codes, finding_codes};
pub use git::{GitProbe, RealGitProbe};
pub use ops::Operations;
pub use paths::{
    ARTIFACT_ROOT, STATE_ROOT_NAMESPACE, artifact_root, display_state_root, resolve_state_root,
};
