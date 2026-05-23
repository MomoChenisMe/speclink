//! SpecLink runtime：負責 bootstrap orchestration（git check、prepare-then-commit
//! pattern、`.gitignore` 寫入）、路徑解析（`git rev-parse --git-common-dir`）、
//! 以及 provider 串接。

#![allow(clippy::doc_markdown)]

pub mod apply_ops;
pub mod archive_ops;
pub mod artifact_ops;
pub mod bootstrap;
pub mod change_ops;
pub mod config_ops;
pub mod dev_precheck;
pub mod error;
pub mod git;
pub mod gitignore;
pub mod ops;
pub mod paths;
pub mod state_machine;
pub mod task_ops;

pub use apply_ops::{ApplyOperations, ApplyPauseData, ApplyStartData};
pub use archive_ops::{ArchiveData, ArchiveOperations, ArchiveOutcome};
pub use artifact_ops::ArtifactOperations;
pub use bootstrap::Bootstrap;
pub use change_ops::{ArtifactRef, ChangeOperations, DEFAULT_SCHEMA_ID, ShowChangeData};
pub use config_ops::ConfigOperations;
pub use dev_precheck::{precheck_a2_archived, precheck_a3_archived};
pub use error::{RuntimeError, RuntimeWarning, codes, finding_codes, task_codes};
pub use git::{GitProbe, RealGitProbe};
pub use ops::Operations;
pub use paths::{
    ARTIFACT_ROOT, STATE_ROOT_NAMESPACE, artifact_root, display_state_root, resolve_state_root,
};
pub use state_machine::{
    AllTasksDoneOutcome, ReviewPolicy, all_tasks_done_outcome, is_legal_transition,
    legal_transitions, proposing_target, resolve_actor,
};
pub use task_ops::{TaskDoneData, TaskItem, TaskListData, TaskOperations, TaskUndoData};
