//! `scripts/smoke.sh` and `scripts/rollout-check.sh` as processes (task 007,
//! slice S3): against the real binary and against a fake server that plays
//! every way a deployment can be wrong. A check that passes only on the
//! happy path proves nothing, so each check has a server that must fail it,
//! and the failure names the check. The origin secret is in no output, no
//! argument, and the token shape is never printed.
#![cfg(feature = "ssr")]

use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The secret the service and the scripts share in these tests.
const SECRET: &str = "origin_canary_scripts_c40e17";
/// A token-shaped string (`pc_dev_` + 43 base62): the scripts must refuse it.
const TOKEN: &str = "pc_dev_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v";

fn script(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../scripts")
        .join(name)
}

/// A free port, never handed out twice by this test binary.
fn free_port() -> u16 {
    static LAST: AtomicU16 = AtomicU16::new(0);
    loop {
        let port = TcpListener::bind("127.0.0.1:0")
            .expect("a free port")
            .local_addr()
            .expect("an address")
            .port();
        if LAST.swap(port, Ordering::SeqCst) != port {
            return port;
        }
    }
}

// ---------------------------------------------------------------- fake server

/// A request as the fake server sees it.
struct Req {
    path: String,
    headers: Vec<(String, String)>,
}

impl Req {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// Whether the request carries the right secret.
    fn authed(&self) -> bool {
        self.header("x-origin-auth") == Some(SECRET)
    }
}

/// A response the fake server sends.
struct Resp {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

impl Resp {
    fn new(status: u16, body: &str) -> Self {
        Resp {
            status,
            headers: Vec::new(),
            body: body.to_owned(),
        }
    }

    fn with(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }
}

/// A server on a local port answering with `handler`; it keeps every
/// request it received.
struct Fake {
    port: u16,
    seen: Arc<Mutex<Vec<Req>>>,
}

impl Fake {
    fn start(handler: impl Fn(&Req) -> Resp + Send + 'static) -> Fake {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a free port");
        let port = listener.local_addr().expect("an address").port();
        let seen: Arc<Mutex<Vec<Req>>> = Arc::new(Mutex::new(Vec::new()));
        let kept = Arc::clone(&seen);
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                answer(stream, &handler, &kept);
            }
        });
        Fake { port, seen }
    }

    fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// How many requests carried the secret.
    fn authed_requests(&self) -> usize {
        self.seen
            .lock()
            .expect("lock")
            .iter()
            .filter(|r| r.authed())
            .count()
    }
}

fn answer(mut stream: TcpStream, handler: &impl Fn(&Req) -> Resp, seen: &Mutex<Vec<Req>>) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut raw = Vec::new();
    let mut buf = [0u8; 2048];
    while !raw.windows(4).any(|w| w == b"\r\n\r\n") {
        match stream.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => raw.extend_from_slice(&buf[..n]),
        }
    }
    let text = String::from_utf8_lossy(&raw).into_owned();
    let mut lines = text.lines();
    let path = lines
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or("/")
        .to_owned();
    let headers = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(n, v)| (n.trim().to_ascii_lowercase(), v.trim().to_owned()))
        .collect();
    let req = Req { path, headers };
    let resp = handler(&req);
    seen.lock().expect("lock").push(req);
    let mut head = format!(
        "HTTP/1.1 {} X\r\ncontent-length: {}\r\nconnection: close\r\n",
        resp.status,
        resp.body.len()
    );
    for (n, v) in &resp.headers {
        let _ = write!(head, "{n}: {v}\r\n");
    }
    let _ = stream.write_all(format!("{head}\r\n{}", resp.body).as_bytes());
}

const WASM: &str = "/pkg/search-front.4f2a9c.wasm";

/// The release the fake deployment serves (task 028): a branch build, as
/// `scripts/release-values.sh` prints it for `main`.
const EXPECT_ENV: &str = "main";
const EXPECT_SHA: &str = "34c489c";

/// The two metas the real service renders in the head (task 028).
const RELEASE_METAS: &str = "<meta name=\"search-release-env\" content=\"main\">\
                             <meta name=\"search-release-sha\" content=\"34c489c\">";

/// The page the real service renders, as far as the scripts read it, with
/// `head` after the title (the release metas live there).
fn page_with_head(head: &str, extra: &str) -> String {
    format!(
        "<!DOCTYPE html><html><head><title>PaliCanon — Search</title>{head}\
         <link rel=\"preload\" href=\"{WASM}\" as=\"fetch\" type=\"application/wasm\">\
         </head><body>{extra}</body></html>"
    )
}

/// The page with the right release metas.
fn page(extra: &str) -> String {
    page_with_head(RELEASE_METAS, extra)
}

const RESULTS: &str = "<ol class=\"divide-y\"><li>MN 10</li></ol>";

/// The engine starting page (the `ColdState` component renders with role="alert").
const STARTING: &str = "<div role=\"alert\" class=\"mt-10 max-w-[60ch] space-y-2\">\
                         <p>The engine is starting</p>\
                         </div>";

/// A healthy deployment: refuses a request without the secret, serves the
/// page, the wasm and a result list to one with it.
fn healthy(req: &Req) -> Resp {
    if !req.authed() {
        return Resp::new(403, "").with("cache-control", "no-store");
    }
    if req.path == WASM {
        return Resp::new(200, "\0asm").with("content-type", "application/wasm");
    }
    if req.path.starts_with("/?q=") && req.path.len() > 4 {
        return Resp::new(200, &page(RESULTS)).with("content-type", "text/html");
    }
    if req.path == "/" || req.path.starts_with("/?q") {
        return Resp::new(200, &page("")).with("content-type", "text/html");
    }
    Resp::new(404, "")
}

// ------------------------------------------------------------------ the binary

/// The real binary, killed when dropped.
struct Service {
    child: Child,
    port: u16,
}

