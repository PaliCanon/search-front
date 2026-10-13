//! The real router, bound on a local port and asked over raw TCP: the
//! statuses and headers the page's outcomes call for, the token that never
//! leaves the server, the server function's route and its body limit
//! (task 005, slice 7).

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::time::Duration;

use leptos::prelude::LeptosOptions;

use super::router;
use crate::origin::{ORIGIN_HEADER, Origin, OriginSecret};
use crate::page::harness::exclusive;
use crate::search::mock::{COLD_QUERY, ERROR_QUERY, MockSource, RATE_LIMITED_QUERY};
use crate::search::real::{ApiToken, RealSource};
use crate::search::source::Source;
use crate::search::test_support::{MockApi, Reply};

/// A token no real one looks like: it must be in no byte the browser gets.
const CANARY: &str = "pc_canary_router_7f3a91";

/// What came back over the wire: the status line, the headers (names in
/// lower case) and the body as text.
struct Http {
    status: Option<u16>,
    headers: Vec<(String, String)>,
    body: String,
}

impl Http {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// The whole response as text, for "this appears nowhere" checks.
    fn everything(&self) -> String {
        let headers: Vec<String> = self
            .headers
            .iter()
            .map(|(n, v)| format!("{n}: {v}"))
            .collect();
        format!("{}\n{}", headers.join("\n"), self.body)
    }
}

/// Reads a response out of the bytes the server sent.
fn parse(raw: &[u8]) -> Http {
    let text = String::from_utf8_lossy(raw).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((text.as_str(), ""));
    let mut lines = head.lines();
    let status = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok());
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(n, v)| (n.trim().to_ascii_lowercase(), v.trim().to_owned()))
        .collect();
    Http {
        status,
        headers,
        body: body.to_owned(),
    }
}

/// Sends `request` (an HTTP/1.0 one: the server closes after its answer, so
/// the body is not chunked) and reads until the server closes. The request
/// is written from another thread: a server that rejects a body early may
/// close before it has read all of it.
async fn send(addr: SocketAddr, request: Vec<u8>) -> Http {
    tokio::task::spawn_blocking(move || {
        let mut stream = TcpStream::connect(addr).expect("connect to the router");
        stream
            .set_read_timeout(Some(Duration::from_secs(20)))
            .expect("a read timeout");
        let mut writer = stream.try_clone().expect("a second handle");
        let writing = std::thread::spawn(move || {
            let _ = writer.write_all(&request);
        });
        let mut raw = Vec::new();
        // A reset after a partial answer keeps what was read.
        let _ = stream.read_to_end(&mut raw);
        let _ = writing.join();
        parse(&raw)
    })
    .await
    .expect("the client thread")
}

/// `GET path`.
async fn get(addr: SocketAddr, path: &str) -> Http {
    let request = format!("GET {path} HTTP/1.0\r\nHost: localhost\r\n\r\n");
    send(addr, request.into_bytes()).await
}

/// `POST path` with a body of the given content type.
async fn post_typed(addr: SocketAddr, path: &str, content_type: &str, body: &str) -> Http {
    let request = format!(
        "POST {path} HTTP/1.0\r\nHost: localhost\r\nContent-Type: {content_type}\r\n\
         Content-Length: {}\r\n\r\n{body}",
        body.len()
    );
    send(addr, request.into_bytes()).await
}

/// `POST path` with a form body, as the browser's server-function call does.
async fn post_form(addr: SocketAddr, path: &str, body: &str) -> Http {
    post_typed(addr, path, "application/x-www-form-urlencoded", body).await
}

/// Header lines, each ending in CRLF.
fn header_lines(headers: &[(&str, &str)]) -> String {
    use std::fmt::Write as _;
    headers
        .iter()
        .fold(String::new(), |mut lines, (name, value)| {
            let _ = write!(lines, "{name}: {value}\r\n");
            lines
        })
}

/// `GET path` carrying `headers` (name, value), repeated names allowed.
async fn get_with(addr: SocketAddr, path: &str, headers: &[(&str, &str)]) -> Http {
    let extra = header_lines(headers);
    let request = format!("GET {path} HTTP/1.0\r\nHost: localhost\r\n{extra}\r\n");
    send(addr, request.into_bytes()).await
}

