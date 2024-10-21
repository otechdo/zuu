use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::execute;
use crossterm::style::{Color, Print, SetForegroundColor};
use crossterm::terminal::{size, Clear, ClearType};
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{stdout, Error, Stdout};
use std::path::Path;
use std::process::{Command, ExitCode};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub const FORMAT_OK: &str = "Source code is formatted correctly";
pub const FORMAT_ERR: &str = "Source code is not formatted correctly. Please run the formatter";
pub const AUDIT_OK: &str = "No security vulnerabilities in the code";
pub const AUDIT_ERR: &str = "Security vulnerabilities detected in the code";
pub const TEST_OK: &str = "All tests pass";
pub const TEST_ERR: &str = "Some tests did not pass. Please review the test results";
pub const LINT_OK: &str = "Your code respect style requirements";
pub const LINT_ERR: &str = "Your code does not meet style requirements";
pub const LICENSE_ERR: &str = "Some dependencies may have incompatible licenses";
pub const LICENSE_OK: &str = "No dependencies incompatible licenses";
pub const TARGET_FMT: &str = "zuu-fmt";
pub const TARGET_AUDIT: &str = "zuu-audit";
pub const TARGET_TEST: &str = "zuu-test";
pub const TARGET_LICENSE: &str = "zuu-license";
pub const TARGET_LINT: &str = "zuu-lint";

pub enum Language {
    Rust,
    Go,
    C,
    Cpp,
    D,
    Python,
    Php,
    Java,
    Kotlin,
    Swift,
    Ruby,
    Perl,
    Scala,
    TypeScript,
    Elixir,
    Haskell,
    Clojure,
    Bash,
    ObjectiveC,
    Erlang,
    Lua,
    FSharp, // F#
    R,
    Julia,
    Crystal,
    Groovy,
    Dart,
    Matlab, // MATLAB
    Cobol,
    Fortran,
    Nim,
    Nodejs,
    Vlang, // V language
    OCaml,
    Tcl,
    VHDL,
    Unknown,
}
#[derive(PartialEq, Eq, Hash)]
pub enum Checked {
    Fmt,
    Audit,
    Test,
    License,
    Lint,
}