impl Drop for Service {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// A site root holding a fake wasm at `/pkg/search-front_bg.wasm`.
fn site_root(name: &str, with_wasm: bool) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("site-scripts-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("pkg")).expect("a site root");
    if with_wasm {
        std::fs::write(dir.join("pkg/search-front_bg.wasm"), b"\0asm\x01\0\0\0").expect("wasm");
    }
    dir
}

/// The service's command: the mock source, the given site root.
fn service_command(binary: &Path, site: &Path, port: Option<u16>) -> Command {
    let mut command = Command::new(binary);
    command
        .env_remove("SEARCH_API_URL")
        .env_remove("SEARCH_API_TOKEN")
        .env_remove("ORIGIN_SECRET")
        .env_remove("LEPTOS_ENV")
        .env_remove("K_SERVICE")
        .env("LEPTOS_OUTPUT_NAME", "search-front")
        .env("LEPTOS_SITE_ROOT", site)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(port) = port {
        command.env("LEPTOS_SITE_ADDR", format!("127.0.0.1:{port}"));
    }
    command
}

/// Starts the real service on the mock, checking `secret` when given.
fn start_service(site: &Path, secret: Option<&str>) -> Service {
    let port = free_port();
    let mut command = service_command(
        Path::new(env!("CARGO_BIN_EXE_search_front")),
        site,
        Some(port),
    );
    if let Some(secret) = secret {
        command.env("ORIGIN_SECRET", secret);
    }
    let child = command.spawn().expect("the binary starts");
    let mut service = Service { child, port };
    let deadline = Instant::now() + Duration::from_secs(30);
    while TcpStream::connect(("127.0.0.1", port)).is_err() {
        assert!(
            service.child.try_wait().expect("status").is_none(),
            "stopped"
        );
        assert!(Instant::now() < deadline, "never listened");
        std::thread::sleep(Duration::from_millis(50));
    }
    service
}

// ------------------------------------------------------------------- helpers

/// Runs bash on `script` with `args` and the environment of a CI step.
fn run(script: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut command = Command::new("bash");
    command
        .arg(script)
        .args(args)
        .env_remove("ORIGIN_SECRET")
        .env_remove("ROLLOUT_EXPECT_ENV")
        .env_remove("ROLLOUT_EXPECT_SHA")
        .env("SMOKE_WAIT", "8");
    for (name, value) in env {
        command.env(name, value);
    }
    command.output().expect("bash runs the script")
}

/// `env` plus the intended release (`main`, `34c489c`): what the deploy
/// step hands `rollout-check.sh` in direct mode (task 028).
fn with_release<'a>(env: &[(&'a str, &'a str)]) -> Vec<(&'a str, &'a str)> {
    let mut all = env.to_vec();
    all.push(("ROLLOUT_EXPECT_ENV", EXPECT_ENV));
    all.push(("ROLLOUT_EXPECT_SHA", EXPECT_SHA));
    all
}

/// Everything the script printed.
fn printed(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// The script failed (exit 1), named `check`, and printed no secret, no
/// token shape.
fn assert_fails_naming(output: &Output, check: &str, case: &str) {
    let text = printed(output);
    assert_eq!(output.status.code(), Some(1), "{case}: {text}");
    assert!(text.contains(check), "{case}: names {check}: {text}");
    assert!(!text.contains(SECRET), "{case}: secret printed: {text}");
    assert!(!text.contains(TOKEN), "{case}: token printed: {text}");
}

// ------------------------------------------------------------------- smoke.sh

#[test]
fn smoke_passes_on_the_real_binary_and_leaves_no_service_behind() {
    let site = site_root("smoke-ok", true);
    let binary = env!("CARGO_BIN_EXE_search_front");
    let out = run(
        &script("smoke.sh"),
        &[binary],
        &[
            ("LEPTOS_OUTPUT_NAME", "search-front"),
            ("LEPTOS_SITE_ROOT", site.to_str().expect("utf-8")),
        ],
    );
    let text = printed(&out);
    assert_eq!(out.status.code(), Some(0), "{text}");
    for check in ["root", "guard", "wasm"] {
        assert!(text.contains(check), "{check} is reported: {text}");
    }
    let _ = std::fs::remove_dir_all(site);
}

#[test]
fn smoke_fails_naming_the_wasm_when_it_is_not_served() {
    let site = site_root("smoke-nowasm", false);
    let binary = env!("CARGO_BIN_EXE_search_front");
    let out = run(
        &script("smoke.sh"),
        &[binary],
        &[
            ("LEPTOS_OUTPUT_NAME", "search-front"),
            ("LEPTOS_SITE_ROOT", site.to_str().expect("utf-8")),
        ],
    );
    assert_fails_naming(&out, "wasm", "no wasm in the site root");
    let _ = std::fs::remove_dir_all(site);
}

#[test]
fn smoke_fails_when_the_binary_dies_at_boot() {
    let out = run(&script("smoke.sh"), &["/usr/bin/false"], &[]);
    assert_fails_naming(&out, "listen", "a binary that exits");
}

/// `smoke.sh` and a `token-pattern` with this text (`None`: no such file) in a
/// fresh folder, as the image's runtime stage holds them (`/app/smoke.sh`).
fn smoke_beside(name: &str, pattern: Option<&str>) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("search-front-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a folder");
    std::fs::copy(script("smoke.sh"), dir.join("smoke.sh")).expect("copy smoke.sh");
    if let Some(text) = pattern {
        std::fs::write(dir.join("token-pattern"), text).expect("write token-pattern");
    }
    dir
}

