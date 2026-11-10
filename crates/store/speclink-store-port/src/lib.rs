//! The store port: the storage contract a host implements.
//!
//! A host that embeds speclink owns the storage medium, the connection,
//! the migrations and the history. speclink only asks for three
//! operations on [`StorePort`]: list, read and commit. This crate holds the
//! contract, the [`conformance`] suite that checks any implementation, and
//! [`MemoryStorePort`], an in-memory reference implementation. It does not
//! depend on the engine.
//!
//! # The three guarantees
//!
//! 1. **A commit applies all of its operations or none of them.**
//! 2. **One expectation that does not hold rejects the whole commit.** The
//!    store returns [`PortError::Conflict`] with one key that does not
//!    match, the caller's expectation and the document's current version.
//!    A rejected commit changes no document, no scope version and keeps no
//!    event.
//! 3. **The events travel in the same commit as the writes.** A store that
//!    keeps events keeps them only when the commit succeeds, in commit
//!    order, tagged with the commit's scope version.
//!
//! # What a host does to implement the port
//!
//! - Run each [`StorePort::commit`] as **one transaction** of its database.
//! - In that transaction, **check every expectation before the first
//!   write**. Do not check and write one operation at a time.
//! - Give the written documents the new scope version as their version,
//!   and the commit's [`Commit::at`] as their last-modified time.
//! - Answer [`StorePort::read`] and [`StorePort::list`] from one completed
//!   commit: never mix documents of two commits in one result.
//! - Run [`conformance::run`] against the implementation, with a
//!   [`conformance::PortHarness`] that gives a new empty scope for each
//!   check.
//!
//! The suite cannot see what a crash in the middle of a commit leaves
//! behind, because it asks for no fault injection points. A transaction is
//! the only safe way to meet guarantee 1.
//!
//! # The rules
//!
//! - **The trait** is synchronous and object-safe. One store object serves
//!   every request from many threads, so it is `Send + Sync`. Async hosts
//!   adapt at their own boundary.
//! - **A scope** ([`ScopeId`]) is an opaque string of 1 to 256 bytes. There
//!   is no operation that makes a scope: a scope that was never written
//!   lists as version 0 with no documents, and its first commit succeeds.
//! - **A document key** ([`DocKey`]) is 1 to 512 bytes of `/`-separated
//!   segments with no control character, no backslash, no empty segment
//!   and no `.` or `..` segment. A [`KeyPrefix`] matches bytes, not
//!   segments. The [`keys`] module maps each kind of speclink document to
//!   its key.
//! - **A version** ([`Version`]) is an integer from 0 to [`MAX_VERSION`]
//!   (2^53 − 1), so a JavaScript number holds it exactly. Each successful
//!   commit makes the scope version strictly larger; it need not grow by
//!   one. A document's version is the scope version of the commit that last
//!   wrote it, so a deleted and recreated document always gets a larger
//!   version. [`Timestamp`] is milliseconds since the Unix epoch, with the
//!   same limit.
//! - **A commit** ([`Commit`]) carries the actor, the command name, the
//!   time, the writes and deletes, and the events. Every write expects a
//!   version or [`Expected::Absent`]; every delete expects a version. There
//!   is no write without an expectation. A commit with only events is
//!   valid and moves the scope version.
//! - **A failure** ([`PortError`]) is a conflict, a temporary failure
//!   ([`PortError::Unavailable`], the caller can retry) or another backend
//!   failure. A missing document is not a failure. A malformed value is an
//!   [`InvalidValue`] from its constructor, so a store never receives one.
//! - **Events** go to the store in the commit. The store decides whether
//!   to keep them; the contract has no operation that reads them back.

pub mod conformance;
mod error;
pub mod keys;
mod memory;
mod types;

pub use error::{InvalidValue, PortError};
pub use memory::{MemoryStorePort, RecordedEvent};
pub use types::{
    Actor, Commit, DocKey, Document, Event, Expected, KeyPrefix, ListEntry, Listing, Op, ScopeId,
    Timestamp, Version, MAX_KEY_BYTES, MAX_SCOPE_BYTES, MAX_VERSION,
};

/// The storage contract: list, read and commit, each on one scope.
///
/// The trait is synchronous and object-safe. One store object serves every
/// request, so it must be `Send + Sync`.
pub trait StorePort: Send + Sync {
    /// The scope's current version and every document whose key starts
    /// with `prefix`, in byte order of the keys. A scope that was never
    /// written lists as version 0 with no entries.
    fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError>;

    /// One result per requested key, in the same order. A key with no
    /// document gives `None`, not an error. All documents of one read come
    /// from the same completed commit.
    fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError>;

    /// Apply every operation of `commit`, or none of them, and return the
    /// new scope version.
    fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_store_object_is_shared_across_threads() {
        let store: &dyn StorePort = &MemoryStorePort::new();
        let scope = ScopeId::new("s").unwrap();
        let prefix = KeyPrefix::new("").unwrap();
        let listing = std::thread::scope(|threads| {
            threads
                .spawn(|| store.list(&scope, &prefix))
                .join()
                .unwrap()
        })
        .unwrap();
        assert_eq!(listing.version, Version::ZERO);
    }
}
