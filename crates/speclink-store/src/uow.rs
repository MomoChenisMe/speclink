//! The Unit of Work: the only write path of the contract. Writes are staged
//! against a UoW and take effect atomically at commit; every staged
//! operation carries its CAS precondition.

use crate::types::{DocumentId, ExpectedRevision, Revision, Scope};

/// Identity of the command performing a unit of work: which command it is
/// and who runs it. Recorded in history and available to drivers for
/// auditing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandContext {
    pub command: String,
    pub actor: String,
}

/// One staged operation inside a unit of work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StagedOp {
    /// Create or update a document. `ExpectedRevision::Absent` is creation
    /// ("must not already exist"); `At(rev)` is a CAS update.
    Put {
        doc: DocumentId,
        content: String,
        expected: ExpectedRevision,
    },
    /// Delete a document the writer read at `expected`. Takes effect as a
    /// tombstone revision — history is never rewritten.
    Delete {
        doc: DocumentId,
        expected: Revision,
    },
}

impl StagedOp {
    pub fn doc(&self) -> &DocumentId {
        match self {
            StagedOp::Put { doc, .. } => doc,
            StagedOp::Delete { doc, .. } => doc,
        }
    }
}

/// A staging buffer for one atomic commit. Obtained from
/// [`crate::TeamStore::begin_unit_of_work`]; consumed by `commit` or
/// `rollback`. Accessors are public so drivers outside this crate can apply
/// the staged operations.
#[derive(Debug, Clone)]
pub struct UnitOfWork {
    scope: Scope,
    ctx: CommandContext,
    ops: Vec<StagedOp>,
}

impl UnitOfWork {
    pub fn new(scope: Scope, ctx: CommandContext) -> Self {
        Self {
            scope,
            ctx,
            ops: Vec::new(),
        }
    }

    /// Stage a create: the document must not exist at commit time.
    pub fn create(&mut self, doc: DocumentId, content: impl Into<String>) {
        self.put(doc, content, ExpectedRevision::Absent);
    }

    /// Stage a CAS update against the revision the writer read.
    pub fn update(&mut self, doc: DocumentId, content: impl Into<String>, read_at: Revision) {
        self.put(doc, content, ExpectedRevision::At(read_at));
    }

    /// Stage a write with an explicit precondition.
    pub fn put(&mut self, doc: DocumentId, content: impl Into<String>, expected: ExpectedRevision) {
        self.ops.push(StagedOp::Put {
            doc,
            content: content.into(),
            expected,
        });
    }

    /// Stage a delete of a document the writer read at `expected`.
    pub fn delete(&mut self, doc: DocumentId, expected: Revision) {
        self.ops.push(StagedOp::Delete { doc, expected });
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn context(&self) -> &CommandContext {
        &self.ctx
    }

    pub fn ops(&self) -> &[StagedOp] {
        &self.ops
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ProjectId, RepoId};

    #[test]
    fn staging_preserves_order_and_preconditions() {
        let scope = Scope::new(ProjectId::new("default"), RepoId::new("main"));
        let ctx = CommandContext {
            command: "archive-change".into(),
            actor: "alice".into(),
        };
        let mut uow = UnitOfWork::new(scope.clone(), ctx.clone());
        uow.create(DocumentId::WorkflowConfig, "cfg v1");
        uow.update(
            DocumentId::CanonicalSpec {
                capability: "auth".into(),
            },
            "spec v2",
            Revision(4),
        );
        uow.delete(
            DocumentId::ChangeMeta {
                change: "old".into(),
            },
            Revision(4),
        );

        assert_eq!(uow.scope(), &scope);
        assert_eq!(uow.context(), &ctx);
        assert_eq!(
            uow.ops(),
            &[
                StagedOp::Put {
                    doc: DocumentId::WorkflowConfig,
                    content: "cfg v1".into(),
                    expected: ExpectedRevision::Absent,
                },
                StagedOp::Put {
                    doc: DocumentId::CanonicalSpec {
                        capability: "auth".into()
                    },
                    content: "spec v2".into(),
                    expected: ExpectedRevision::At(Revision(4)),
                },
                StagedOp::Delete {
                    doc: DocumentId::ChangeMeta {
                        change: "old".into()
                    },
                    expected: Revision(4),
                },
            ]
        );
    }
}