/// Task 007 F1: the runtime stage lacked `token-pattern`, `head` printed an
/// error, the pattern was empty and `grep -E ''` matched every response: the
/// smoke test said "a token shape" about a healthy page. A missing, empty or
/// blank pattern must fail loudly, naming the file, before anything starts.
#[test]
fn smoke_refuses_a_missing_or_empty_token_pattern() {
    let binary = env!("CARGO_BIN_EXE_search_front");
    for (case, pattern) in [
        ("missing", None),
        ("empty", Some("")),
        ("a blank line", Some("\n")),
        ("blanks", Some("  \n\n")),
    ] {
        let dir = smoke_beside(
            &format!("smoke-pattern-{}", case.replace(' ', "-")),
            pattern,
        );
        let out = run(&dir.join("smoke.sh"), &[binary], &[]);
        let text = printed(&out);
        assert_eq!(out.status.code(), Some(2), "{case}: {text}");
        assert!(
            text.contains("token-pattern"),
            "{case}: names the file: {text}"
        );
        assert!(
            !text.contains("FAIL token"),
            "{case}: must not blame a response: {text}"
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// The same refusal in `--port` mode (nothing listens: the pattern is read
/// first, so the answer is about the pattern, not the port).
#[test]
fn smoke_against_a_server_refuses_a_missing_token_pattern_too() {
    let dir = smoke_beside("smoke-pattern-port", None);
    let out = run(
        &dir.join("smoke.sh"),
        &["--port", "9"],
        &[("ORIGIN_SECRET", SECRET)],
    );
    let text = printed(&out);
    assert_eq!(out.status.code(), Some(2), "{text}");
    assert!(text.contains("token-pattern"), "{text}");
    let _ = std::fs::remove_dir_all(dir);
}

/// A fake server that breaks one check of `smoke.sh`, with the check it
/// breaks and what is wrong.
type Handler = Box<dyn Fn(&Req) -> Resp + Send>;
type Case = (&'static str, &'static str, Handler);

/// All of them.
fn smoke_cases() -> Vec<Case> {
    let mut cases = smoke_status_cases();
    cases.extend(smoke_token_cases());
    cases
}

fn smoke_status_cases() -> Vec<Case> {
    vec![
        (
            "root",
            "/ is a 500",
            Box::new(|r| {
                if r.authed() && r.path == "/" {
                    Resp::new(500, "boom")
                } else {
                    healthy(r)
                }
            }),
        ),
        (
            "guard",
            "no header answers 200",
            Box::new(|r| {
                if r.path == "/" {
                    Resp::new(200, &page("")).with("content-type", "text/html")
                } else {
                    healthy(r)
                }
            }),
        ),
        (
            "guard",
            "a wrong header answers 200",
            Box::new(|r| {
                if r.header("x-origin-auth").is_some_and(|v| v != SECRET) {
                    Resp::new(200, &page(""))
                } else {
                    healthy(r)
                }
            }),
        ),
        (
            "wasm",
            "the wasm is a 404",
            Box::new(|r| {
                if r.path == WASM {
                    Resp::new(404, "")
                } else {
                    healthy(r)
                }
            }),
        ),
    ]
}

fn smoke_token_cases() -> Vec<Case> {
    vec![
        (
            "wasm",
            "the wasm is served as text",
            Box::new(|r| {
                if r.path == WASM && r.authed() {
                    Resp::new(200, "<html>").with("content-type", "text/html")
                } else {
                    healthy(r)
                }
            }),
        ),
        (
            "token",
            "the page holds a token shape",
            Box::new(|r| {
                if r.authed() && r.path == "/" {
                    Resp::new(200, &page(TOKEN)).with("content-type", "text/html")
                } else {
                    healthy(r)
                }
            }),
        ),
    ]
}

/// `smoke.sh --port` against a fake server: the passing one, then one that
/// breaks each check in turn.
#[test]
fn smoke_against_a_server_names_the_check_that_failed() {
    let env = [("ORIGIN_SECRET", SECRET)];
    let good = Fake::start(healthy);
    let out = run(
        &script("smoke.sh"),
        &["--port", &good.port.to_string()],
        &env,
    );
    assert_eq!(out.status.code(), Some(0), "healthy: {}", printed(&out));

    for (check, case, handler) in smoke_cases() {
        let fake = Fake::start(handler);
        let out = run(
            &script("smoke.sh"),
            &["--port", &fake.port.to_string()],
            &env,
        );
        assert_fails_naming(&out, check, case);
    }
}

#[test]
fn smoke_against_a_server_needs_the_secret_in_the_environment() {
    let fake = Fake::start(healthy);
    let out = run(
        &script("smoke.sh"),
        &["--port", &fake.port.to_string()],
        &[],
    );
    assert_eq!(out.status.code(), Some(2), "{}", printed(&out));
    assert!(printed(&out).contains("ORIGIN_SECRET"), "{}", printed(&out));
}

// ---------------------------------------------------------- rollout-check.sh

fn check_script() -> PathBuf {
    script("rollout-check.sh")
}

/// A header file as the CI step writes it: `x-origin-auth: <secret>`.
fn header_file(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("rollout-header-{}-{name}", std::process::id()));
    std::fs::write(&path, format!("x-origin-auth: {SECRET}\n")).expect("a header file");
    path
}

#[test]
fn rollout_check_passes_on_the_real_binary_in_each_mode() {
    let site = site_root("rollout-ok", true);
    let checked = start_service(&site, Some(SECRET));
    let url = format!("http://127.0.0.1:{}", checked.port);

    // The service with no `SEARCH_FRONT_ENV` is a `dev` build with no sha
    // (task 028). This is the one test that stays red on the lane's branch
    // until the Rust side (lane A) renders the metas.
    let release = [("ROLLOUT_EXPECT_ENV", "dev"), ("ROLLOUT_EXPECT_SHA", "")];
    let mut env = vec![("ORIGIN_SECRET", SECRET)];
    env.extend(release);
    let out = run(&check_script(), &[&url], &env);
    let text = printed(&out);
    assert_eq!(
        out.status.code(),
        Some(0),
        "secret from the environment: {text}"
    );
    for check in ["root", "release", "wasm", "search", "token", "guard"] {
        assert!(text.contains(check), "{check} is reported: {text}");
    }

    let file = header_file("real");
    let file_arg = file.to_str().expect("utf-8");
    let out = run(&check_script(), &[&url, file_arg], &release);
    assert_eq!(
        out.status.code(),
        Some(0),
        "secret from a file: {}",
        printed(&out)
    );

    let out = run(&check_script(), &["--refused", &url], &[]);
    assert_eq!(out.status.code(), Some(0), "refused: {}", printed(&out));

    // Through the Worker the service is reached without the header: here a
    // service that checks nothing stands for it.
    let open = start_service(&site, None);
    let open_url = format!("http://127.0.0.1:{}", open.port);
    let chrome = FakeChrome::new("real-binary");
    let out = run(
        &check_script(),
        &["--via-worker", &open_url],
        &chrome.env("ok"),
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "via the Worker: {}",
        printed(&out)
    );
    assert!(
        printed(&out).contains("hydrate ok"),
        "the Worker mode checks hydration: {}",
        printed(&out)
    );
    let _ = std::fs::remove_file(file);
    let _ = std::fs::remove_dir_all(site);
}

/// A fake server that breaks one check of `rollout-check.sh`.
/// A server that plants a token shape in `path`'s page, or in its headers.
fn planted(path: &'static str, in_header: bool) -> Handler {
    Box::new(move |r| {
        let resp = healthy(r);
        if !r.authed() || r.path != path {
            return resp;
        }
        if in_header {
            resp.with("x-debug", TOKEN)
        } else {
            Resp::new(200, &page(&format!("{RESULTS}{TOKEN}"))).with("content-type", "text/html")
        }
    })
}

/// All of them.
fn rollout_cases() -> Vec<Case> {
    let mut cases = rollout_status_cases();
    cases.extend(rollout_token_cases());
    cases
}

/// A server that answers `path`, to a caller with the secret, with `status`
/// and `body`, and is healthy for the rest.
fn answering(path: &'static str, status: u16, body: &str) -> Handler {
    let body = body.to_owned();
    Box::new(move |r| {
        if r.authed() && r.path == path {
            Resp::new(status, &body)
        } else {
            healthy(r)
        }
    })
}

/// A server that answers `path` with the starting state `times` times,
/// then with `final_body`.
fn starting_then(path: &'static str, times: u16, final_body: String) -> Handler {
    let count: Arc<Mutex<u16>> = Arc::new(Mutex::new(0));
    Box::new(move |r| {
        if !r.authed() || r.path != path {
            return healthy(r);
        }
        let mut c = count.lock().expect("lock");
        *c += 1;
        if *c <= times {
            Resp::new(200, STARTING).with("content-type", "text/html")
        } else {
            Resp::new(200, &final_body).with("content-type", "text/html")
        }
    })
}

const SEARCH: &str = "/?q=satipatthana";

fn rollout_status_cases() -> Vec<Case> {
    vec![
        ("root", "/ is a 500", answering("/", 500, "boom")),
        (
            "root",
            "/ does not name the page",
            answering("/", 200, "<html><title>nginx</title></html>"),
        ),
        ("wasm", "the wasm is a 404", answering(WASM, 404, "")),
        (
            "search",
            "a search answers without a result list",
            answering(SEARCH, 200, &page("<p>No passage matches</p>")),
        ),
        ("search", "a search is a 502", answering(SEARCH, 502, "")),
    ]
}

fn rollout_token_cases() -> Vec<Case> {
    vec![
        (
            "token",
            "a token shape in the home page",
            planted("/", false),
        ),
        (
            "token",
            "a token shape in a search page",
            planted("/?q=satipatthana", false),
        ),
        (
            "token",
            "a token shape in a home header",
            planted("/", true),
        ),
        (
            "token",
            "a token shape in a search header",
            planted("/?q=satipatthana", true),
        ),
        (
            "guard",
            "no header answers 200",
            Box::new(|r| {
                if r.path == "/" {
                    Resp::new(200, &page("")).with("content-type", "text/html")
                } else {
                    healthy(r)
                }
            }),
        ),
    ]
}

/// Each way a rollout can be wrong has a server that must fail the check
/// that guards it, and the failure names that check.
#[test]
fn rollout_check_names_the_check_that_failed() {
    for (check, case, handler) in rollout_cases() {
        let fake = Fake::start(handler);
        let out = run(
            &check_script(),
            &[&fake.url()],
            &with_release(&[("ORIGIN_SECRET", SECRET)]),
        );
        assert_fails_naming(&out, check, case);
    }
}

#[test]
fn rollout_check_demands_a_secret_and_refuses_to_send_it_to_the_worker() {
    let fake = Fake::start(healthy);
    let out = run(&check_script(), &[&fake.url()], &with_release(&[]));
    assert_eq!(out.status.code(), Some(2), "no secret: {}", printed(&out));
    let file = header_file("worker");
    let file_arg = file.to_str().expect("utf-8");
    let out = run(
        &check_script(),
        &["--via-worker", &fake.url(), file_arg],
        &[],
    );
    assert_eq!(out.status.code(), Some(2), "{}", printed(&out));
    let out = run(
        &check_script(),
        &["--via-worker", &fake.url()],
        &[("ORIGIN_SECRET", SECRET)],
    );
    assert_eq!(out.status.code(), Some(2), "{}", printed(&out));
    assert_eq!(fake.seen.lock().expect("lock").len(), 0, "nothing was sent");
    let bad = std::env::temp_dir().join(format!("rollout-bad-{}", std::process::id()));
    std::fs::write(&bad, "authorization: Bearer x\n").expect("a file");
    let out = run(
        &check_script(),
        &[&fake.url(), bad.to_str().expect("utf-8")],
        &with_release(&[]),
    );
    assert_eq!(
        out.status.code(),
        Some(2),
        "a file of another header: {}",
        printed(&out)
    );
    let _ = std::fs::remove_file(file);
    let _ = std::fs::remove_file(bad);
}

/// The engine is starting twice, then returns results: the check should
/// retry and pass.
#[test]
fn rollout_check_retries_while_the_engine_is_starting() {
    let fake = Fake::start(starting_then(SEARCH, 2, page(RESULTS)));
    let out = run(
        &check_script(),
        &[&fake.url()],
        &with_release(&[("ORIGIN_SECRET", SECRET)]),
    );
    let text = printed(&out);
    assert_eq!(out.status.code(), Some(0), "passes after retries: {text}");
    assert!(
        text.contains("search ok"),
        "the search check passes: {text}"
    );
    // Should see retry messages indicating the engine was starting.
    assert!(
        text.contains("the engine is starting"),
        "saw retry messages: {text}"
    );
}

/// The engine is always starting (never returns results): the check should
/// fail after the retry bound with a message naming the bound.
#[test]
fn rollout_check_fails_after_retries_if_engine_never_starts() {
    let fake = Fake::start(starting_then(SEARCH, 999, page(RESULTS)));
    let out = run(
        &check_script(),
        &[&fake.url()],
        &with_release(&[
            ("ORIGIN_SECRET", SECRET),
            ("ROLLOUT_SEARCH_TRIES", "3"),
            ("ROLLOUT_SEARCH_WAIT", "0"),
        ]),
    );
    let text = printed(&out);
    assert_eq!(out.status.code(), Some(1), "fails: {text}");
    assert!(text.contains("search"), "names the search check: {text}");
    // Should see "did not start" message with timeout value.
    assert!(text.contains("did not start"), "mentions timeout: {text}");
}

/// The search page has neither results nor the starting marker: the check
/// should fail immediately without retrying.
#[test]
fn rollout_check_fails_immediately_on_bad_page_without_starting_marker() {
    let fake = Fake::start(answering(SEARCH, 200, &page("<p>Something weird</p>")));
    let out = run(
        &check_script(),
        &[&fake.url()],
        &with_release(&[("ORIGIN_SECRET", SECRET)]),
    );
    let text = printed(&out);
    assert_eq!(out.status.code(), Some(1), "fails: {text}");
    assert!(text.contains("search"), "names the search check: {text}");
    // Should see "has no result list" message, not "did not start".
    assert!(
        text.contains("has no result list"),
        "immediate failure: {text}"
    );
    assert!(
        !text.contains("the engine is starting"),
        "no retry messages: {text}"
    );
}

/// One `rollout-check.sh` run: its mode name, its arguments and its extra
/// environment.
type RolloutRun<'a> = (&'a str, Vec<&'a str>, Vec<(&'a str, &'a str)>);

/// Task 007 F5: `rollout-check.sh` read `token-pattern` with an unguarded
/// `head` as `smoke.sh` did before F1: a missing or empty pattern made
/// `grep -E ''` match every response ("a token shape in the body of the root
/// response" about a healthy page). It must exit 2 naming the file, in every
/// mode, before any request.
#[test]
fn rollout_check_refuses_a_missing_or_empty_token_pattern() {
    let fake = Fake::start(healthy);
    for (case, pattern) in [
        ("missing", None),
        ("empty", Some("")),
        ("a blank line", Some("\n")),
        ("blanks", Some("  \n\n")),
    ] {
        let dir = std::env::temp_dir().join(format!(
            "rollout-pattern-{}-{}",
            std::process::id(),
            case.replace(' ', "-")
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a folder");
        std::fs::copy(check_script(), dir.join("rollout-check.sh")).expect("copy the script");
        if let Some(text) = pattern {
            std::fs::write(dir.join("token-pattern"), text).expect("write token-pattern");
        }
        let script = dir.join("rollout-check.sh");
        let url = fake.url();
        let runs: [RolloutRun; 3] = [
            (
                "direct",
                vec![url.as_str()],
                with_release(&[("ORIGIN_SECRET", SECRET)]),
            ),
            ("worker", vec!["--via-worker", url.as_str()], vec![]),
            ("refused", vec!["--refused", url.as_str()], vec![]),
        ];
        for (mode, args, env) in runs {
            let out = run(&script, &args, &env);
            let text = printed(&out);
            assert_eq!(out.status.code(), Some(2), "{case}, {mode}: {text}");
            assert!(
                text.contains("token-pattern"),
                "{case}, {mode}: names the file: {text}"
            );
            assert!(
                !text.contains("FAIL token"),
                "{case}, {mode}: must not blame a response: {text}"
            );
        }
        let _ = std::fs::remove_dir_all(dir);
    }
    assert_eq!(fake.seen.lock().expect("lock").len(), 0, "nothing was sent");
}

#[test]
fn rollout_check_refused_mode_fails_on_a_server_that_answers() {
    let open = Fake::start(|r| {
        healthy(&Req {
            path: r.path.clone(),
            headers: vec![("x-origin-auth".into(), SECRET.into())],
        })
    });
    let out = run(&check_script(), &["--refused", &open.url()], &[]);
    assert_fails_naming(&out, "refused", "an open origin");
    let wordy = Fake::start(|_| Resp::new(403, "forbidden: send x-origin-auth"));
    let out = run(&check_script(), &["--refused", &wordy.url()], &[]);
    assert_fails_naming(&out, "refused", "a 403 with a body");
}

/// R6: Cloudflare's challenge is told apart from our own refusal.
#[test]
fn rollout_check_tells_a_cloudflare_challenge_from_a_failure() {
    let challenge = Fake::start(|_| Resp::new(403, "challenge").with("cf-mitigated", "challenge"));
    let out = run(&check_script(), &["--via-worker", &challenge.url()], &[]);
    let text = printed(&out);
    assert_eq!(out.status.code(), Some(3), "{text}");
    assert!(text.contains("challenge"), "{text}");
}

/// Exit 4: the Worker itself answered 500 (it has no secret or no origin;
/// `origin-secret push` is the remedy), told from a 500 of the service that
/// the Worker relays (a bad rollout: exit 1) and from a refusal (403).
#[test]
fn rollout_check_tells_the_workers_500_from_a_failure() {
    for body in [
        "The origin secret is not configured.",
        "The origin is not configured.",
    ] {
        let worker = Fake::start(move |_| Resp::new(500, body).with("cache-control", "no-store"));
        let out = run(&check_script(), &["--via-worker", &worker.url()], &[]);
        let text = printed(&out);
        assert_eq!(out.status.code(), Some(4), "{body}: {text}");
        assert!(text.contains("Worker"), "{body}: names the Worker: {text}");
    }
    let relayed = Fake::start(|_| Resp::new(500, "boom"));
    let out = run(&check_script(), &["--via-worker", &relayed.url()], &[]);
    assert_fails_naming(&out, "root", "the service's own 500, relayed");
    let refusal = Fake::start(|_| Resp::new(403, "forbidden"));
    let out = run(&check_script(), &["--via-worker", &refusal.url()], &[]);
    assert_fails_naming(&out, "root", "a 403");
}

/// The Worker's 500 is recognised on the suffix every one of its bodies
/// shares, `is not configured.`, after trimming whitespace and CR: a proxy
/// or a hop may add a CRLF, a trailing newline or padding.
#[test]
fn rollout_check_recognises_the_workers_500_through_crlf_and_newlines() {
    for body in [
        "The origin secret is not configured.\r\n",
        "The origin secret is not configured.\n",
        "The origin secret is not configured.\n\n",
        "The origin is not configured.\r",
        "The public origin is not configured.",
        "The public origin is not configured.\r\n",
        "  The public origin is not configured.  \r\n",
        "\r\nThe origin secret is not configured.\r\n\r\n",
    ] {
        let worker = Fake::start(move |_| Resp::new(500, body).with("cache-control", "no-store"));
        let out = run(&check_script(), &["--via-worker", &worker.url()], &[]);
        let text = printed(&out);
        assert_eq!(out.status.code(), Some(4), "{body:?}: {text}");
        assert!(
            text.contains("Worker"),
            "{body:?}: names the Worker: {text}"
        );
    }
}

/// A 500 of the service, relayed, is a failed check even when it is close:
/// the suffix must end the body, and must be a whole sentence's end.
#[test]
fn rollout_check_does_not_take_a_service_500_for_the_workers() {
    for body in [
        "boom",
        "",
        "is not configured. and then some",
        "The origin is not configured, sorry",
        "The origin is not configured",
    ] {
        let relayed = Fake::start(move |_| Resp::new(500, body));
        let out = run(&check_script(), &["--via-worker", &relayed.url()], &[]);
        assert_fails_naming(&out, "root", &format!("the service's own 500 {body:?}"));
    }
}

/// Only through the Worker is a 500 the Worker's: a direct call to a
/// revision that answers the same words is a failing check.
#[test]
fn rollout_check_direct_mode_has_no_worker_500() {
    let worker_like = Fake::start(|_| Resp::new(500, "The origin secret is not configured."));
    let out = run(
        &check_script(),
        &[&worker_like.url()],
        &with_release(&[("ORIGIN_SECRET", SECRET)]),
    );
    assert_fails_naming(&out, "root", "direct mode");
}

// ------------------------------------------------- rollout-check.sh: release

/// A server that serves `head` in the page's head, to a caller with the
/// secret, on `/`; healthy for the rest.
fn serving_head(head: &'static str) -> Handler {
    Box::new(move |r| {
        if r.authed() && r.path == "/" {
            Resp::new(200, &page_with_head(head, "")).with("content-type", "text/html")
        } else {
            healthy(r)
        }
    })
}

/// Runs the direct mode on a server serving `head`, wanting `env` and `sha`.
fn release_run(head: &'static str, env: &str, sha: &str) -> Output {
    let fake = Fake::start(serving_head(head));
    run(
        &check_script(),
        &[&fake.url()],
        &[
            ("ORIGIN_SECRET", SECRET),
            ("ROLLOUT_EXPECT_ENV", env),
            ("ROLLOUT_EXPECT_SHA", sha),
        ],
    )
}

/// A release case: what is wrong or right, the head the page serves, the
/// intended env and the intended sha.
type ReleaseCase = (&'static str, &'static str, &'static str, &'static str);

/// Every way the served metas can differ from the intended release.
fn release_mismatch_cases() -> [ReleaseCase; 8] {
    [
        (
            "the env meta says feat/x while main is wanted",
            r#"<meta name="search-release-env" content="feat/x"><meta name="search-release-sha" content="34c489c">"#,
            "main",
            "34c489c",
        ),
        (
            "the sha meta differs",
            r#"<meta name="search-release-env" content="main"><meta name="search-release-sha" content="0000000">"#,
            "main",
            "34c489c",
        ),
        (
            "the env meta is missing",
            r#"<meta name="search-release-sha" content="34c489c">"#,
            "main",
            "34c489c",
        ),
        (
            "the sha meta is missing",
            r#"<meta name="search-release-env" content="main">"#,
            "main",
            "34c489c",
        ),
        (
            "the env meta has no content",
            r#"<meta name="search-release-env"><meta name="search-release-sha" content="34c489c">"#,
            "main",
            "34c489c",
        ),
        (
            "content before name, a differing sha",
            r#"<meta content="main" name="search-release-env"><meta content="0000000" name="search-release-sha">"#,
            "main",
            "34c489c",
        ),
        (
            "a tag is wanted (no sha) and the page serves a sha",
            r#"<meta name="search-release-env" content="v0.1.8"><meta name="search-release-sha" content="34c489c">"#,
            "v0.1.8",
            "",
        ),
        (
            "a branch is wanted and the page serves an empty sha",
            r#"<meta name="search-release-env" content="main"><meta name="search-release-sha" content="">"#,
            "main",
            "34c489c",
        ),
    ]
}

/// Design (060, twin 028): the gate asserts the revision serves the
/// intended release, exit 8 apart from every other failure. One server per
/// way the metas can be wrong; each failure names the `release` check.
#[test]
fn rollout_check_exits_8_when_a_release_meta_differs() {
    for (case, head, env, sha) in release_mismatch_cases() {
        let out = release_run(head, env, sha);
        let text = printed(&out);
        assert_eq!(out.status.code(), Some(8), "{case}: {text}");
        assert!(
            text.contains("FAIL release"),
            "{case}: names release: {text}"
        );
        assert!(!text.contains(SECRET), "{case}: secret printed: {text}");
    }
}

/// The right metas pass, on a branch and on a tag (empty sha, which has no
/// sha), whatever the attribute order and even over several lines.
#[test]
fn rollout_check_release_ok_on_a_page_with_the_right_metas() {
    let cases: [ReleaseCase; 4] = [
        ("a branch", RELEASE_METAS, "main", "34c489c"),
        (
            "a tag",
            r#"<meta name="search-release-env" content="v0.1.8"><meta name="search-release-sha" content="">"#,
            "v0.1.8",
            "",
        ),
        (
            "content before name",
            r#"<meta content="main" name="search-release-env"><meta content="34c489c" name="search-release-sha">"#,
            "main",
            "34c489c",
        ),
        (
            "tags over several lines",
            "<meta\n  name=\"search-release-env\"\n  content=\"main\"\n><meta\r\n name='search-release-sha'\r\n content='34c489c'>",
            "main",
            "34c489c",
        ),
    ];
    for (case, head, env, sha) in cases {
        let out = release_run(head, env, sha);
        let text = printed(&out);
        assert_eq!(out.status.code(), Some(0), "{case}: {text}");
        assert!(text.contains("release ok"), "{case}: {text}");
    }
}

/// A usage case: the variable named, the case, the release variables given.
type UsageCase = (
    &'static str,
    &'static str,
    Vec<(&'static str, &'static str)>,
);

/// Design: either variable missing is a usage error (exit 2) before any
/// request, naming the variable; a sha that is SET but empty (a tag) is not.
#[test]
fn rollout_check_demands_the_intended_release() {
    let cases: [UsageCase; 3] = [
        (
            "ROLLOUT_EXPECT_ENV",
            "env unset",
            vec![("ROLLOUT_EXPECT_SHA", EXPECT_SHA)],
        ),
        (
            "ROLLOUT_EXPECT_ENV",
            "env empty",
            vec![
                ("ROLLOUT_EXPECT_ENV", ""),
                ("ROLLOUT_EXPECT_SHA", EXPECT_SHA),
            ],
        ),
        (
            "ROLLOUT_EXPECT_SHA",
            "sha unset",
            vec![("ROLLOUT_EXPECT_ENV", EXPECT_ENV)],
        ),
    ];
    for (name, case, release) in cases {
        let fake = Fake::start(healthy);
        let mut env = vec![("ORIGIN_SECRET", SECRET)];
        env.extend(release);
        let out = run(&check_script(), &[&fake.url()], &env);
        let text = printed(&out);
        assert_eq!(out.status.code(), Some(2), "{case}: {text}");
        assert!(text.contains(name), "{case}: names {name}: {text}");
        assert_eq!(
            fake.seen.lock().expect("lock").len(),
            0,
            "{case}: no request"
        );
    }
    // A tag has no sha: set but empty is a valid intent.
    let tag = r#"<meta name="search-release-env" content="v0.1.8"><meta name="search-release-sha" content="">"#;
    let out = release_run(tag, "v0.1.8", "");
    assert_eq!(out.status.code(), Some(0), "empty sha: {}", printed(&out));
}

/// A page that serves the origin secret as a release meta must not make the
/// message print it, whichever way the secret was given.
#[test]
fn rollout_check_never_prints_the_origin_secret_in_the_release_message() {
    let echo: &'static str = Box::leak(
        format!(
            r#"<meta name="search-release-env" content="{SECRET}"><meta name="search-release-sha" content="34c489c">"#
        )
        .into_boxed_str(),
    );
    let file = header_file("release-secret");
    let file_arg = file.to_str().expect("utf-8");
    let release = [
        ("ROLLOUT_EXPECT_ENV", EXPECT_ENV),
        ("ROLLOUT_EXPECT_SHA", EXPECT_SHA),
    ];
    let mut env_mode = vec![("ORIGIN_SECRET", SECRET)];
    env_mode.extend(release);
    for (case, args, env) in [
        ("environment", vec![], env_mode),
        ("header file", vec![file_arg], release.to_vec()),
    ] {
        let fake = Fake::start(serving_head(echo));
        let url = fake.url();
        let mut all: Vec<&str> = vec![&url];
        all.extend(args);
        let out = run(&check_script(), &all, &env);
        let text = printed(&out);
        assert_eq!(out.status.code(), Some(8), "{case}: {text}");
        assert!(
            text.contains("<the origin secret>"),
            "{case}: masked: {text}"
        );
        assert!(!text.contains(SECRET), "{case}: secret printed: {text}");
    }
    let _ = std::fs::remove_file(file);
}

/// Design: `--via-worker` and `--refused` neither read nor need the intended
/// release, and never run the release check.
#[test]
fn rollout_check_release_is_for_the_direct_mode_only() {
    let fake = Fake::start(through_the_worker);
    let chrome = FakeChrome::new("release-worker");
    let out = run(
        &check_script(),
        &["--via-worker", &fake.url()],
        &chrome.env("ok"),
    );
    let text = printed(&out);
    assert_eq!(out.status.code(), Some(0), "via the Worker: {text}");
    assert!(!text.contains("release"), "via the Worker: {text}");

    let wall = Fake::start(|_| Resp::new(403, ""));
    let out = run(&check_script(), &["--refused", &wall.url()], &[]);
    let text = printed(&out);
    assert_eq!(out.status.code(), Some(0), "refused: {text}");
    assert!(!text.contains("release"), "refused: {text}");
}

// ------------------------------------------------- rollout-check.sh: hydrate

/// The DOM a hydrated page dumps: `<html data-hydrated>` (ui-lib's
/// `hydrate` sets it), Cloudflare's decoy first in `<body>`, the app's root
/// after it.
const HYDRATED_DOM: &str = "<html lang=\"en\" data-hydrated=\"\"><head></head><body>\
     <a href=\"/cdn-cgi/content?id=x\" style=\"display:none\"></a>\
     <div id=\"pc-app\"></div></body></html>";
/// The same page that never hydrated: no marker.
const BARE_DOM: &str =
    "<html lang=\"en\"><head></head><body><div id=\"pc-app\"></div></body></html>";
/// Cloudflare's managed challenge, as Chrome dumps it.
const CHALLENGE_DOM: &str = "<html><head><title>Just a moment...</title></head><body>\
     <script>window._cf_chl_opt={cType:'managed'};</script></body></html>";
/// What Chrome's `--enable-logging=stderr` prints for the panic of ui-lib 063.
const PANIC_LINE: &str =
    "[1:2:INFO:CONSOLE(1)] \"panicked at tachys-0.2.19/src/hydration.rs:227:9\"";

/// A `chrome` that logs its arguments and plays `FAKE_CHROME_MODE` per URL:
/// `ok`, `panic` (the panic on stderr, the marker still dumped: stderr is
/// read on its own), `runtime` (a wasm `RuntimeError`), `bare` (no marker),
/// `challenge`, `hang` (never answers), `privacy-panic` (`ok` but on
/// `/privacy`, which panics).
struct FakeChrome {
    dir: PathBuf,
    bin: PathBuf,
    log: PathBuf,
}

impl FakeChrome {
    fn new(name: &str) -> FakeChrome {
        let dir = std::env::temp_dir().join(format!("fake-chrome-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a fake chrome dir");
        let log = dir.join("argv.log");
        let bin = dir.join("chrome");
        let body = format!(
            r#"#!/bin/bash
printf '%s\n' "$@" >> '{log}'
url="${{!#}}"
mode="${{FAKE_CHROME_MODE:-ok}}"
if [ "$mode" = privacy-panic ]; then
  if [ "${{url%/privacy}}" != "$url" ]; then mode=panic; else mode=ok; fi
fi
case "$mode" in
  ok) printf '%s\n' '{HYDRATED_DOM}' ;;
  panic) printf '%s\n' '{PANIC_LINE}' >&2; printf '%s\n' '{HYDRATED_DOM}' ;;
  runtime) printf '%s\n' 'Uncaught RuntimeError: unreachable' >&2; printf '%s\n' '{BARE_DOM}' ;;
  bare) printf '%s\n' '{BARE_DOM}' ;;
  challenge) printf '%s\n' '{CHALLENGE_DOM}' ;;
  hang) exec sleep 30 ;;
esac
"#,
            log = log.display()
        );
        std::fs::write(&bin, body).expect("a fake chrome");
        Command::new("chmod")
            .args(["+x", bin.to_str().expect("utf-8")])
            .status()
            .expect("chmod");
        FakeChrome { dir, bin, log }
    }

    /// The environment that makes `rollout-check.sh` run this Chrome in
    /// `mode`, with a limit of 3 s for `hang` and 30 s for the others (a
    /// loaded machine must not turn a pass into a timeout; 60 s in a
    /// deploy).
    fn env(&self, mode: &'static str) -> Vec<(&'static str, &str)> {
        let limit = if mode == "hang" { "3" } else { "30" };
        vec![
            ("CHROME", self.bin.to_str().expect("utf-8")),
            ("FAKE_CHROME_MODE", mode),
            ("ROLLOUT_CHROME_TIMEOUT", limit),
        ]
    }

    /// Every argument Chrome was given, one per line.
    fn argv(&self) -> String {
        std::fs::read_to_string(&self.log).unwrap_or_default()
    }
}

