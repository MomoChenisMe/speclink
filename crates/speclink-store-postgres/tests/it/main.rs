//! Single `harness = false` entry point for the PostgreSQL driver's suite.
//!
//! Modules run in the alphabetical order the six standalone binaries used to,
//! through one `support::run` call — so a missing database reports every test
//! as skipped in one block, and failures aggregate before the exit code.

mod support;

mod bundle_and_outbox;
mod conformance;
mod infra;
mod resilience;
mod single_writer;
mod version_gate;

fn main() {
    let mut tests: Vec<(&str, fn())> = Vec::new();
    tests.extend_from_slice(bundle_and_outbox::tests());
    tests.extend_from_slice(conformance::tests());
    tests.extend_from_slice(infra::tests());
    tests.extend_from_slice(resilience::tests());
    tests.extend_from_slice(single_writer::tests());
    tests.extend_from_slice(version_gate::tests());
    support::run(&tests);
}
