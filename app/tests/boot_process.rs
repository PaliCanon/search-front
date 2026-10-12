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

/// A free local port (a race with another process is possible, never seen).
fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("a free port")
        .local_addr()
        .expect("an address")
        .port()
}

/// The binary, with the given search variables and its own address.
fn command(port: u16, search_env: &[(&str, &str)]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_search_front"));
    command
        .env_remove("SEARCH_API_URL")
        .env_remove("SEARCH_API_TOKEN")
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
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .expect("a timeout");
    write!(stream, "GET {path} HTTP/1.0\r\nHost: localhost\r\n\r\n").expect("write");
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
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains(named), "stderr names {named}: {stderr}");
        let everything = format!("{stderr}{}", String::from_utf8_lossy(&out.stdout));
        assert!(!everything.contains(CANARY), "token leaked: {everything}");
    }
}

#[test]
fn the_mock_boots_with_no_variables_and_says_so() {
    let port = free_port();
    let service = start(port, &[]);
    let (status, _) = get(port, "/?q=mindfulness");
    assert_eq!(status, 200, "the mock answers");
    let ended = finish(service);
    assert_eq!(
        ended.stdout.matches("search: mock").count(),
        1,
        "{}",
        ended.stdout
    );
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
    assert_eq!(
        ended.stdout.matches("search: real").count(),
        1,
        "{}",
        ended.stdout
    );
    let sent = seen.lock().expect("requests lock").join("\n");
    assert!(
        sent.contains(&format!("Bearer {CANARY}")),
        "the API got it: {sent}"
    );
    assert!(
        ended.stderr.contains("401"),
        "the failure is logged: {}",
        ended.stderr
    );
    for (what, text) in [
        ("stdout", &ended.stdout),
        ("stderr", &ended.stderr),
        ("page", &page),
    ] {
        assert!(!text.contains(CANARY), "token in {what}: {text}");
        assert!(!text.contains(API_MESSAGE), "api message in {what}: {text}");
    }
    assert!(!ended.stderr.contains("secret_query_words"), "query logged");
    assert!(
        !ended.success,
        "the service was killed, not ended by itself"
    );
}
