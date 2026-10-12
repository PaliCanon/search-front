//! `scripts/check-wasm-no-token.sh` as a process: the guard that no token,
//! no API client and no mock reaches the browser bundle (task 005, review
//! round 2, T3). The script is the check's only copy of its needles, so each
//! needle has a bundle that must fail it, and a clean bundle must pass.
#![cfg(feature = "ssr")]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Every literal the script refuses in the bundle, with no canary.
const NEEDLES: [&str; 6] = [
    "SEARCH_API_TOKEN",
    "palicanon_api_client",
    "MockSource",
    "__search_front_mock_cold__",
    "__search_front_mock_error__",
    "__search_front_mock_rate_limited__",
];

/// A bundle directory holding one `a.js`, removed when dropped.
struct Pkg(PathBuf);

impl Pkg {
    /// A fresh directory whose `a.js` holds `content`.
    fn with(name: &str, content: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("check-wasm-script-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a temp dir");
        std::fs::write(dir.join("a.js"), content).expect("a.js written");
        Pkg(dir)
    }

    /// A fresh directory with no `.js` or `.wasm` at all.
    fn empty(name: &str) -> Self {
        let pkg = Pkg::with(name, "");
        std::fs::remove_file(pkg.0.join("a.js")).expect("a.js removed");
        pkg
    }

    /// The script run on this directory, with an optional canary as `$1`.
    fn check(&self, canary: Option<&str>) -> Output {
        let script =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts/check-wasm-no-token.sh");
        let mut command = Command::new("bash");
        command.arg(script).env("PKG_DIR", &self.0);
        if let Some(canary) = canary {
            command.arg(canary);
        }
        command.output().expect("bash runs the script")
    }
}

impl Drop for Pkg {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The script's stderr as text.
fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// A bundle holding the in-repo mock's type name fails the check: the mock
/// is compiled into the WASM (review 005, Δ1).
#[test]
fn a_bundle_holding_the_mock_source_fails() {
    let pkg = Pkg::with("mock", "var x = 'MockSource';");
    let out = pkg.check(None);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("MockSource"), "{}", stderr(&out));
    assert!(stderr(&out).contains("a.js"), "{}", stderr(&out));
}

/// Every needle fails the check on its own.
#[test]
fn every_needle_fails_the_check() {
    for needle in NEEDLES {
        let pkg = Pkg::with("needle", &format!("before {needle} after"));
        let out = pkg.check(None);
        assert_eq!(out.status.code(), Some(1), "{needle}: {}", stderr(&out));
        assert!(stderr(&out).contains(needle), "{needle}: {}", stderr(&out));
    }
}

/// A clean bundle passes, with and without a canary that is not in it.
#[test]
fn a_clean_bundle_passes() {
    let pkg = Pkg::with("clean", "console.log('hello');");
    let out = pkg.check(None);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let out = pkg.check(Some("pc_canary_not_in_the_bundle"));
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
}

/// A canary passed as `$1` and found in the bundle fails the check, and the
/// failure names "the canary", never its value.
#[test]
fn a_planted_canary_fails_without_being_printed() {
    let canary = "pc_canary_planted_9d2f";
    let pkg = Pkg::with("canary", &format!("var t = '{canary}';"));
    let out = pkg.check(Some(canary));
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("the canary"), "{}", stderr(&out));
    assert!(!stderr(&out).contains(canary), "{}", stderr(&out));
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains(canary),
        "stdout"
    );
}

/// No bundle to check is exit 2, not a pass.
#[test]
fn a_directory_with_no_bundle_is_exit_two() {
    let pkg = Pkg::empty("none");
    let out = pkg.check(None);
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
}

/// T3 (review 005, round 3): the script checks WASM files too: a bundle
/// holding `app.wasm` that contains a forbidden needle fails with exit 1,
/// with the needle and the file name in stderr.
#[test]
fn a_bundle_holding_the_mock_in_wasm_fails() {
    let dir = std::env::temp_dir().join(format!("check-wasm-script-wasm-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a temp dir");
    std::fs::write(dir.join("app.wasm"), "MockSource binary content").expect("wasm written");
    let pkg = Pkg(dir);
    let out = pkg.check(None);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("MockSource"), "{}", stderr(&out));
    assert!(stderr(&out).contains("app.wasm"), "{}", stderr(&out));
}
