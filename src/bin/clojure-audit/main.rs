use std::process::ExitCode;
use zuu::{Language, Zuu};

fn main() -> ExitCode {
    Zuu::new(Language::Clojure).check()
}