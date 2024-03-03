use crate::helpers::{ko, ok, run};
use std::path::Path;
use std::process::exit;
use std::time::Instant;

pub mod helpers;
use gettextrs::{bind_textdomain_codeset, gettext, textdomain};

fn check_rust(started: Instant) {
    let verify = Instant::now();
    run(
        "Project",
        "cargo",
        "verify-project --quiet --color=always",
        gettext("Project is valid").as_str(),
        gettext("Project is not valid").as_str(),
        verify,
    );
    let audit = Instant::now();
    run(
        "Audit",
        "cargo",
        "audit",
        gettext("Audit no detect errors").as_str(),
        gettext("Audit detect errors").as_str(),
        audit,
    );
    let clippy = Instant::now();
    run("Clippy",
        "cargo",
        "clippy -- -F keyword_idents -F warnings -F let-underscore -F rust-2018-compatibility -F rust-2018-idioms  -F rust-2021-compatibility -F future-incompatible -F unused -F unused_crate_dependencies -F unused_extern_crates  -D unused_macro_rules -F unused_results -F unused_qualifications -F nonstandard-style -F macro_use_extern_crate -F absolute_paths_not_starting_with_crate -F ambiguous_glob_imports -F clippy::all -F clippy::perf -F clippy::pedantic -F clippy::style -F clippy::suspicious -F clippy::correctness -F clippy::nursery -F clippy::complexity -F clippy::cargo",
        gettext("Your code is correct").as_str(),
        gettext("Your code is incorrect").as_str(),
        clippy);
    run(
        "Tests",
        "cargo",
        "test --no-fail-fast",
        gettext("No test failures").as_str(),
        gettext("Test have failures").as_str(),
        clippy,
    );
    let check = Instant::now();
    run(
        "Check",
        "cargo",
        "check",
        gettext("Your code is correct").as_str(),
        gettext("Your code is incorrect").as_str(),
        check,
    );
    let fmt = Instant::now();
    run(
        "Format",
        "cargo",
        "fmt --check",
        gettext("Your code is formatted correctness").as_str(),
        gettext("Your project is bad formatted").as_str(),
        fmt,
    );
    ok(gettext("Your code can be committed").as_str(), started);
    println!();
}

fn main() {
    assert!(textdomain("zuu").is_ok());
    assert!(bind_textdomain_codeset("zuu", "UTF-8").is_ok());
    let s = Instant::now();
    if Path::new("Cargo.toml").exists() {
        check_rust(Instant::now());
        exit(0);
    }
    ko(gettext("Source code not supported").as_str(), s);
    exit(1);
}