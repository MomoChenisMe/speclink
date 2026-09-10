//! CommandOutcome → payload 的型別化轉換層。「哪個 Command 產哪個 outcome」
//! 的不變式由 [`super::execute`] 持有，消費端只認 payload 型別——同一型別由
//! 多個 variant 共用時，該支轉換接受全部載此型別的 variant（design D1）。
//! 轉換失敗回 [`WrongOutcome`] 錯誤值，不 panic（design D2）。

use super::{
    CommandOutcome, DiscussArchiveOutcome, DiscussBindOutcome, DiscussConcludeOutcome,
    DiscussPromoteOutcome, DiscussRoundOutcome, DiscussShowOutcome, DiscussSubjectOutcome,
    InProgressOutcome, InProgressRemoveOutcome, InstructionsOutcome, ListOutcome,
    NewArtifactOutcome, NewChangeOutcome, ShowOutcome, TaskFlipOutcome, ValidateOutcome,
};
use crate::analyzer::AnalyzeReport;
use crate::archive::ArchiveOutcome;
use crate::discard::DiscardOutcome;
use crate::discuss::{DiscussionHit, DiscussionInfo};
use crate::status::StatusReport;
use crate::trace::TraceReport;

/// 轉換要求的 payload 型別與 outcome 實際的 variant 不合。現行引擎不變式下
/// 不會觸發；作為錯誤值而非 panic，讓函式庫呼叫端自行決定處置。
#[derive(Debug)]
pub struct WrongOutcome {
    /// 呼叫端要求的 payload 型別名。
    pub expected: &'static str,
    /// outcome 實際的 variant 名。
    pub actual: &'static str,
}

impl std::fmt::Display for WrongOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "wrong outcome: expected {} payload, got CommandOutcome::{}",
            self.expected, self.actual
        )
    }
}

impl std::error::Error for WrongOutcome {}

fn variant_name(o: &CommandOutcome) -> &'static str {
    match o {
        CommandOutcome::List(_) => "List",
        CommandOutcome::Show(_) => "Show",
        CommandOutcome::Status(_) => "Status",
        CommandOutcome::Instructions(_) => "Instructions",
        CommandOutcome::Validate(_) => "Validate",
        CommandOutcome::Analyze(_) => "Analyze",
        CommandOutcome::Trace(_) => "Trace",
        CommandOutcome::ArtifactCat(_) => "ArtifactCat",
        CommandOutcome::Language(_) => "Language",
        CommandOutcome::DiscussList(_) => "DiscussList",
        CommandOutcome::DiscussShow(_) => "DiscussShow",
        CommandOutcome::DiscussSearch(_) => "DiscussSearch",
        CommandOutcome::NewChange(_) => "NewChange",
        CommandOutcome::NewArtifact(_) => "NewArtifact",
        CommandOutcome::TaskDone(_) => "TaskDone",
        CommandOutcome::TaskUndone(_) => "TaskUndone",
        CommandOutcome::TaskMove(_) => "TaskMove",
        CommandOutcome::Claim(_) => "Claim",
        CommandOutcome::InProgressAdd(_) => "InProgressAdd",
        CommandOutcome::InProgressRemove(_) => "InProgressRemove",
        CommandOutcome::Archive(_) => "Archive",
        CommandOutcome::Discard(_) => "Discard",
        CommandOutcome::DiscussNew(_) => "DiscussNew",
        CommandOutcome::DiscussContext(_) => "DiscussContext",
        CommandOutcome::DiscussAddRound(_) => "DiscussAddRound",
        CommandOutcome::DiscussConclude(_) => "DiscussConclude",
        CommandOutcome::DiscussPromote(_) => "DiscussPromote",
        CommandOutcome::DiscussLink(_) => "DiscussLink",
        CommandOutcome::DiscussSeal(_) => "DiscussSeal",
        CommandOutcome::DiscussArchive(_) => "DiscussArchive",
        CommandOutcome::DiscussDiscard(_) => "DiscussDiscard",
        CommandOutcome::ReviewAddRound(_) => "ReviewAddRound",
        CommandOutcome::ReviewShow(_) => "ReviewShow",
        CommandOutcome::ReviewStamp(_) => "ReviewStamp",
        CommandOutcome::ReviewDiscard(_) => "ReviewDiscard",
        CommandOutcome::VerifyAddRound(_) => "VerifyAddRound",
        CommandOutcome::VerifyShow(_) => "VerifyShow",
        CommandOutcome::VerifyStamp(_) => "VerifyStamp",
        CommandOutcome::VerifyDiscard(_) => "VerifyDiscard",
    }
}