/// `POST path` with a form body and `headers`.
async fn post_form_with(
    addr: SocketAddr,
    path: &str,
    body: &str,
    headers: &[(&str, &str)],
) -> Http {
    let extra = header_lines(headers);
    let request = format!(
        "POST {path} HTTP/1.0\r\nHost: localhost\r\n{extra}\
         Content-Type: application/x-www-form-urlencoded\r\n\
         Content-Length: {}\r\n\r\n{body}",
        body.len()
    );
    send(addr, request.into_bytes()).await
}

/// The router over `source`, serving on a free local port of this runtime.
async fn serve(source: Source) -> SocketAddr {
    serve_with(source, Origin::Open).await
}

/// The router over `source` and `origin`.
async fn serve_with(source: Source, origin: Origin) -> SocketAddr {
    let options = LeptosOptions::builder().output_name("search-front").build();
    let app = router(options, Arc::new(source), origin);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a free local port");
    let addr = listener.local_addr().expect("a bound address");
    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .expect("the router serves");
    });
    addr
}

/// A source on the mock API, holding the canary as its token.
fn real(api: &MockApi) -> Source {
    Source::Real(RealSource::new(
        api.url(),
        ApiToken::new(CANARY),
        Duration::from_secs(10),
    ))
}

/// The mock source's server.
async fn mock_server() -> SocketAddr {
    serve(Source::Mock(MockSource)).await
}

/// A form body for the server function.
fn form(q: &str) -> String {
    format!("q={q}&lang=en&mode=both&collection=all")
}

