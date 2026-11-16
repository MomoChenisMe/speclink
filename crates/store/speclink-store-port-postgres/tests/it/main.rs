//! Integration tests of the PostgreSQL example, one section per requirement
//! of the `store-port-examples` spec.
//!
//! This binary is its own `main` (`harness = false`). The standard test
//! runner cannot decide at run time to leave a test out, and a test that
//! needs a database must not report `ok` when there is none. With no
//! `SPECLINK_TEST_POSTGRES_URL`, each of those tests is printed as `skipped`
//! and the exit code is 0. With it, each test runs in a new schema of its
//! own, which is dropped at the end.

use std::io::{self, Write};
use std::panic::{self, AssertUnwindSafe};

use speclink_store_port_postgres::PASSWORD_VAR;
use support::{base_url, NotChecked, Test, URL_VAR};

mod support;

mod availability;
mod boundaries;
mod conformance;
mod consistency;
mod events;
mod layout;
mod two_objects;

/// One line that starts a database and points the tests at it.
const ENABLE_HINT: &str = "docker run --rm -d -p 5432:5432 -e POSTGRES_PASSWORD=speclink --name speclink-pg postgres:15 && export SPECLINK_TEST_POSTGRES_URL=postgres://postgres:speclink@localhost:5432/postgres";

/// How the tests that ran ended.
#[derive(Default)]
struct Tally {
    passed: usize,
    not_checked: usize,
    failed: Vec<&'static str>,
}

fn run(tests: &[Test], tally: &mut Tally) {
    for (name, test) in tests {
        print!("test {name} ... ");
        io::stdout().flush().ok();
        match panic::catch_unwind(AssertUnwindSafe(test)) {
            Ok(()) => {
                println!("ok");
                tally.passed += 1;
            }
            Err(payload) => match payload.downcast_ref::<NotChecked>() {
                // Printed as `skipped`, so the CI guard does not let a test
                // that checked nothing pass for a test that ran.
                Some(NotChecked(reason)) => {
                    println!("skipped ({reason})");
                    tally.not_checked += 1;
                }
                None => {
                    println!("FAILED");
                    tally.failed.push(name);
                }
            },
        }
    }
}

fn main() {
    // The store reads this variable in preference to the password of the
    // test URL. One that the developer's shell exports must not decide the
    // tests; the one test about it sets its own value.
    std::env::remove_var(PASSWORD_VAR);

    let no_database = [availability::NO_DATABASE, boundaries::NO_DATABASE].concat();
    let database = [
        conformance::TESTS,
        events::TESTS,
        layout::TESTS,
        consistency::TESTS,
        two_objects::TESTS,
        availability::TESTS,
        boundaries::TESTS,
    ]
    .concat();
    let mut tally = Tally::default();
    run(&no_database, &mut tally);
    if base_url().is_some() {
        run(&database, &mut tally);
    } else {
        for (name, _) in &database {
            println!("test {name} ... skipped");
        }
        tally.not_checked += database.len();
        println!();
        println!("{URL_VAR} names no PostgreSQL instance. Enable these tests with:");
        println!("  {ENABLE_HINT}");
    }
    println!();
    // The word "skipped" is in the output only when a test did not run.
    let not_checked = match tally.not_checked {
        0 => String::new(),
        n => format!("; {n} skipped"),
    };
    println!(
        "test result: {}. {} passed; {} failed{not_checked}",
        if tally.failed.is_empty() {
            "ok"
        } else {
            "FAILED"
        },
        tally.passed,
        tally.failed.len()
    );
    for name in &tally.failed {
        println!("    {name}");
    }
    if !tally.failed.is_empty() {
        std::process::exit(1);
    }
}
