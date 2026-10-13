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

/// The page the real service renders, as far as the scripts read it.
fn page(extra: &str) -> String {
    format!(
        "<!DOCTYPE html><html><head><title>PaliCanon — Search</title>\
         <link rel=\"preload\" href=\"{WASM}\" as=\"fetch\" type=\"application/wasm\">\
         </head><body>{extra}</body></html>"
    )
}

const RESULTS: &str = "<ol class=\"divide-y\"><li>MN 10</li></ol>";

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
        .env("SMOKE_WAIT", "8");
    for (name, value) in env {
        command.env(name, value);
    }
    command.output().expect("bash runs the script")
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

    let out = run(&check_script(), &[&url], &[("ORIGIN_SECRET", SECRET)]);
    let text = printed(&out);
    assert_eq!(
        out.status.code(),
        Some(0),
        "secret from the environment: {text}"
    );
    for check in ["root", "wasm", "search", "token", "guard"] {
        assert!(text.contains(check), "{check} is reported: {text}");
    }

    let file = header_file("real");
    let file_arg = file.to_str().expect("utf-8");
    let out = run(&check_script(), &[&url, file_arg], &[]);
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
    let out = run(&check_script(), &["--via-worker", &open_url], &[]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "via the Worker: {}",
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
            &[("ORIGIN_SECRET", SECRET)],
        );
        assert_fails_naming(&out, check, case);
    }
}

#[test]
fn rollout_check_demands_a_secret_and_refuses_to_send_it_to_the_worker() {
    let fake = Fake::start(healthy);
    let out = run(&check_script(), &[&fake.url()], &[]);
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
        &[],
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
        let runs: [(&str, Vec<&str>, Vec<(&str, &str)>); 3] = [
            (
                "direct",
                vec![url.as_str()],
                vec![("ORIGIN_SECRET", SECRET)],
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
        &[("ORIGIN_SECRET", SECRET)],
    );
    assert_fails_naming(&out, "root", "direct mode");
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
        let out = run(&check_script(), &all, &env);
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
