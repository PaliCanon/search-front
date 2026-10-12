//! A mock API on a local port for the real source's tests (slice 3 of task
//! 005): the `GET /v1/search` of `api.palicanon.app` reduced to canned
//! replies. It answers the n-th call with the n-th reply (the last one
//! repeats), and keeps what each call carried — its query string and its
//! `Authorization` header — so a test reads what the API would have seen.
//! Error replies have the API's own body shape,
//! `{"error": "<code>", "message": "…"}` (`palicanon-api-client`'s tests).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde_json::{Value, json};

/// One canned answer of the mock API.
pub(crate) enum Reply {
    /// A status, headers and a body.
    Answer {
        /// The HTTP status.
        status: u16,
        /// Extra response headers (`retry-after`).
        headers: Vec<(&'static str, String)>,
        /// The body, sent as is.
        body: String,
    },
    /// The request is accepted and never answered (the timeout's case).
    Hang,
}

impl Reply {
    /// `status` with a JSON body.
    pub(crate) fn json(status: u16, body: &Value) -> Self {
        Reply::raw(status, &body.to_string())
    }

    /// `status` with a body that need not be JSON.
    pub(crate) fn raw(status: u16, body: &str) -> Self {
        Reply::Answer {
            status,
            headers: Vec::new(),
            body: body.to_owned(),
        }
    }

    /// A refusal in the API's own shape: `{"error": code, "message": …}`.
    pub(crate) fn error(status: u16, code: &str, message: &str) -> Self {
        Reply::json(status, &json!({ "error": code, "message": message }))
    }

    /// A 200 with [`full_wire_body`].
    pub(crate) fn hits() -> Self {
        Reply::json(200, &full_wire_body())
    }

    /// The same reply with one more response header.
    #[must_use]
    pub(crate) fn with_header(self, name: &'static str, value: &str) -> Self {
        match self {
            Reply::Answer {
                status,
                mut headers,
                body,
            } => {
                headers.push((name, value.to_owned()));
                Reply::Answer {
                    status,
                    headers,
                    body,
                }
            }
            Reply::Hang => Reply::Hang,
        }
    }
}

/// What one call carried.
#[derive(Clone, Debug)]
pub(crate) struct Call {
    /// The decoded query-string pairs (`q`, `lang`, `k`, `mode`, …).
    pub(crate) query: HashMap<String, String>,
    /// The `Authorization` header, when there was one.
    pub(crate) authorization: Option<String>,
}

/// The replies and what the calls carried, shared with the server task.
struct Shared {
    replies: Vec<Reply>,
    calls: Mutex<Vec<Call>>,
}

/// A running mock API: its address, and what it has been asked.
pub(crate) struct MockApi {
    url: String,
    shared: Arc<Shared>,
}

impl MockApi {
    /// Starts the server on a free port of this runtime; its n-th call is
    /// answered by `replies[n]`, the last reply repeating.
    pub(crate) async fn start(replies: Vec<Reply>) -> Self {
        assert!(!replies.is_empty(), "a mock API needs at least one reply");
        let shared = Arc::new(Shared {
            replies,
            calls: Mutex::new(Vec::new()),
        });
        let app = Router::new()
            .route("/v1/search", get(handle))
            .with_state(shared.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a free local port");
        let url = format!("http://{}", listener.local_addr().expect("a bound address"));
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("the mock API serves");
        });
        MockApi { url, shared }
    }

    /// The base URL, without a trailing slash.
    pub(crate) fn url(&self) -> &str {
        &self.url
    }

    /// How many calls reached the API.
    pub(crate) fn calls(&self) -> usize {
        self.shared.calls.lock().expect("calls lock").len()
    }

    /// The `n`-th call (from 0).
    pub(crate) fn call(&self, n: usize) -> Call {
        self.shared.calls.lock().expect("calls lock")[n].clone()
    }
}

/// `GET /v1/search`: records the call, then plays the reply of its rank.
async fn handle(
    State(shared): State<Arc<Shared>>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let rank = {
        let mut calls = shared.calls.lock().expect("calls lock");
        calls.push(Call {
            query,
            authorization: headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned),
        });
        calls.len() - 1
    };
    match &shared.replies[rank.min(shared.replies.len() - 1)] {
        Reply::Hang => std::future::pending().await,
        Reply::Answer {
            status,
            headers,
            body,
        } => answer(*status, headers, body),
    }
}

/// Builds the response of an [`Reply::Answer`].
fn answer(status: u16, headers: &[(&'static str, String)], body: &str) -> Response {
    let mut response = (
        StatusCode::from_u16(status).expect("a valid status"),
        [("content-type", "application/json")],
        body.to_owned(),
    )
        .into_response();
    for (name, value) in headers {
        response.headers_mut().insert(
            HeaderName::from_static(name),
            HeaderValue::from_str(value).expect("a header value"),
        );
    }
    response
}

/// A `/v1/search` answer with every field the API serves: one passage hit
/// with its titles, its anchor key and both branches.
pub(crate) fn full_wire_body() -> Value {
    json!({
        "query": "metta", "lang": "en", "detected_lang": null, "k": 20,
        "mode": "hybrid", "collections": [], "corrected_query": null,
        "took_ms": 3, "timings": { "embed_ms": 21, "search_ms": 2, "snippet_ms": 0 },
        "degraded": false, "cached": false,
        "hits": [{
            "sutta_uid": "mn10", "segment_key": "mn10:1.1", "display_id": "MN 10",
            "title": "Satipatthana Sutta", "pali_title": "Satipaṭṭhānasutta",
            "titles": { "en": "The Foundations of Mindfulness", "fr": "Les fondements", "vi": "Kinh Niệm Xứ" },
            "text": "Satipatthana Sutta", "snippet": "the four establishments of mindfulness",
            "snippet_key": "mn10:3.2", "snippet_kind": "passage", "pali": null,
            "url": "https://suttacentral.net/mn10", "score": 9.5,
            "sources": { "dense": true, "sparse": false, "reference": false }
        }]
    })
}
