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

/// The router over `source`, serving on a free local port of this runtime.
async fn serve(source: Source) -> SocketAddr {
    let options = LeptosOptions::builder().output_name("search-front").build();
    let app = router(options, Arc::new(source));
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
