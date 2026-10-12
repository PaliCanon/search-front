//! `RealSource`: the search through `palicanon-api-client` (slice 3 of task 005).
//!
//! One call to the API per search, inside a timeout, its answer mapped to a
//! [`SearchOutcome`] (never an error): the engine waking (or a call that
//! outlived its timeout) is `Cold`, the edge's 429 is `RateLimited`, anything
//! else is `Failed`. A language the API has no index for (`404
//! lang_unavailable`) is searched again once in English, and the answer says
//! so (`fell_back`). The token lives in [`ApiToken`], whose `Debug` is
//! redacted; a failure is logged as a status, an API code and a duration,
//! never the query, the API's message or the token.

use std::fmt;
use std::time::{Duration, Instant};

use palicanon_api_client::{Client, ClientError, SearchQuery};

use super::{Collection, Mode, SearchFailure, SearchOutcome, SearchSource, adapt};

/// How long production waits for the API: the engine takes up to a minute
/// to wake, as the current site knows (ruled at Plan, 005).
pub(crate) const API_TIMEOUT: Duration = Duration::from_secs(30);

/// The `retry_after` a timeout is answered with: the engine is waking.
const TIMEOUT_RETRY_AFTER: u64 = 10;

/// Hits asked of the API, whatever the browser sends.
const HITS_WANTED: usize = 20;

/// The API's token. Its `Debug` never shows it.
#[derive(Clone)]
pub struct ApiToken(String);

impl ApiToken {
    /// Wraps a token read from the environment at boot.
    pub(crate) fn new(token: impl Into<String>) -> Self {
        ApiToken(token.into())
    }
}

impl fmt::Debug for ApiToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiToken(<redacted>)")
    }
}

/// Searches through the PaliCanon API.
pub struct RealSource {
    client: Client,
    timeout: Duration,
}

impl RealSource {
    /// A source of the API at `base_url`, authenticated with `token`, giving
    /// up on a call after `timeout` ([`API_TIMEOUT`] in production).
    pub(crate) fn new(base_url: &str, token: ApiToken, timeout: Duration) -> Self {
        RealSource {
            client: Client::new(base_url, token.0),
            timeout,
        }
    }
}

impl fmt::Debug for RealSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RealSource")
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

/// What went wrong with one call, for the log.
pub(crate) enum Fault<'a> {
    /// The client's error.
    Client(&'a ClientError),
    /// The call outlived its timeout.
    Timeout,
}

/// The log line of a failed call: its status, the API's error code and the
/// duration. Never the query, never the API's message, never the token — and
/// never the client's own text for a failed request, which names the URL.
pub(crate) fn log_line(fault: &Fault<'_>, elapsed: Duration) -> String {
    let what = match fault {
        Fault::Timeout => "timeout".to_owned(),
        Fault::Client(ClientError::Unauthorized(_)) => {
            "status 401 (the token was refused)".to_owned()
        }
        Fault::Client(ClientError::RateLimited { .. }) => "status 429".to_owned(),
        Fault::Client(ClientError::ColdEngine { .. }) => "status 503 engine_starting".to_owned(),
        Fault::Client(ClientError::Other { status, code, .. }) => {
            format!("status {status} {}", clean_code(code))
        }
        Fault::Client(ClientError::Request(_)) => {
            "request failed (no answer or not JSON)".to_owned()
        }
    };
    format!("search: api {what} after {} ms", elapsed.as_millis())
}

/// An API error code made safe for a log line: `[a-z0-9_]`, at most 40.
fn clean_code(code: &str) -> String {
    code.chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')
        .take(40)
        .collect()
}

/// Why one call did not give hits.
enum Failure {
    /// A state the page shows.
    Typed(SearchFailure),
    /// `404 lang_unavailable`: the language has no index.
    LangUnavailable,
}

/// Whole seconds of a `retry-after`, rounded up.
fn seconds(after: Option<Duration>) -> Option<u64> {
    after.map(|d| d.as_secs() + u64::from(d.subsec_nanos() > 0))
}