// The guard serialises the renders of the test binary (the page's harness
// takes the same lock); it is held across the awaits on purpose.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn each_outcome_has_its_status_and_a_retry_after() {
    let _one = exclusive();
    let addr = mock_server().await;
    let cold = get(addr, &format!("/?q={COLD_QUERY}")).await;
    assert_eq!(cold.status, Some(503), "cold: {}", cold.body);
    assert_eq!(cold.header("retry-after"), Some("10"), "cold retry-after");
    let limited = get(addr, &format!("/?q={RATE_LIMITED_QUERY}")).await;
    assert_eq!(limited.status, Some(429), "rate-limited: {}", limited.body);
    assert_eq!(
        limited.header("retry-after"),
        Some("10"),
        "limited retry-after"
    );
    let failed = get(addr, &format!("/?q={ERROR_QUERY}")).await;
    assert_eq!(failed.status, Some(502), "failed: {}", failed.body);
    let hits = get(addr, "/?q=mindfulness").await;
    assert_eq!(hits.status, Some(200), "hits: {}", hits.body);
    assert!(
        hits.body.contains("MN 10"),
        "a hit is on the page: {}",
        hits.body
    );
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn no_store_goes_with_a_search_and_only_with_one() {
    let _one = exclusive();
    let addr = mock_server().await;
    let search = get(addr, "/?q=mindfulness").await;
    assert_eq!(search.header("cache-control"), Some("no-store"), "a search");
    for path in ["/", "/?q="] {
        let page = get(addr, path).await;
        assert_eq!(page.status, Some(200), "{path}");
        assert_eq!(page.header("cache-control"), None, "{path} is no search");
    }
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn an_unknown_path_is_a_404() {
    let _one = exclusive();
    let addr = mock_server().await;
    let page = get(addr, "/no/such/page").await;
    assert_eq!(page.status, Some(404), "{}", page.body);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn the_token_reaches_the_api_and_no_byte_of_the_hits_page() {
    let _one = exclusive();
    let api = MockApi::start(vec![Reply::hits()]).await;
    let addr = serve(real(&api)).await;
    let page = get(addr, "/?q=metta").await;
    assert_eq!(page.status, Some(200), "{}", page.body);
    assert!(
        page.body.contains("the four establishments"),
        "the API's hit is on the page: {}",
        page.body
    );
    assert_eq!(api.calls(), 1, "one search, one call");
    assert_eq!(
        api.call(0).authorization,
        Some(format!("Bearer {CANARY}")),
        "the API got the token"
    );
    assert!(
        page.body.contains("__RESOLVED_RESOURCES"),
        "the page carries its hydration payload: {}",
        page.body
    );
    assert!(
        !page.everything().contains(CANARY),
        "token leaked: {}",
        page.body
    );
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn the_token_is_in_no_byte_of_the_error_page() {
    let _one = exclusive();
    let api = MockApi::start(vec![Reply::error(401, "bad_token", "the token is wrong")]).await;
    let addr = serve(real(&api)).await;
    let page = get(addr, "/?q=metta").await;
    assert_eq!(page.status, Some(502), "{}", page.body);
    assert_eq!(api.calls(), 1, "the API was asked");
    assert!(
        !page.everything().contains(CANARY),
        "token leaked: {}",
        page.body
    );
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn the_server_fn_body_never_holds_the_token() {
    let _one = exclusive();
    let api = MockApi::start(vec![Reply::hits(), Reply::error(401, "bad", "no")]).await;
    let addr = serve(real(&api)).await;
    for (call, want) in [(0, "the four establishments"), (1, "Failed")] {
        let answer = post_form(addr, "/api/search_passages", &form("metta")).await;
        assert_eq!(answer.status, Some(200), "call {call}: {}", answer.body);
        assert!(answer.body.contains(want), "call {call}: {}", answer.body);
        assert!(!answer.everything().contains(CANARY), "call {call} leaked");
    }
    assert_eq!(api.calls(), 2);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn the_server_fn_route_answers_the_outcome_the_page_renders() {
    let _one = exclusive();
    let addr = mock_server().await;
    let rows = [
        (COLD_QUERY, 503, "Cold"),
        (RATE_LIMITED_QUERY, 429, "RateLimited"),
        (ERROR_QUERY, 502, "Failed"),
        ("mindfulness", 200, "mn10"),
    ];
    for (q, page_status, outcome) in rows {
        let page = get(addr, &format!("/?q={q}")).await;
        assert_eq!(page.status, Some(page_status), "page for {q}");
        let answer = post_form(addr, "/api/search_passages", &form(q)).await;
        assert_eq!(
            answer.status,
            Some(200),
            "server fn for {q}: {}",
            answer.body
        );
        assert!(
            answer.body.contains(outcome),
            "server fn for {q}: {}",
            answer.body
        );
    }
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn an_oversized_post_is_rejected_before_it_reaches_the_api() {
    let _one = exclusive();
    let api = MockApi::start(vec![Reply::hits()]).await;
    let addr = serve(real(&api)).await;
    let huge = form(&"a".repeat(64 * 1024));
    let answer = post_form(addr, "/api/search_passages", &huge).await;
    assert_eq!(answer.status, Some(500), "rejected: {}", answer.body);
    assert_eq!(api.calls(), 0, "the API never saw it");
}

/// T4: a body that is no form at all (whatever its `Content-Type`) is
/// refused with a 500 (leptos' `Args` error) before the handler runs: zero
/// calls to the API, and the refusal's text holds no token.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn a_body_that_is_not_a_form_is_refused_and_asks_the_api_nothing() {
    let _one = exclusive();
    let cases = [
        ("text/plain", "not a form"),
        ("text/plain", ""),
        ("application/json", r#"{"q":"metta"}"#),
    ];
    for (content_type, body) in cases {
        let api = MockApi::start(vec![Reply::hits()]).await;
        let addr = serve(real(&api)).await;
        let answer = post_typed(addr, "/api/search_passages", content_type, body).await;
        let case = format!("{content_type} {body:?}");
        assert_eq!(answer.status, Some(500), "{case}: {}", answer.body);
        assert_eq!(api.calls(), 0, "{case}: the API never saw it");
        assert!(!answer.everything().contains(CANARY), "{case}: leaked");
    }
}

/// Δ1 (review 005, round 2): a well-typed form that misses an argument
/// (`lang`, `mode` or `collection`) is refused like any unreadable body: a
/// 500, zero calls to the API.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn a_partial_form_is_refused() {
    let _one = exclusive();
    let forms = [
        "q=metta",
        "q=metta&lang=en",
        "q=metta&lang=en&mode=both",
        "lang=en&mode=both&collection=all",
    ];
    for body in forms {
        let api = MockApi::start(vec![Reply::hits()]).await;
        let addr = serve(real(&api)).await;
        let answer = post_typed(
            addr,
            "/api/search_passages",
            "application/x-www-form-urlencoded",
            body,
        )
        .await;
        assert_eq!(answer.status, Some(500), "{body}: {}", answer.body);
        assert_eq!(api.calls(), 0, "{body}: the API never saw it");
        assert!(!answer.everything().contains(CANARY), "{body}: leaked");
    }
}

/// T4, pinned: the wrong `Content-Type` alone is no refusal: the body is
/// read as the form (leptos 0.8), the answer is the outcome like any other.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn a_valid_form_under_text_plain_is_answered_like_any_other() {
    let _one = exclusive();
    let api = MockApi::start(vec![Reply::hits()]).await;
    let addr = serve(real(&api)).await;
    let answer = post_typed(addr, "/api/search_passages", "text/plain", &form("metta")).await;
    assert_eq!(answer.status, Some(200), "{}", answer.body);
    assert_eq!(api.calls(), 1);
    assert!(!answer.everything().contains(CANARY), "leaked");
}

/// T6 (review 005, round 3): an empty search box is a valid no-op: the
/// server returns 200, never asks the API, and does not set `Retry-After`
/// or `Cache-Control: no-store` (the home may be cached).
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn an_empty_search_box_is_a_valid_no_op() {
    let _one = exclusive();
    let api = MockApi::start(vec![Reply::hits()]).await;
    let addr = serve(real(&api)).await;
    let answer = post_form(
        addr,
        "/api/search_passages",
        "q=&lang=en&mode=both&collection=all",
    )
    .await;
    assert_eq!(answer.status, Some(200), "{}", answer.body);
    assert_eq!(api.calls(), 0, "the API was never called");
    assert_eq!(
        answer.header("retry-after"),
        None,
        "no retry-after for blank query"
    );
    assert_eq!(
        answer.header("cache-control"),
        None,
        "the home may be cached"
    );
    assert!(!answer.everything().contains(CANARY), "leaked");
}

/// The origin secret of the guard tests: no real one looks like it.
const SECRET: &str = "origin_canary_router_2e8c";

/// The mock source behind a guard that wants [`SECRET`].
async fn guarded_mock() -> SocketAddr {
    let origin = Origin::Checked(OriginSecret::new(SECRET));
    serve_with(Source::Mock(MockSource), origin).await
}

/// A refusal says nothing: a 403, an empty body, no echo, nothing cached.
fn assert_refused(answer: &Http, case: &str) {
    assert_eq!(answer.status, Some(403), "{case}: {}", answer.body);
    assert_eq!(answer.body, "", "{case}: the body is empty");
    assert_eq!(answer.header(ORIGIN_HEADER), None, "{case}: no echo");
    assert_eq!(answer.header("cache-control"), Some("no-store"), "{case}");
    assert!(!answer.everything().contains(SECRET), "{case}: leaked");
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn a_request_without_the_origin_header_is_a_403() {
    let _one = exclusive();
    let addr = guarded_mock().await;
    assert_refused(&get(addr, "/").await, "no header");
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn a_wrong_empty_or_partial_secret_is_a_403() {
    let _one = exclusive();
    let addr = guarded_mock().await;
    let prefix = &SECRET[..SECRET.len() - 1];
    let longer = format!("{SECRET}x");
    let upper = SECRET.to_ascii_uppercase();
    let values = [
        ("wrong", "not_the_secret_at_all_"),
        ("empty", ""),
        ("a prefix", prefix),
        ("longer", longer.as_str()),
        ("other case", upper.as_str()),
    ];
    for (case, value) in values {
        let answer = get_with(addr, "/", &[(ORIGIN_HEADER, value)]).await;
        assert_refused(&answer, case);
    }
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn two_origin_headers_are_a_403_even_when_one_is_right() {
    let _one = exclusive();
    let addr = guarded_mock().await;
    for pair in [
        [(ORIGIN_HEADER, SECRET), (ORIGIN_HEADER, "other")],
        [(ORIGIN_HEADER, "other"), (ORIGIN_HEADER, SECRET)],
        [(ORIGIN_HEADER, SECRET), (ORIGIN_HEADER, SECRET)],
    ] {
        assert_refused(&get_with(addr, "/", &pair).await, "two headers");
    }
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn the_right_secret_gets_the_page_and_the_server_function() {
    let _one = exclusive();
    let addr = guarded_mock().await;
    let header = [(ORIGIN_HEADER, SECRET)];
    let page = get_with(addr, "/?q=mindfulness", &header).await;
    assert_eq!(page.status, Some(200), "{}", page.body);
    assert!(page.body.contains("MN 10"), "{}", page.body);
    let answer = post_form_with(addr, "/api/search_passages", &form("mindfulness"), &header).await;
    assert_eq!(answer.status, Some(200), "{}", answer.body);
    assert!(answer.body.contains("mn10"), "{}", answer.body);
    let missing = get_with(addr, "/no/such/page", &header).await;
    assert_eq!(missing.status, Some(404), "the guard lets the 404 through");
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn a_search_without_the_header_asks_the_api_nothing() {
    let _one = exclusive();
    let api = MockApi::start(vec![Reply::hits()]).await;
    let origin = Origin::Checked(OriginSecret::new(SECRET));
    let addr = serve_with(real(&api), origin).await;
    let answer = post_form_with(addr, "/api/search_passages", &form("metta"), &[]).await;
    assert_refused(&answer, "server function");
    let page = get(addr, "/?q=metta").await;
    assert_refused(&page, "search page");
    assert_eq!(api.calls(), 0, "zero calls to the API");
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn the_static_files_and_unknown_paths_are_refused_too() {
    let _one = exclusive();
    let addr = guarded_mock().await;
    for path in [
        "/pkg/search-front.js",
        "/pkg/search-front.wasm",
        "/favicon.ico",
        "/no/such/page",
        "/api/search_passages",
        "/api/unknown",
    ] {
        assert_refused(&get(addr, path).await, path);
    }
}

/// Pair: guard x body limit. Without the header an oversized body is a 403,
/// with it the limit answers as before (a 500, zero calls to the API).
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn the_guard_comes_before_the_body_limit() {
    let _one = exclusive();
    let api = MockApi::start(vec![Reply::hits()]).await;
    let origin = Origin::Checked(OriginSecret::new(SECRET));
    let addr = serve_with(real(&api), origin).await;
    let huge = form(&"a".repeat(64 * 1024));
    let refused = post_form_with(addr, "/api/search_passages", &huge, &[]).await;
    assert_eq!(refused.status, Some(403), "{}", refused.body);
    let header = [(ORIGIN_HEADER, SECRET)];
    let limited = post_form_with(addr, "/api/search_passages", &huge, &header).await;
    assert_eq!(limited.status, Some(500), "{}", limited.body);
    assert_eq!(api.calls(), 0, "the API never saw either");
}

/// Pair: guard x token. A page served to the right header holds neither the
/// API token nor the origin secret.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn the_page_holds_neither_the_api_token_nor_the_origin_secret() {
    let _one = exclusive();
    let api = MockApi::start(vec![Reply::hits(), Reply::hits()]).await;
    let origin = Origin::Checked(OriginSecret::new(SECRET));
    let addr = serve_with(real(&api), origin).await;
    let header = [(ORIGIN_HEADER, SECRET)];
    let page = get_with(addr, "/?q=metta", &header).await;
    assert_eq!(page.status, Some(200), "{}", page.body);
    let answer = post_form_with(addr, "/api/search_passages", &form("metta"), &header).await;
    assert_eq!(answer.status, Some(200), "{}", answer.body);
    for (what, text) in [("page", page.everything()), ("fn", answer.everything())] {
        assert!(!text.contains(CANARY), "{what}: API token leaked");
        assert!(!text.contains(SECRET), "{what}: origin secret leaked");
    }
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn an_open_origin_checks_nothing() {
    let _one = exclusive();
    let addr = mock_server().await;
    let page = get(addr, "/").await;
    assert_eq!(page.status, Some(200), "{}", page.body);
}