pub fn zuu_exit(status: &Result<(), Error>) -> ExitCode {
    if status.is_err() {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
pub fn ok(output: &mut Stdout, description: &str, x: u16) -> std::io::Result<()> {
    let (cols, _rows) = size().expect("failed to get terminal size");

    let status: &str = "[ ok ]";

    let status_len: u16 = status.len() as u16;
    let status_position: u16 = cols.saturating_sub(status_len);

    execute!(
        output,
        SetForegroundColor(Color::Green),
        MoveTo(0, x),
        Print("*"),
        MoveTo(2, x),
        SetForegroundColor(Color::White),
        Print(description),
        MoveTo(status_position, 1),
        SetForegroundColor(Color::Blue),
        MoveTo(status_position, x),
        Print("["),
        SetForegroundColor(Color::Green),
        Print(" ok "),
        SetForegroundColor(Color::Blue),
        Print("]"),
        SetForegroundColor(Color::Reset),
    )
}

pub fn ko(output: &mut Stdout, description: &str, x: u16) -> std::io::Result<()> {
    let (cols, _rows) = size().expect("failed to get terminal size");

    let status: &str = "[ !! ]";

    let status_len: u16 = status.len() as u16;
    let status_position: u16 = cols.saturating_sub(status_len);

    execute!(
        output,
        SetForegroundColor(Color::Red),
        MoveTo(0, x),
        Print("*"),
        MoveTo(2, x),
        SetForegroundColor(Color::White),
        Print(description),
        SetForegroundColor(Color::Blue),
        MoveTo(status_position, x),
        Print("["),
        SetForegroundColor(Color::Red),
        Print(" !! "),
        SetForegroundColor(Color::Blue),
        Print("]"),
        SetForegroundColor(Color::Reset),
    )
}

pub fn exec(
    output: &mut Stdout,
    description: &'static str,
    cmd: &mut Command,
    f: &'static str,
    x: u16,
) -> std::io::Result<()> {
    let spinner_done = Arc::new(AtomicBool::new(false));
    let spinner_done_clone = Arc::clone(&spinner_done);
    let (cols, _rows) = size().expect("failed to get terminal size");
    let status: &str = "   ";
    let status_len: u16 = status.len() as u16;
    let status_position: u16 = cols.saturating_sub(status_len);
    assert!(execute!(
        output,
        MoveTo(0, x),
        SetForegroundColor(Color::Green),
        Print("*"),
        MoveTo(2, x),
        SetForegroundColor(Color::White),
        Print(description),
        MoveTo(status_position, x),
        SetForegroundColor(Color::Green),
        Print(" "),
        SetForegroundColor(Color::Reset),
    )
    .is_ok());
    let spinner_thread = thread::spawn(move || {
        let mut output = stdout();
        while !spinner_done_clone.load(Ordering::SeqCst) {
            let status: &str = "[ :: ]";
            let status_len: u16 = status.len() as u16;
            let spinner_chars = [". ", "..", ".:", "::"];
            let status_position: u16 = cols.saturating_sub(status_len);
            for spin in &spinner_chars {
                assert!(execute!(
                    output,
                    Hide,
                    SetForegroundColor(Color::Green),
                    MoveTo(0, x),
                    Print("*"),
                    MoveTo(2, x),
                    SetForegroundColor(Color::White),
                    Print(description),
                    MoveTo(status_position, x),
                    SetForegroundColor(Color::Blue),
                    Print("["),
                    SetForegroundColor(Color::Green),
                    Print(format!(" {spin} ")),
                    SetForegroundColor(Color::Blue),
                    Print("]"),
                    SetForegroundColor(Color::Reset),
                )
                .is_ok());
                sleep(Duration::from_millis(400));
            }
        }
    });

    let output = cmd
        .stdout(File::create(format!("zuu/stdout/{f}")).expect("failed to create output"))
        .stderr(File::create(format!("zuu/stderr/{f}")).expect("failed to create output"))
        .spawn()?
        .wait()?
        .success();

    spinner_done.store(true, Ordering::SeqCst);
    spinner_thread.join().unwrap();
    assert!(execute!(stdout(), MoveTo(0, x), Clear(ClearType::CurrentLine)).is_ok());
    if output {
        return Ok(());
    }
    Err(Error::other("a error encountered"))
}
fn check(x: &HashMap<Checked, bool>) -> Result<(), Error> {
    let mut output: Stdout = stdout();
    for (i, v) in x {
        match i {
            Checked::Fmt => {
                if v.eq(&true) {
                    assert!(ok(
                        &mut output,
                        "The source code format respect the standard",
                        0
                    )
                    .is_ok());
                } else {
                }
            }
            Checked::Audit => {
                if v.eq(&true) {
                    assert!(ok(&mut output, "No vulnerabilities has been founded", 1).is_ok());
                } else {
                }
            }
            Checked::Test => {
                if v.eq(&true) {
                    assert!(ok(&mut output, "All tests passes", 1).is_ok());
                } else {
                }
            }
            Checked::License => {
                if v.eq(&true) {
                    assert!(ok(&mut output, "No licences problem has bee founded", 1).is_ok());
                } else {
                }
            }
            Checked::Lint => {
                if v.eq(&true) {
                    assert!(ok(&mut output, "No problem detected", 1).is_ok());
                } else {
                }
            }
        }
    }
    Err(Error::other("Zuu has detected errors"))
}
pub struct Zuu {
    checked: HashMap<Checked, bool>,
    language: Language,
}
impl Zuu {
    pub fn new(lang: Language) -> Self {
        create_dir_all("zuu").expect("msg");
        create_dir_all("zuu/stderr").expect("msg");
        create_dir_all("zuu/stdout").expect("msg");
        execute!(&mut stdout(), Clear(ClearType::All)).expect("msg");
        Self {
            checked: HashMap::new(),
            language: lang,
        }
    }

    fn rust(&mut self) -> Result<(), Error> {
        if Path::new("Cargo.toml").is_file() {
            let mut results: Vec<bool> = Vec::new();
            let mut output: Stdout = stdout();
            execute!(&mut output, Clear(ClearType::All)).expect("msg");
            if exec(
                &mut output,
                "Checking licenses",
                &mut Command::new("cargo").arg("deny").arg("check"),
                "license",
                1,
            )
            .is_ok()
            {
                results.push(true);
                assert!(ok(&mut output, LICENSE_OK, 1).is_ok());
            } else {
                assert!(ko(&mut output, LICENSE_ERR, 1).is_ok());
                results.push(false);
            }
            if exec(
                &mut output,
                "Auditing code",
                &mut Command::new("cargo").arg("audit"),
                "audit",
                2,
            )
            .is_ok()
            {
                results.push(true);
                assert!(ok(&mut output, AUDIT_OK, 2).is_ok());
            } else {
                results.push(false);
                assert!(ko(&mut output, AUDIT_ERR, 2).is_ok());
            }
            if exec(
                &mut output,
                "Checking code",
                &mut Command::new("cargo").arg("clippy"),
                "lint",
                3,
            )
            .is_ok()
            {
                results.push(true);
                assert!(ok(&mut output, LINT_OK, 3).is_ok());
            } else {
                results.push(false);
                assert!(ko(&mut output, LINT_ERR, 3).is_ok());
            }
            if exec(
                &mut output,
                "Running tests",
                &mut Command::new("cargo").arg("test").arg("--no-fail-fast"),
                "tests",
                4,
            )
            .is_ok()
            {
                results.push(true);
                assert!(ok(&mut output, TEST_OK, 4).is_ok());
            } else {
                results.push(false);
                assert!(ko(&mut output, TEST_ERR, 4).is_ok());
            }
            if exec(
                &mut output,
                "Checking code format",
                &mut Command::new("cargo").arg("fmt").arg("--check"),
                "fmt",
                5,
            )
            .is_ok()
            {
                results.push(true);
                assert!(ok(&mut output, FORMAT_OK, 5).is_ok());
            } else {
                results.push(false);
                assert!(ko(&mut output, FORMAT_ERR, 5).is_ok());
            }
            return self.end(&mut output, results);
        }
        Err(Error::other("no cargo"))
    }

    pub fn end(&mut self, output: &mut Stdout, results: Vec<bool>) -> Result<(), Error> {
        assert!(execute!(output, Show, Print("\n\n")).is_ok());
        if results.contains(&false) {
            return Err(Error::other("zuu detect error"));
        }
        Ok(())
    }
    fn php(&mut self) -> Result<(), Error> {
        if Path::new("composer.json").is_file() {
            let mut results: (bool, bool, bool, bool, bool, bool) =
                (false, false, false, false, false, false);
            let mut output: Stdout = stdout();
            execute!(&mut output, Clear(ClearType::All)).expect("msg");
            if Command::new("composer")
                .arg("validate")
                .arg("--strict")
                .stderr(File::create("zuu/stderr/validate")?)
                .stdout(File::create("zuu/stdout/validate")?)
                .current_dir(".")
                .spawn()
                .expect("composer")
                .wait()
                .expect("wait")
                .success()
            {
                results.0 = true;
                assert!(ok(&mut output, "No composer problem founded", 1).is_ok());
            } else {
                assert!(ko(&mut output, "Composer validate detect problem", 1).is_ok());
                results.0 = false;
            }
            if Command::new("composer")
                .arg("diagnose")
                .stderr(File::create("zuu/stderr/diagnose")?)
                .stdout(File::create("zuu/stdout/diagnose")?)
                .current_dir(".")
                .spawn()
                .expect("cargo")
                .wait()
                .expect("wait")
                .success()
            {
                results.1 = true;
                assert!(ok(&mut output, "Diagnose no detect problem", 2).is_ok());
            } else {
                results.1 = false;
                assert!(ko(&mut output, "Diagnose detect problem", 2).is_ok());
            }
            if Command::new("composer")
                .arg("audit")
                .stderr(File::create("zuu/stderr/audit")?)
                .stdout(File::create("zuu/stdout/audit")?)
                .current_dir(".")
                .spawn()
                .expect("cargo")
                .wait()
                .expect("wait")
                .success()
            {
                results.2 = true;
                assert!(ok(&mut output, "No audit errors founded", 3).is_ok());
            } else {
                results.2 = false;
                assert!(ko(&mut output, "Audit errors has been founded", 3).is_ok());
            }

            if Command::new("composer")
                .arg("test")
                .stderr(File::create("zuu/stderr/tests")?)
                .stdout(File::create("zuu/stdout/tests")?)
                .current_dir(".")
                .spawn()
                .expect("test")
                .wait()
                .expect("wait")
                .success()
            {
                results.3 = true;
                assert!(ok(&mut output, "All tests passes", 4).is_ok());
            } else {
                results.3 = false;
                assert!(ko(&mut output, TEST_ERR, 4).is_ok());
            }
            if Command::new("composer")
                .arg("fmt")
                .stderr(File::create("zuu/stderr/fmt")?)
                .stdout(File::create("zuu/stdout/fmt")?)
                .current_dir(".")
                .spawn()
                .expect("composer")
                .wait()
                .expect("wait")
                .success()
            {
                results.4 = true;
                assert!(ok(&mut output, "Source code format respect stantard", 5).is_ok());
            } else {
                results.4 = false;
                assert!(ko(&mut output, FORMAT_ERR, 5).is_ok());
            }
            if Command::new("composer")
                .arg("outdated")
                .stderr(File::create("zuu/stderr/outdated")?)
                .stdout(File::create("zuu/stdout/outdated")?)
                .current_dir(".")
                .spawn()
                .expect("composer")
                .wait()
                .expect("wait")
                .success()
            {
                results.5 = true;
                assert!(ok(&mut output, "Dependencies are up to date", 6).is_ok());
            } else {
                results.5 = false;
                assert!(ko(&mut output, "Dependencies must be updated", 6).is_ok());
            }
            assert!(execute!(&mut output, Print("\n\n")).is_ok());
            if results.0 && results.1 && results.2 && results.3 && results.4 && results.5 {
                return Ok(());
            }
            return Err(Error::other("zuu detect error"));
        }
        Err(Error::new(std::io::ErrorKind::NotFound, "no composer.json"))
    }

    fn js(&mut self) -> Result<(), Error> {
        if Path::new("package.json").is_file() {
            let mut results: (bool, bool, bool, bool, bool, bool, bool) =
                (false, false, false, false, false, false, false);
            let mut output: Stdout = stdout();
            execute!(&mut output, Clear(ClearType::All)).expect("msg");
            if Command::new("npm")
                .arg("audit")
                .stderr(File::create("zuu/stderr/audit")?)
                .stdout(File::create("zuu/stdout/audit")?)
                .current_dir(".")
                .spawn()
                .expect("npm")
                .wait()
                .expect("wait")
                .success()
            {
                results.0 = true;
                assert!(ok(&mut output, "No vulnerabilities founded", 1).is_ok());
            } else {
                assert!(ko(&mut output, "Audit detect vulnerabilities", 1).is_ok());
                results.0 = false;
            }
            if Command::new("npm")
                .arg("outdated")
                .stderr(File::create("zuu/stderr/outdated")?)
                .stdout(File::create("zuu/stdout/outdated")?)
                .current_dir(".")
                .spawn()
                .expect("npm")
                .wait()
                .expect("wait")
                .success()
            {
                results.1 = true;
                assert!(ok(&mut output, "All dependencies are up to date", 2).is_ok());
            } else {
                results.1 = false;
                assert!(ko(&mut output, "Dependencies must be updated", 2).is_ok());
            }
            if Command::new("npm")
                .arg("run")
                .arg("licenses")
                .stderr(File::create("zuu/stderr/licenses")?)
                .stdout(File::create("zuu/stdout/licenses")?)
                .current_dir(".")
                .spawn()
                .expect("cargo")
                .wait()
                .expect("wait")
                .success()
            {
                results.2 = true;
                assert!(ok(&mut output, "No licenses errors founded", 3).is_ok());
            } else {
                results.2 = false;
                assert!(ko(&mut output, "Licenses errors has been founded", 3).is_ok());
            }
            if Command::new("npm")
                .arg("run")
                .arg("lint")
                .stderr(File::create("zuu/stderr/lint")?)
                .stdout(File::create("zuu/stdout/lint")?)
                .current_dir(".")
                .spawn()
                .expect("npm")
                .wait()
                .expect("wait")
                .success()
            {
                results.3 = true;
                assert!(ok(&mut output, "No lint errors founded", 4).is_ok());
            } else {
                results.3 = false;
                assert!(ko(&mut output, "Lint errors has been founded", 4).is_ok());
            }

            if Command::new("npm")
                .arg("test")
                .stderr(File::create("zuu/stderr/tests")?)
                .stdout(File::create("zuu/stdout/tests")?)
                .current_dir(".")
                .spawn()
                .expect("test")
                .wait()
                .expect("wait")
                .success()
            {
                results.4 = true;
                assert!(ok(&mut output, "All tests passes", 5).is_ok());
            } else {
                results.4 = false;
                assert!(ko(&mut output, TEST_ERR, 5).is_ok());
            }
            if Command::new("npm")
                .arg("doctor")
                .stderr(File::create("zuu/stderr/doctor")?)
                .stdout(File::create("zuu/stdout/doctor")?)
                .current_dir(".")
                .spawn()
                .expect("npm")
                .wait()
                .expect("wait")
                .success()
            {
                results.5 = true;
                assert!(ok(&mut output, "The health of your npm environment is ok", 6).is_ok());
            } else {
                results.5 = false;
                assert!(ko(&mut output, "The health of your npm environment is bad", 6).is_ok());
            }
            if Command::new("npm")
                .arg("cache")
                .arg("verify")
                .stderr(File::create("zuu/stderr/cache")?)
                .stdout(File::create("zuu/stdout/cache")?)
                .current_dir(".")
                .spawn()
                .expect("npm")
                .wait()
                .expect("wait")
                .success()
            {
                results.6 = true;
                assert!(ok(
                    &mut output,
                    "The cache integrity of the cache index and all cached data are ok",
                    7
                )
                .is_ok());
            } else {
                results.6 = false;
                assert!(ko(
                    &mut output,
                    "The cache integrity of the cache index and all cached data have problem",
                    7
                )
                .is_ok());
            }
            assert!(execute!(&mut output, Print("\n\n")).is_ok());
            if results.0
                && results.1
                && results.2
                && results.3
                && results.4
                && results.5
                && results.6
            {
                return Ok(());
            }
            return Err(Error::other("zuu detect error"));
        }
        Err(Error::new(std::io::ErrorKind::NotFound, "no package.json"))
    }
    fn python(&mut self) -> Result<(), Error> {
        if Path::new("setup.py").is_file() {
            let mut results: (bool, bool, bool, bool, bool, bool, bool) =
                (false, false, false, false, false, false, false);
            let mut output: Stdout = stdout();
            if exec(
                &mut output,
                "Auditing code",
                &mut Command::new("bandit").arg("-r").arg("."),
                "audit",
                1,
            )
            .is_ok()
            {
                results.0 = true;
                assert!(ok(&mut output, "No vulnerabilities founded", 2).is_ok());
            } else {
                assert!(ko(&mut output, "Audit detect vulnerabilities", 2).is_ok());
                results.0 = false;
            }
            assert!(execute!(&mut output, Clear(ClearType::CurrentLine), Print("\n\n")).is_ok());
            if results.0 {
                return Ok(());
            }
            return Err(Error::other("zuu detect error"));
        }
        Err(Error::new(std::io::ErrorKind::NotFound, "no package.json"))
    }
    fn d(&mut self) -> Result<(), Error> {
        if Path::new("dub.json").is_file() || Path::new("dub.sdl").is_file() {
            let mut results: (bool, bool, bool) = (false, false, false);
            let mut output: Stdout = stdout();
            execute!(&mut output, Clear(ClearType::All)).expect("msg");
            if Command::new("dub")
                .arg("test")
                .stderr(File::create("zuu/stderr/test")?)
                .stdout(File::create("zuu/stdout/test")?)
                .current_dir(".")
                .spawn()
                .expect("dub")
                .wait()
                .expect("wait")
                .success()
            {
                results.0 = true;
                assert!(ok(&mut output, "All tests passes", 1).is_ok());
            } else {
                assert!(ko(&mut output, "Test have failures", 1).is_ok());
                results.0 = false;
            }
            if Command::new("dub")
                .arg("lint")
                .arg("--nodeps")
                .arg("--syntax-check")
                .stderr(File::create("zuu/stderr/syntax")?)
                .stdout(File::create("zuu/stdout/syntax")?)
                .current_dir(".")
                .spawn()
                .expect("dub")
                .wait()
                .expect("wait")
                .success()
            {
                results.1 = true;
                assert!(ok(&mut output, "The project respect the syntax", 2).is_ok());
            } else {
                results.1 = false;
                assert!(ko(&mut output, "The project has bad syntax", 2).is_ok());
            }
            if Command::new("dub")
                .arg("lint")
                .arg("--style-check")
                .arg("--nodeps")
                .stderr(File::create("zuu/stderr/style")?)
                .stdout(File::create("zuu/stdout/style")?)
                .current_dir(".")
                .spawn()
                .expect("dub")
                .wait()
                .expect("wait")
                .success()
            {
                results.2 = true;
                assert!(ok(
                    &mut output,
                    "The source style analysis no contains failures",
                    3
                )
                .is_ok());
            } else {
                results.2 = false;
                assert!(ko(
                    &mut output,
                    "The source style analysis contains failures",
                    3
                )
                .is_ok());
            }
            assert!(execute!(&mut output, Print("\n\n")).is_ok());
            if results.0 && results.1 && results.2 {
                return Ok(());
            }
            return Err(Error::other("zuu detect error"));
        }
        Err(Error::new(std::io::ErrorKind::NotFound, "no package.json"))
    }
    fn all(&mut self) -> Result<(), Error> {
        self.checked.insert(
            Checked::License,
            Command::new("make")
                .arg(TARGET_LICENSE)
                .current_dir(".")
                .spawn()
                .expect("license")
                .wait()
                .expect("wait")
                .success(),
        );
        self.checked.insert(
            Checked::Audit,
            Command::new("make")
                .arg(TARGET_AUDIT)
                .current_dir(".")
                .spawn()
                .expect("audit")
                .wait()
                .expect("wait")
                .success(),
        );

        self.checked.insert(
            Checked::Lint,
            Command::new("make")
                .arg(TARGET_LINT)
                .current_dir(".")
                .spawn()
                .expect("lint")
                .wait()
                .expect("wait")
                .success(),
        );
        self.checked.insert(
            Checked::Test,
            Command::new("make")
                .arg(TARGET_TEST)
                .current_dir(".")
                .spawn()
                .expect("test")
                .wait()
                .expect("wait")
                .success(),
        );
        self.checked.insert(
            Checked::Fmt,
            Command::new("make")
                .arg(TARGET_FMT)
                .current_dir(".")
                .spawn()
                .expect("fantomas")
                .wait()
                .expect("wait")
                .success(),
        );
        check(&self.checked)
    }

    pub fn check(&mut self) -> ExitCode {
        match self.language {
            Language::Rust => zuu_exit(&self.rust()),
            Language::Php => zuu_exit(&self.php()),
            Language::D => zuu_exit(&self.d()),
            Language::Python => zuu_exit(&self.python()),
            Language::Nodejs | Language::TypeScript => zuu_exit(&self.js()),
            Language::Unknown => ExitCode::FAILURE,
            _ => zuu_exit(&self.all()),
        }
    }
}