/// The client's error as the page's failure.
fn classify(err: ClientError) -> Failure {
    match err {
        ClientError::ColdEngine { retry_after } => Failure::Typed(SearchFailure::Cold {
            retry_after: seconds(retry_after),
        }),
        ClientError::RateLimited { retry_after } => Failure::Typed(SearchFailure::RateLimited {
            retry_after: seconds(retry_after),
        }),
        ClientError::Other {
            status: 404, code, ..
        } if code == "lang_unavailable" => Failure::LangUnavailable,
        ClientError::Unauthorized(_) | ClientError::Other { .. } | ClientError::Request(_) => {
            Failure::Typed(SearchFailure::Failed)
        }
    }
}

impl RealSource {
    /// One call, under the timeout; a failure is logged here.
    async fn once(
        &self,
        query: &str,
        lang: &str,
        mode: Mode,
        collection: Collection,
    ) -> Result<palicanon_api_client::SearchResponse, Failure> {
        let mut wire = SearchQuery::new(query)
            .lang(lang)
            .k(HITS_WANTED)
            .mode(adapt::mode_wire_code(mode));
        if let Some(code) = adapt::collection_wire_code(collection) {
            wire = wire.collections(code);
        }
        let started = Instant::now();
        let answer = tokio::time::timeout(self.timeout, self.client.search(&wire)).await;
        match answer {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(err)) => {
                eprintln!("{}", log_line(&Fault::Client(&err), started.elapsed()));
                Err(classify(err))
            }
            Err(_elapsed) => {
                eprintln!("{}", log_line(&Fault::Timeout, started.elapsed()));
                Err(Failure::Typed(SearchFailure::Cold {
                    retry_after: Some(TIMEOUT_RETRY_AFTER),
                }))
            }
        }
    }
}