/// 表驅動生成 TryFrom<CommandOutcome>：一行一 payload 型別，列出該型別
/// 接受的全部 variant。
macro_rules! typed_outcomes {
    ($($ty:ty => [$($variant:ident),+ $(,)?]),+ $(,)?) => {$(
        impl TryFrom<CommandOutcome> for $ty {
            type Error = WrongOutcome;
            fn try_from(outcome: CommandOutcome) -> Result<Self, WrongOutcome> {
                match outcome {
                    $(CommandOutcome::$variant(payload) => Ok(payload),)+
                    other => Err(WrongOutcome {
                        expected: stringify!($ty),
                        actual: variant_name(&other),
                    }),
                }
            }
        }
    )+};
}

typed_outcomes! {
    ListOutcome => [List],
    ShowOutcome => [Show],
    StatusReport => [Status],
    InstructionsOutcome => [Instructions],
    ValidateOutcome => [Validate],
    AnalyzeReport => [Analyze],
    TraceReport => [Trace],
    String => [ArtifactCat, Language],
    Vec<DiscussionInfo> => [DiscussList],
    DiscussShowOutcome => [DiscussShow],
    Vec<DiscussionHit> => [DiscussSearch],
    NewChangeOutcome => [NewChange],
    NewArtifactOutcome => [NewArtifact],
    TaskFlipOutcome => [TaskDone, TaskUndone],
    InProgressOutcome => [InProgressAdd],
    InProgressRemoveOutcome => [InProgressRemove],
    ArchiveOutcome => [Archive],
    DiscardOutcome => [Discard],
    DiscussionInfo => [DiscussNew],
    DiscussSubjectOutcome => [DiscussContext, DiscussDiscard],
    DiscussRoundOutcome => [DiscussAddRound],
    DiscussConcludeOutcome => [DiscussConclude],
    DiscussPromoteOutcome => [DiscussPromote],
    DiscussBindOutcome => [DiscussLink, DiscussSeal],
    DiscussArchiveOutcome => [DiscussArchive],
}

#[cfg(test)]
mod tests {
    use crate::command::*;

    fn flip_outcome(change: &str) -> TaskFlipOutcome {
        TaskFlipOutcome {
            change: change.into(),
            task_id: 1,
            task_id_arg: "1".into(),
            description: "demo task".into(),
            already: false,
            stable_id: None,
            touched_files: vec![],
        }
    }

    #[test]
    fn list_outcome_converts_from_list_variant() {
        let o = CommandOutcome::List(ListOutcome { changes: None, specs: None });
        let got = ListOutcome::try_from(o).expect("List variant carries ListOutcome");
        assert!(got.changes.is_none());
        assert!(got.specs.is_none());
    }

    #[test]
    fn wrong_variant_yields_wrong_outcome_naming_both_sides() {
        let o = CommandOutcome::Language("doc body".into());
        let err: WrongOutcome =
            ListOutcome::try_from(o).expect_err("Language variant does not carry ListOutcome");
        // WrongOutcome 是標準錯誤值：Display 同時載期望型別名與實際 variant 名。
        let _: &dyn std::error::Error = &err;
        let msg = err.to_string();
        assert!(msg.contains("ListOutcome"), "expected type name missing: {msg}");
        assert!(msg.contains("Language"), "actual variant name missing: {msg}");
    }

    #[test]
    fn task_flip_outcome_accepts_done_and_undone() {
        let done = TaskFlipOutcome::try_from(CommandOutcome::TaskDone(flip_outcome("a")))
            .expect("TaskDone carries TaskFlipOutcome");
        assert_eq!(done.change, "a");
        let undone = TaskFlipOutcome::try_from(CommandOutcome::TaskUndone(flip_outcome("b")))
            .expect("TaskUndone carries TaskFlipOutcome");
        assert_eq!(undone.change, "b");
    }

    #[test]
    fn string_accepts_artifact_cat_and_language() {
        let cat = String::try_from(CommandOutcome::ArtifactCat("cat body".into()))
            .expect("ArtifactCat carries String");
        assert_eq!(cat, "cat body");
        let lang = String::try_from(CommandOutcome::Language("lang body".into()))
            .expect("Language carries String");
        assert_eq!(lang, "lang body");
    }
}