impl Drop for FakeChrome {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The public host as the Worker serves it: healthy without the header.
fn through_the_worker(req: &Req) -> Resp {
    healthy(&Req {
        path: req.path.clone(),
        headers: vec![("x-origin-auth".into(), SECRET.into())],
    })
}

/// T3 (ui-lib 063): through the Worker, a page that hydrates on `/`, a
/// search and `/privacy` passes, and Chrome ran headless, with its console
/// on stderr, a virtual-time budget, its DOM dumped, on the three pages.
#[test]
fn rollout_check_hydrate_passes_when_every_page_hydrates() {
    let fake = Fake::start(through_the_worker);
    let chrome = FakeChrome::new("ok");
    let out = run(
        &check_script(),
        &["--via-worker", &fake.url()],
        &chrome.env("ok"),
    );
    let text = printed(&out);
    assert_eq!(out.status.code(), Some(0), "{text}");
    assert!(text.contains("rollout-check: hydrate ok"), "{text}");
    let argv = chrome.argv();
    for flag in [
        "--headless=new",
        "--dump-dom",
        "--virtual-time-budget=15000",
        "--enable-logging=stderr",
    ] {
        assert_eq!(
            argv.lines().filter(|l| *l == flag).count(),
            3,
            "{flag}: {argv}"
        );
    }
    for page in ["/", "/?q=satipatthana", "/privacy"] {
        let url = format!("{}{page}", fake.url());
        assert!(argv.lines().any(|l| l == url), "{url} visited: {argv}");
    }
}

/// A panic (or a wasm `RuntimeError`) in Chrome's console fails the
/// `hydrate` check, even when the marker is in the DOM.
#[test]
fn rollout_check_hydrate_fails_on_a_panic_in_the_browser() {
    let fake = Fake::start(through_the_worker);
    for mode in ["panic", "runtime"] {
        let chrome = FakeChrome::new(mode);
        let out = run(
            &check_script(),
            &["--via-worker", &fake.url()],
            &chrome.env(mode),
        );
        assert_fails_naming(&out, "FAIL hydrate", mode);
        assert!(
            printed(&out).contains("panicked in the browser"),
            "{mode}: {}",
            printed(&out)
        );
    }
    let chrome = FakeChrome::new("privacy-panic");
    let out = run(
        &check_script(),
        &["--via-worker", &fake.url()],
        &chrome.env("privacy-panic"),
    );
    assert_fails_naming(
        &out,
        "FAIL hydrate: /privacy: the page panicked",
        "only /privacy panics",
    );
}

/// A page whose dumped DOM has no `data-hydrated` never hydrated: a fail,
/// and so is a Chrome stopped by the time limit.
#[test]
fn rollout_check_hydrate_fails_without_the_marker_or_on_a_timeout() {
    let fake = Fake::start(through_the_worker);
    for (mode, why) in [
        ("bare", "the page never hydrated"),
        ("hang", "Chrome stopped"),
    ] {
        let chrome = FakeChrome::new(mode);
        let started = Instant::now();
        let out = run(
            &check_script(),
            &["--via-worker", &fake.url()],
            &chrome.env(mode),
        );
        assert_fails_naming(&out, "FAIL hydrate", mode);
        assert!(
            printed(&out).contains(&format!("no data-hydrated on <html>: {why}")),
            "{mode}: {}",
            printed(&out)
        );
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "{mode}: the limit holds"
        );
    }
}

