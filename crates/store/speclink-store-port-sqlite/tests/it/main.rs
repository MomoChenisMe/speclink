//! Integration tests of the SQLite example, one section per requirement of
//! the `store-port-examples` spec.
//!
//! One module for each requirement; `support` has the fixtures.

mod support;

mod availability;
mod boundaries;
mod conformance;
mod consistency;
mod events;
mod layout;
mod two_objects;
