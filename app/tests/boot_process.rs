//! The service as a process: the binary booted with an environment, the
//! way `cargo leptos watch` runs it (task 005, slice 7). It is the only
//! place where the real `main`, its exit code and every line it prints can
//! be read; the router's own behaviour is tested in-process
//! (`router/tests.rs`).
#![cfg(feature = "ssr")]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// A token no real one looks like: it must be in no line the service prints.
const CANARY: &str = "pc_canary_process_5c1e44";
/// A message the (mock) API refuses with: it must not be logged either.
const API_MESSAGE: &str = "the_api_message_canary_88";

/// The running binary, killed when dropped (a failed assertion leaves none).
struct Service(Option<Child>);

impl Drop for Service {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// A free local port, never one this test binary already handed out: the
/// tests run in parallel and a port is released before its service binds it,
/// so two of them could otherwise be given the same one.
fn free_port() -> u16 {
    static HANDED_OUT: Mutex<Vec<u16>> = Mutex::new(Vec::new());
    loop {
        let port = TcpListener::bind("127.0.0.1:0")
            .expect("a free port")
            .local_addr()
            .expect("an address")
            .port();
        let mut handed_out = HANDED_OUT.lock().expect("ports lock");
        if !handed_out.contains(&port) {
            handed_out.push(port);
            return port;
        }
    }
}

/// The binary, with the given search variables and its own address.
fn command(port: u16, search_env: &[(&str, &str)]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_search_front"));
    command
        .env_remove("SEARCH_API_URL")
        .env_remove("SEARCH_API_TOKEN")
        .env_remove("ORIGIN_SECRET")
        .env_remove("LEPTOS_ENV")
        .env_remove("K_SERVICE")
        .env_remove("SEARCH_FRONT_ENV")
        .env_remove("SEARCH_FRONT_SHA")
        .env_remove("SEARCH_FRONT_LOG_LEVEL")
        .env_remove("SEARCH_FRONT_DEPLOYMENT_ENV")
        .env("LEPTOS_OUTPUT_NAME", "search-front")
        .env("LEPTOS_SITE_ADDR", format!("127.0.0.1:{port}"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (name, value) in search_env {
        command.env(name, value);
    }
    command
}

/// What the service printed once stopped, and how it ended.
struct Ended {
    stdout: String,
    stderr: String,
    success: bool,
}

/// Stops the service (if it still runs) and reads everything it printed.
fn finish(mut service: Service) -> Ended {
    let mut child = service.0.take().expect("a running service");
    let _ = child.kill();
    let out = child.wait_with_output().expect("the service's output");
    Ended {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        success: out.status.success(),
    }
}

/// Starts the service and waits until it accepts connections.
fn start(port: u16, search_env: &[(&str, &str)]) -> Service {
    let child = command(port, search_env)
        .spawn()
        .expect("the binary starts");
    let mut service = Service(Some(child));
    let deadline = Instant::now() + Duration::from_secs(30);
    while TcpStream::connect(("127.0.0.1", port)).is_err() {
        let child = service.0.as_mut().expect("a running service");
        assert!(
            child.try_wait().expect("a status").is_none(),
            "the service stopped before listening"
        );
        assert!(Instant::now() < deadline, "the service never listened");
        std::thread::sleep(Duration::from_millis(50));
    }
    service
}

/// `GET path` over a fresh HTTP/1.0 connection: the status and the text.
fn get(port: u16, path: &str) -> (u16, String) {
    get_with(port, path, "")
}

/// `GET path` with extra header lines (each ending in CRLF).
fn get_with(port: u16, path: &str, headers: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .expect("a timeout");
    write!(
        stream,
        "GET {path} HTTP/1.0\r\nHost: localhost\r\n{headers}\r\n"
    )
    .expect("write");
    let mut raw = Vec::new();
    let _ = stream.read_to_end(&mut raw);
    let text = String::from_utf8_lossy(&raw).into_owned();
    let status = text
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .expect("a status line");
    (status, text)
}

/// A search API that refuses every call with a 401 and its own message;
/// the requests it received are kept for the test to read.
fn refusing_api() -> (u16, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a free port");
    let port = listener.local_addr().expect("an address").port();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let kept = Arc::clone(&seen);
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            answer_401(stream, &kept);
        }
    });
    (port, seen)
}