/// A Cloudflare challenge in the dumped page says nothing about the
/// service: exit 3 (a skip the orchestrator re-runs by hand), never 0.
#[test]
fn rollout_check_hydrate_tells_a_challenge_from_a_pass() {
    let fake = Fake::start(through_the_worker);
    let chrome = FakeChrome::new("challenge");
    let out = run(
        &check_script(),
        &["--via-worker", &fake.url()],
        &chrome.env("challenge"),
    );
    let text = printed(&out);
    assert_eq!(out.status.code(), Some(3), "{text}");
    assert!(text.contains("challenge"), "{text}");
    assert!(!text.contains("hydrate ok"), "{text}");
}

/// No usable Chrome is the runner's fault, not the service's: exit 2.
#[test]
fn rollout_check_hydrate_needs_a_chrome() {
    let fake = Fake::start(through_the_worker);
    let out = run(
        &check_script(),
        &["--via-worker", &fake.url()],
        &[("CHROME", "/nonexistent/chrome")],
    );
    let text = printed(&out);
    assert_eq!(out.status.code(), Some(2), "{text}");
    assert!(text.contains("Chrome"), "{text}");
}

/// Chrome cannot send the origin header, so a revision URL (direct mode)
/// is never driven by it.
#[test]
fn rollout_check_hydrate_runs_through_the_worker_only() {
    let fake = Fake::start(healthy);
    let chrome = FakeChrome::new("direct");
    let out = run(
        &check_script(),
        &[&fake.url()],
        &with_release(&[
            ("ORIGIN_SECRET", SECRET),
            ("CHROME", chrome.bin.to_str().expect("utf-8")),
            ("FAKE_CHROME_MODE", "panic"),
        ]),
    );
    let text = printed(&out);
    assert_eq!(out.status.code(), Some(0), "{text}");
    assert_eq!(chrome.argv().len(), 0, "Chrome ran: {}", chrome.argv());
}

