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
        self.check_in(None, canary)
    }

    /// The script run on this directory in `BUNDLE_MODE=mode` (`None`: the
    /// variable is not set, the site's bundle).
    fn check_in(&self, mode: Option<&str>, canary: Option<&str>) -> Output {
        let script =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts/check-wasm-no-token.sh");
        let mut command = Command::new("bash");
        command.arg(script).env("PKG_DIR", &self.0);
        command.env_remove("BUNDLE_MODE");
        if let Some(mode) = mode {
            command.env("BUNDLE_MODE", mode);
        }
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

/// 43 base62 characters, the body of a PaliCanon API token.
fn body(len: usize) -> String {
    "A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4Y5z6"[..len].to_owned()
}

/// A bundle holding one token-shaped string fails, whatever its environment
/// (`pc_dev_`, `pc_prod_`), in a `.js` and in a `.wasm`; the failure names
/// the file and "token shape", never the bytes.
#[test]
fn a_token_shaped_string_fails_the_check_without_being_printed() {
    for prefix in ["pc_dev_", "pc_prod_"] {
        let token = format!("{prefix}{}", body(43));
        let pkg = Pkg::with("shape", &format!("var t = '{token}';"));
        let out = pkg.check(None);
        assert_eq!(out.status.code(), Some(1), "{prefix}: {}", stderr(&out));
        assert!(stderr(&out).contains("token shape"), "{}", stderr(&out));
        assert!(stderr(&out).contains("a.js"), "{}", stderr(&out));
        for text in [
            stderr(&out),
            String::from_utf8_lossy(&out.stdout).into_owned(),
        ] {
            assert!(!text.contains(&body(20)), "{prefix}: the token is printed");
        }
    }
    let dir = std::env::temp_dir().join(format!("check-wasm-script-shape-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a temp dir");
    let wasm = [
        b"\0asm\x01\0\0\0".as_slice(),
        format!("pc_dev_{}", body(43)).as_bytes(),
    ]
    .concat();
    std::fs::write(dir.join("app.wasm"), wasm).expect("wasm written");
    let pkg = Pkg(dir);
    let out = pkg.check(None);
    assert_eq!(out.status.code(), Some(1), "wasm: {}", stderr(&out));
    assert!(stderr(&out).contains("app.wasm"), "{}", stderr(&out));
}

/// What is not a token's shape passes: one character short, another
/// environment, a character outside base62 inside the body.
#[test]
fn near_misses_of_the_token_shape_pass() {
    let dashed = format!("pc_dev_{}-{}", body(20), body(22));
    let cases = [
        ("42 characters", format!("pc_dev_{}", body(42))),
        ("other environment", format!("pc_stage_{}", body(43))),
        ("no prefix", body(43)),
        ("a dash inside", dashed),
    ];
    for (case, content) in cases {
        let pkg = Pkg::with("near", &format!("var t = '{content}';"));
        let out = pkg.check(None);
        assert_eq!(out.status.code(), Some(0), "{case}: {}", stderr(&out));
    }
}

/// Whether `grep -E pattern` matches `text`.
fn shape_matches(pattern: &str, text: &str) -> bool {
    use std::io::Write as _;
    let mut grep = Command::new("grep")
        .args(["-Eq", pattern])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("grep runs");
    let mut stdin = grep.stdin.take().expect("stdin");
    stdin.write_all(text.as_bytes()).expect("written");
    drop(stdin);
    grep.wait().expect("grep ends").success()
}

/// The shape lives in `scripts/token-pattern`, one line, read by the
/// script: changing that file changes the verdict.
#[test]
fn the_script_reads_the_shape_from_token_pattern() {
    let scripts = Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts");
    let pattern = std::fs::read_to_string(scripts.join("token-pattern")).expect("token-pattern");
    assert_eq!(pattern.trim_end().lines().count(), 1, "one line: {pattern}");
    let token = format!("pc_dev_{}", body(43));
    assert!(
        shape_matches(pattern.trim_end(), &token),
        "the file's pattern matches a token: {pattern}"
    );

    let dir = std::env::temp_dir().join(format!("check-wasm-pattern-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a temp dir");
    std::fs::copy(scripts.join("check-wasm-no-token.sh"), dir.join("check.sh")).expect("copy");
    std::fs::write(dir.join("token-pattern"), "zzz[0-9]{3}\n").expect("pattern");
    let run = |content: &str| {
        let pkg = Pkg::with("pattern", content);
        Command::new("bash")
            .arg(dir.join("check.sh"))
            .env("PKG_DIR", &pkg.0)
            .output()
            .expect("bash runs the copy")
    };
    assert_eq!(
        run("a zzz123 b").status.code(),
        Some(1),
        "the new shape fails"
    );
    assert_eq!(
        run(&token).status.code(),
        Some(0),
        "the old shape passes now"
    );
    std::fs::remove_file(dir.join("token-pattern")).expect("remove the pattern");
    let out = run("clean");
    assert_eq!(
        out.status.code(),
        Some(2),
        "no pattern file: {}",
        stderr(&out)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// The server function's endpoint, which the site's hydrate bundle holds
/// (it is how the browser searches) and the app's csr bundle must not.
const ENDPOINT: &str = "/api/search_passages";

/// Task 021: the endpoint string fails a csr bundle (`BUNDLE_MODE=csr`) and
/// names the file and the needle; the site's own bundle, where it is
/// legitimate, passes.
#[test]
fn the_endpoint_fails_a_csr_bundle_but_not_the_site_s() {
    let pkg = Pkg::with("endpoint", &format!("fetch('{ENDPOINT}');"));
    let site = pkg.check(None);
    assert_eq!(site.status.code(), Some(0), "site: {}", stderr(&site));
    let site = pkg.check_in(Some("site"), None);
    assert_eq!(site.status.code(), Some(0), "site: {}", stderr(&site));
    let csr = pkg.check_in(Some("csr"), None);
    assert_eq!(csr.status.code(), Some(1), "csr: {}", stderr(&csr));
    assert!(stderr(&csr).contains(ENDPOINT), "{}", stderr(&csr));
    assert!(stderr(&csr).contains("a.js"), "{}", stderr(&csr));
}

/// A csr bundle is checked for every needle of the site's, plus the
/// endpoint; a clean one passes, with the success line naming the mode.
#[test]
fn a_csr_bundle_keeps_every_needle_and_a_clean_one_passes() {
    for needle in NEEDLES {
        let pkg = Pkg::with("csr-needle", &format!("before {needle} after"));
        let out = pkg.check_in(Some("csr"), None);
        assert_eq!(out.status.code(), Some(1), "{needle}: {}", stderr(&out));
        assert!(stderr(&out).contains(needle), "{needle}: {}", stderr(&out));
    }
    let pkg = Pkg::with("csr-clean", "console.log('hello');");
    let out = pkg.check_in(Some("csr"), None);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
}

/// A mode the script does not know, or an empty one (a CI variable that
/// expanded to nothing), is exit 2, never a silent pass of the site's needles
/// on an app's bundle. Only an unset `BUNDLE_MODE` is the site.
#[test]
fn an_empty_or_unknown_bundle_mode_is_exit_two() {
    let pkg = Pkg::with("mode", &format!("fetch('{ENDPOINT}');"));
    for mode in ["cs", "", "CSR", "csr "] {
        let out = pkg.check_in(Some(mode), None);
        assert_eq!(out.status.code(), Some(2), "{mode:?}: {}", stderr(&out));
        assert!(stderr(&out).contains("BUNDLE_MODE"), "{}", stderr(&out));
    }
    let out = pkg.check_in(None, None);
    assert_eq!(
        out.status.code(),
        Some(0),
        "unset is the site: {}",
        stderr(&out)
    );
}