/// Reads one request and refuses it.
fn answer_401(mut stream: TcpStream, seen: &Mutex<Vec<String>>) {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).unwrap_or(0);
    seen.lock()
        .expect("requests lock")
        .push(String::from_utf8_lossy(&buf[..n]).into_owned());
    let body = format!(r#"{{"error":"bad_token","message":"{API_MESSAGE}"}}"#);
    let _ = write!(
        stream,
        "HTTP/1.1 401 Unauthorized\r\ncontent-type: application/json\r\n\
         content-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    );
}

/// Runs the command for at most 10 s: its output, and whether it ended by
/// itself (a service that listens is killed, and did not).
fn run_to_exit(mut command: Command) -> (std::process::Output, bool) {
    let mut child = command.spawn().expect("the binary starts");
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut by_itself = true;
    while child.try_wait().expect("a status").is_none() {
        if Instant::now() > deadline {
            by_itself = false;
            let _ = child.kill();
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    (child.wait_with_output().expect("the output"), by_itself)
}

#[test]
fn a_half_configured_environment_refuses_to_start_and_names_the_variable() {
    let cases = [
        (vec![("SEARCH_API_TOKEN", CANARY)], "SEARCH_API_URL"),
        (
            vec![
                ("SEARCH_API_URL", "http://127.0.0.1:9"),
                ("SEARCH_API_TOKEN", ""),
            ],
            "SEARCH_API_TOKEN",
        ),
    ];
    for (env, named) in cases {
        let (out, ended_by_itself) = run_to_exit(command(free_port(), &env));
        assert!(ended_by_itself, "{env:?} must stop at boot, not listen");
        assert!(!out.status.success(), "{env:?} must exit non-zero");
        assert_eq!(out.stderr, b"");
        let stdout = String::from_utf8_lossy(&out.stdout);
        let failure = event(&stdout, "service.failed");
        assert_eq!(failure["stage"], named);
        assert_eq!(failure["severity"], "ERROR");
        assert!(!stdout.contains(CANARY), "token leaked: {stdout}");
    }
}

#[test]
fn the_mock_boots_with_no_variables_and_says_so() {
    let port = free_port();
    let service = start(port, &[]);
    let (status, _) = get(port, "/?q=mindfulness");
    assert_eq!(status, 200, "the mock answers");
    let ended = finish(service);
    assert_eq!(ended.stderr, "");
    assert_eq!(event(&ended.stdout, "service.started")["source.mock"], true);
}

#[test]
fn the_boot_logs_its_release_once() {
    let cases = [
        (vec![], "dev", ""),
        (vec![("SEARCH_FRONT_ENV", "v0.1.8")], "v0.1.8", ""),
        (
            vec![
                ("SEARCH_FRONT_ENV", "main"),
                ("SEARCH_FRONT_SHA", "0123456789abcdef"),
            ],
            "main",
            "0123456",
        ),
    ];
    for (env, version, commit) in cases {
        let port = free_port();
        let service = start(port, &env);
        assert_eq!(get(port, "/?q=mindfulness").0, 200);
        let ended = finish(service);
        let started = event(&ended.stdout, "service.started");
        assert_eq!(started["service.version"], version);
        assert_eq!(started["service.commit"], commit);
    }
}

#[test]
fn the_real_boot_logs_its_source_once_and_no_line_holds_the_token() {
    let (api_port, seen) = refusing_api();
    let url = format!("http://127.0.0.1:{api_port}");
    let port = free_port();
    let service = start(
        port,
        &[("SEARCH_API_URL", &url), ("SEARCH_API_TOKEN", CANARY)],
    );
    let (status, page) = get(port, "/?q=secret_query_words");
    assert_eq!(status, 502, "a refused token is the failed state");
    let ended = finish(service);
    assert_eq!(ended.stderr, "");
    assert_eq!(
        event(&ended.stdout, "service.started")["source.mock"],
        false
    );
    let sent = seen.lock().expect("requests lock").join("\n");
    assert!(
        sent.contains(&format!("Bearer {CANARY}")),
        "the API got it: {sent}"
    );
    let failed = event(&ended.stdout, "api.failed");
    assert_eq!(failed["error.type"], "authentication");
    assert_eq!(failed["http.response.status_code"], 401);
    for (what, text) in [
        ("stdout", &ended.stdout),
        ("stderr", &ended.stderr),
        ("page", &page),
    ] {
        assert!(!text.contains(CANARY), "token in {what}: {text}");
        assert!(!text.contains(API_MESSAGE), "api message in {what}: {text}");
    }
    assert!(!ended.stdout.contains("secret_query_words"), "query logged");
    assert!(
        !ended.success,
        "the service was killed, not ended by itself"
    );
}

/// An origin secret no real one looks like: it must be in no line printed.
const ORIGIN_CANARY: &str = "origin_canary_process_b7d3";

/// The search variables of a deployed build that has a real source.
fn real_env(url: &str) -> Vec<(&str, &str)> {
    vec![("SEARCH_API_URL", url), ("SEARCH_API_TOKEN", CANARY)]
}

/// Runs a build that must refuse to start: JSON names `named`, nothing
/// is listening, no canary is printed.
fn assert_refuses(env: &[(&str, &str)], named: &str) {
    let (out, ended_by_itself) = run_to_exit(command(free_port(), env));
    assert!(ended_by_itself, "{env:?} must stop at boot, not listen");
    assert!(!out.status.success(), "{env:?} must exit non-zero");
    assert_eq!(out.stderr, b"");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(event(&stdout, "service.failed")["stage"], named);
    for canary in [CANARY, ORIGIN_CANARY] {
        assert!(!stdout.contains(canary), "leaked: {stdout}");
    }
}

#[test]
fn a_deployed_build_without_the_origin_secret_refuses_to_start() {
    let mut cloud_run = real_env("http://127.0.0.1:9");
    cloud_run.push(("K_SERVICE", "search-front-dev"));
    assert_refuses(&cloud_run, "ORIGIN_SECRET");
    let mut blank = real_env("http://127.0.0.1:9");
    blank.push(("ORIGIN_SECRET", "  "));
    assert_refuses(&blank, "ORIGIN_SECRET");
}

#[test]
fn production_outside_cloud_run_starts_without_origin_secret() {
    let port = free_port();
    let mut env = real_env("http://127.0.0.1:9");
    env.push(("LEPTOS_ENV", "PROD"));
    let service = start(port, &env);
    assert_eq!(get(port, "/").0, 200);
    let ended = finish(service);
    assert_eq!(
        event(&ended.stdout, "service.started")["origin.checked"],
        false
    );
}

#[test]
fn a_deployed_build_refuses_the_mock() {
    for deployed in [("LEPTOS_ENV", "PROD"), ("K_SERVICE", "search-front-dev")] {
        assert_refuses(
            &[deployed, ("ORIGIN_SECRET", ORIGIN_CANARY)],
            "SEARCH_API_URL",
        );
    }
}

#[test]
fn a_checked_origin_answers_only_the_right_header_and_logs_once() {
    let (api_port, _seen) = refusing_api();
    let url = format!("http://127.0.0.1:{api_port}");
    let mut env = real_env(&url);
    env.extend([("LEPTOS_ENV", "PROD"), ("ORIGIN_SECRET", ORIGIN_CANARY)]);
    let port = free_port();
    let service = start(port, &env);
    let (without, body) = get(port, "/");
    assert_eq!(without, 403, "no header: {body}");
    let (wrong, _) = get_with(port, "/", "x-origin-auth: nope\r\n");
    assert_eq!(wrong, 403, "wrong header");
    let right = format!("x-origin-auth: {ORIGIN_CANARY}\r\n");
    let (with, page) = get_with(port, "/", &right);
    assert_eq!(with, 200, "right header: {page}");
    let ended = finish(service);
    assert_eq!(
        event(&ended.stdout, "service.started")["origin.checked"],
        true
    );
    assert_eq!(
        event(&ended.stdout, "origin.refused")["reason"],
        "invalid_origin_auth"
    );
    for (what, text) in [("stdout", &ended.stdout), ("stderr", &ended.stderr)] {
        assert!(!text.contains(ORIGIN_CANARY), "origin secret in {what}");
        assert!(!text.contains(CANARY), "api token in {what}");
    }
}

#[test]
fn a_local_run_says_the_origin_is_open() {
    let port = free_port();
    let service = start(port, &[]);
    let (status, _) = get(port, "/");
    assert_eq!(status, 200);
    let ended = finish(service);
    assert_eq!(
        event(&ended.stdout, "service.started")["origin.checked"],
        false
    );
}

/// Parse every emitted line, requiring exactly one matching lifecycle event.
fn event(stdout: &str, name: &str) -> serde_json::Value {
    let logs: Vec<serde_json::Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON log"))
        .filter(|log: &serde_json::Value| log["event.name"] == name)
        .collect();
    assert_eq!(logs.len(), 1, "one {name}: {stdout}");
    logs.into_iter().next().unwrap()
}

/// Cloud Run SIGTERM is a clean stop with its own lifecycle event.
#[cfg(unix)]
#[test]
fn termination_emits_a_clean_stop() {
    let mut service = start(free_port(), &[]);
    let child = service.0.as_mut().unwrap();
    assert!(
        Command::new("kill")
            .args(["-TERM", &child.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    while child.try_wait().unwrap().is_none() {
        assert!(
            Instant::now() < deadline,
            "SIGTERM did not stop the service"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    let ended = finish(service);
    assert!(ended.success);
    assert_eq!(ended.stderr, "");
    assert_eq!(event(&ended.stdout, "service.stopped")["severity"], "INFO");
}