/// The secret reaches the server and nothing else: not the output, not a
/// curl argument, and the header file's path is not printed either.
/// A directory holding a `curl` that logs its arguments, then runs the real
/// one: its path, the log, and a `PATH` that finds it first.
fn curl_shim() -> (PathBuf, PathBuf, String) {
    let dir = std::env::temp_dir().join(format!("rollout-shim-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a shim dir");
    let real = Command::new("bash")
        .args(["-c", "command -v curl"])
        .output()
        .expect("bash runs");
    let real = String::from_utf8_lossy(&real.stdout).trim().to_owned();
    assert!(!real.is_empty(), "curl is installed");
    let log = dir.join("argv.log");
    let shim = dir.join("curl");
    let body = format!(
        "#!/bin/bash\nprintf '%s\\n' \"$@\" >> '{}'\nexec '{real}' \"$@\"\n",
        log.display()
    );
    std::fs::write(&shim, body).expect("a shim");
    Command::new("chmod")
        .args(["+x", shim.to_str().expect("utf-8")])
        .status()
        .expect("chmod");
    let path = format!(
        "{}:{}",
        dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    (dir, log, path)
}

#[test]
fn the_secret_is_sent_but_never_printed_nor_put_on_a_command_line() {
    let (shim_dir, log, path) = curl_shim();
    let file = header_file("hygiene");
    let file_arg = file.to_str().expect("utf-8");
    for (case, args, env) in [
        (
            "env",
            vec![],
            vec![("ORIGIN_SECRET", SECRET), ("PATH", path.as_str())],
        ),
        ("file", vec![file_arg], vec![("PATH", path.as_str())]),
    ] {
        let fake = Fake::start(healthy);
        let url = fake.url();
        let mut all: Vec<&str> = vec![&url];
        all.extend(args);
        let out = run(&check_script(), &all, &with_release(&env));
        let text = printed(&out);
        assert_eq!(out.status.code(), Some(0), "{case}: {text}");
        assert!(
            fake.authed_requests() >= 3,
            "{case}: the secret reached the server"
        );
        assert!(!text.contains(SECRET), "{case}: secret printed");
        assert!(
            !text.contains(file_arg),
            "{case}: the header file's path is printed"
        );
        let argv = std::fs::read_to_string(&log).unwrap_or_default();
        assert!(!argv.is_empty(), "{case}: the shim saw curl run");
        assert!(!argv.contains(SECRET), "{case}: secret on a command line");
        let _ = std::fs::remove_file(&log);
    }
    let _ = std::fs::remove_file(file);
    let _ = std::fs::remove_dir_all(shim_dir);
}