impl SearchSource for RealSource {
    async fn search(
        &self,
        query: &str,
        lang: &str,
        mode: Mode,
        collection: Collection,
    ) -> SearchOutcome {
        let first = self.once(query, lang, mode, collection).await;
        let (result, fell_back) = match first {
            Err(Failure::LangUnavailable) if lang != "en" => {
                (self.once(query, "en", mode, collection).await, true)
            }
            other => (other, false),
        };
        match result {
            Ok(wire) => SearchOutcome::Hits(adapt::response(wire, fell_back)),
            Err(Failure::Typed(failure)) => failure.into(),
            Err(Failure::LangUnavailable) => SearchFailure::Failed.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::test_support::{MockApi, Reply};
    use palicanon_api_client::SearchQuery;

    /// A token no real one looks like: it must reach the API's
    /// `Authorization` header and nothing else.
    const CANARY_TOKEN: &str = "pc_canary_5f3a9c7e";
    /// A query text that must never reach a log line.
    const CANARY_QUERY: &str = "canaryquery-b7e1";
    /// A generous timeout for the tests that are not about it.
    const LONG: Duration = Duration::from_secs(10);

    fn source(api: &MockApi, timeout: Duration) -> RealSource {
        RealSource::new(api.url(), ApiToken::new(CANARY_TOKEN), timeout)
    }

    async fn ask(api: &MockApi, lang: &str, mode: Mode, collection: Collection) -> SearchOutcome {
        source(api, LONG)
            .search("metta", lang, mode, collection)
            .await
    }

    async fn ask_en(api: &MockApi) -> SearchOutcome {
        ask(api, "en", Mode::Both, Collection::All).await
    }

    #[tokio::test]
    async fn a_full_wire_body_becomes_the_hits() {
        let api = MockApi::start(vec![Reply::hits()]).await;
        let SearchOutcome::Hits(res) = ask_en(&api).await else {
            panic!("expected Hits");
        };
        assert_eq!(res.hits.len(), 1, "one hit in the wire body");
        assert_eq!(res.hits[0].sutta_uid, "mn10");
        assert_eq!(res.hits[0].title, "Satipatthana Sutta");
        assert_eq!(res.query, "metta");
        assert!(!res.fell_back, "the first call answered: no fallback");
    }

    #[tokio::test]
    async fn a_passage_hit_comes_back_with_its_read_link_anchored_on_the_snippet_key() {
        let api = MockApi::start(vec![Reply::hits()]).await;
        let SearchOutcome::Hits(res) = ask_en(&api).await else {
            panic!("expected Hits");
        };
        assert!(
            res.hits[0].url.ends_with("#mn10:3.2"),
            "the adapter anchors a passage on its snippet_key: {}",
            res.hits[0].url
        );
    }

    #[tokio::test]
    async fn the_api_receives_the_query_k_mode_lang_and_the_bearer_token() {
        let api = MockApi::start(vec![Reply::hits()]).await;
        ask(&api, "fr", Mode::Both, Collection::All).await;
        assert_eq!(api.calls(), 1);
        let call = api.call(0);
        assert_eq!(call.query.get("q").map(String::as_str), Some("metta"));
        assert_eq!(call.query.get("k").map(String::as_str), Some("20"));
        assert_eq!(call.query.get("mode").map(String::as_str), Some("hybrid"));
        assert_eq!(call.query.get("lang").map(String::as_str), Some("fr"));
        assert_eq!(
            call.authorization.as_deref(),
            Some(format!("Bearer {CANARY_TOKEN}").as_str())
        );
    }

    #[tokio::test]
    async fn the_words_and_meaning_modes_are_sent_as_themselves() {
        let api = MockApi::start(vec![Reply::hits()]).await;
        ask(&api, "en", Mode::Words, Collection::All).await;
        ask(&api, "en", Mode::Meaning, Collection::All).await;
        assert_eq!(
            api.call(0).query.get("mode").map(String::as_str),
            Some("words")
        );
        assert_eq!(
            api.call(1).query.get("mode").map(String::as_str),
            Some("meaning")
        );
    }

    #[tokio::test]
    async fn all_collections_sends_no_collections_parameter() {
        let api = MockApi::start(vec![Reply::hits()]).await;
        ask(&api, "en", Mode::Both, Collection::All).await;
        assert_eq!(
            api.call(0).query.get("collections"),
            None,
            "`all` is not an API code: no narrowing is asked by sending nothing"
        );
    }

    #[tokio::test]
    async fn a_nikaya_is_sent_as_its_code() {
        let api = MockApi::start(vec![Reply::hits()]).await;
        ask(&api, "en", Mode::Both, Collection::Mn).await;
        assert_eq!(
            api.call(0).query.get("collections").map(String::as_str),
            Some("mn")
        );
    }

    #[tokio::test]
    async fn a_401_is_failed() {
        let api = MockApi::start(vec![Reply::error(401, "unauthorized", "bad token")]).await;
        assert_eq!(ask_en(&api).await, SearchOutcome::Failed);
    }

    #[tokio::test]
    async fn a_429_with_retry_after_is_rate_limited_for_that_long() {
        let api = MockApi::start(vec![
            Reply::raw(429, "slow down").with_header("retry-after", "7"),
        ])
        .await;
        assert_eq!(
            ask_en(&api).await,
            SearchOutcome::RateLimited {
                retry_after: Some(7)
            }
        );
    }

    #[tokio::test]
    async fn a_429_without_retry_after_is_rate_limited_with_none() {
        let api = MockApi::start(vec![Reply::raw(429, "slow down")]).await;
        assert_eq!(
            ask_en(&api).await,
            SearchOutcome::RateLimited { retry_after: None }
        );
    }

    #[tokio::test]
    async fn a_503_engine_starting_is_cold_with_its_retry_after() {
        let api = MockApi::start(vec![
            Reply::error(503, "engine_starting", "warming up").with_header("retry-after", "4"),
        ])
        .await;
        assert_eq!(
            ask_en(&api).await,
            SearchOutcome::Cold {
                retry_after: Some(4)
            }
        );
    }

    #[tokio::test]
    async fn a_503_engine_starting_without_retry_after_is_cold_with_none() {
        let api = MockApi::start(vec![Reply::error(503, "engine_starting", "warming up")]).await;
        assert_eq!(
            ask_en(&api).await,
            SearchOutcome::Cold { retry_after: None }
        );
    }

    #[tokio::test]
    async fn a_503_with_another_code_is_failed() {
        let api = MockApi::start(vec![Reply::error(503, "search_timeout", "too slow")]).await;
        assert_eq!(ask_en(&api).await, SearchOutcome::Failed);
    }

    #[tokio::test]
    async fn a_500_is_failed() {
        let api = MockApi::start(vec![Reply::error(500, "internal", "boom")]).await;
        assert_eq!(ask_en(&api).await, SearchOutcome::Failed);
    }

    #[tokio::test]
    async fn a_200_that_is_not_json_is_failed() {
        let api = MockApi::start(vec![Reply::raw(200, "not json")]).await;
        assert_eq!(ask_en(&api).await, SearchOutcome::Failed);
    }

    #[tokio::test]
    async fn an_unreachable_api_is_failed() {
        let source = RealSource::new("http://127.0.0.1:0", ApiToken::new(CANARY_TOKEN), LONG);
        let outcome = source
            .search("metta", "en", Mode::Both, Collection::All)
            .await;
        assert_eq!(outcome, SearchOutcome::Failed);
    }

    #[tokio::test]
    async fn a_server_that_never_answers_is_cold_after_the_timeout() {
        let api = MockApi::start(vec![Reply::Hang]).await;
        let outcome = source(&api, Duration::from_millis(100))
            .search("metta", "en", Mode::Both, Collection::All)
            .await;
        assert_eq!(
            outcome,
            SearchOutcome::Cold {
                retry_after: Some(10)
            },
            "the engine takes up to a minute to wake: a timeout is Cold, not Failed"
        );
        assert_eq!(api.calls(), 1, "a timeout is not retried");
    }

    #[tokio::test]
    async fn an_unavailable_language_is_searched_again_in_english_and_says_so() {
        let api = MockApi::start(vec![
            Reply::error(404, "lang_unavailable", "no store for vi"),
            Reply::hits(),
        ])
        .await;
        let SearchOutcome::Hits(res) = ask(&api, "fr", Mode::Both, Collection::All).await else {
            panic!("expected Hits from the English retry");
        };
        assert_eq!(api.calls(), 2, "the language, then English");
        assert_eq!(
            api.call(0).query.get("lang").map(String::as_str),
            Some("fr")
        );
        assert_eq!(
            api.call(1).query.get("lang").map(String::as_str),
            Some("en")
        );
        assert_eq!(
            api.call(1).query.get("q").map(String::as_str),
            Some("metta")
        );
        assert!(res.fell_back, "the page must say English was searched");
        assert_eq!(res.hits.len(), 1);
    }

    #[tokio::test]
    async fn the_english_retry_keeps_the_mode_and_the_collection() {
        let api = MockApi::start(vec![
            Reply::error(404, "lang_unavailable", "no store"),
            Reply::hits(),
        ])
        .await;
        ask(&api, "vi", Mode::Words, Collection::Sn).await;
        let retry = api.call(1);
        assert_eq!(retry.query.get("mode").map(String::as_str), Some("words"));
        assert_eq!(
            retry.query.get("collections").map(String::as_str),
            Some("sn")
        );
    }

    #[tokio::test]
    async fn english_unavailable_is_failed_without_a_second_call() {
        let api = MockApi::start(vec![Reply::error(404, "lang_unavailable", "no store")]).await;
        assert_eq!(ask_en(&api).await, SearchOutcome::Failed);
        assert_eq!(api.calls(), 1, "no loop when already English");
    }

    #[tokio::test]
    async fn a_failed_english_retry_stops_there() {
        let api = MockApi::start(vec![Reply::error(404, "lang_unavailable", "no store")]).await;
        let outcome = ask(&api, "fr", Mode::Both, Collection::All).await;
        assert_eq!(outcome, SearchOutcome::Failed);
        assert_eq!(api.calls(), 2, "one retry, never a third call");
    }

    #[tokio::test]
    async fn a_404_with_another_code_is_failed_without_a_retry() {
        let api = MockApi::start(vec![Reply::error(404, "not_found", "no route")]).await;
        let outcome = ask(&api, "fr", Mode::Both, Collection::All).await;
        assert_eq!(outcome, SearchOutcome::Failed);
        assert_eq!(api.calls(), 1);
    }

    #[test]
    fn the_token_debug_never_shows_the_token() {
        let shown = format!("{:?}", ApiToken::new(CANARY_TOKEN));
        assert!(!shown.contains(CANARY_TOKEN), "leaked: {shown}");
        assert_eq!(shown, "ApiToken(<redacted>)");
    }

    #[tokio::test]
    async fn the_source_debug_never_shows_the_token() {
        let api = MockApi::start(vec![Reply::hits()]).await;
        let shown = format!("{:?}", source(&api, LONG));
        assert!(!shown.contains(CANARY_TOKEN), "leaked: {shown}");
    }

    const SOME_MS: Duration = Duration::from_millis(42);

    #[test]
    fn the_log_line_of_an_api_error_has_its_status_and_code_but_not_its_message() {
        let err = ClientError::Other {
            status: 500,
            code: "internal".to_owned(),
            message: format!("MESSAGE q={CANARY_QUERY} token={CANARY_TOKEN}"),
        };
        let line = log_line(&Fault::Client(&err), SOME_MS);
        assert!(line.contains("500"), "the status: {line}");
        assert!(line.contains("internal"), "the API's code: {line}");
        assert!(line.contains("42"), "the duration in ms: {line}");
        for secret in ["MESSAGE", CANARY_QUERY, CANARY_TOKEN] {
            assert!(!line.contains(secret), "{secret} leaked: {line}");
        }
    }

    #[test]
    fn the_log_line_of_a_refused_token_names_the_status_not_the_message() {
        let err = ClientError::Unauthorized(format!("unknown token {CANARY_TOKEN}"));
        let line = log_line(&Fault::Client(&err), SOME_MS);
        assert!(line.contains("401"), "the status: {line}");
        assert!(!line.contains(CANARY_TOKEN), "token leaked: {line}");
    }

    #[test]
    fn the_log_line_of_the_typed_refusals_has_their_status() {
        let cold = ClientError::ColdEngine { retry_after: None };
        let limited = ClientError::RateLimited { retry_after: None };
        assert!(log_line(&Fault::Client(&cold), SOME_MS).contains("503"));
        assert!(log_line(&Fault::Client(&limited), SOME_MS).contains("429"));
    }

    #[test]
    fn the_log_line_of_a_timeout_says_so() {
        let line = log_line(&Fault::Timeout, SOME_MS);
        assert!(line.contains("timeout"), "{line}");
        assert!(line.contains("42"), "the duration in ms: {line}");
    }

    #[tokio::test]
    async fn the_log_line_of_a_failed_request_never_shows_the_query() {
        // reqwest's own text for this error names the URL, query included.
        let err = Client::new("http://127.0.0.1:0", CANARY_TOKEN)
            .search(&SearchQuery::new(CANARY_QUERY))
            .await
            .unwrap_err();
        assert!(matches!(err, ClientError::Request(_)), "got {err:?}");
        let line = log_line(&Fault::Client(&err), SOME_MS);
        assert!(line.contains("request"), "it says what failed: {line}");
        for secret in [CANARY_QUERY, CANARY_TOKEN] {
            assert!(!line.contains(secret), "{secret} leaked: {line}");
        }
    }
}
