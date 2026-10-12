//! The `search_passages` server function: the page's only search path (slice 5 of task 005).
//!
//! On the server the function asks the configured `Source` (provided as an
//! `Arc<Source>` context by the router); in the browser it is an HTTP call
//! to `POST /api/search_passages`, so the WASM holds neither the mock nor a
//! token. It always answers `Ok(`[`SearchOutcome`]`)`: a cold engine, a
//! rate limit or a failure is a state the page renders, and a `ServerFnError`
//! would be a 500 on the browser path. Its arguments are bounded before the
//! source sees them (`Request::from_args`): the browser's `maxlength` is no
//! guard, the server is.

// Under `hydrate` the `#[server]` macro leaves a client stub with no `.await`
// in it: a lint the macro's own output trips, which an attribute on the fn
// does not reach.
#![cfg_attr(not(feature = "ssr"), allow(clippy::unused_async_trait_impl))]

use leptos::prelude::*;

use super::SearchOutcome;
#[cfg(feature = "ssr")]
use super::source::Source;
#[cfg(feature = "ssr")]
use super::{Collection, Mode, SearchResponse, SearchSource};

/// The longest query the server searches, in characters (the search input's
/// `maxlength`; the server enforces it for a caller that skips the input).
#[cfg(feature = "ssr")]
pub(crate) const MAX_QUERY_CHARS: usize = 200;

/// The languages the API searches; anything else is English.
#[cfg(feature = "ssr")]
const LANGS: [&str; 3] = ["en", "fr", "vi"];

/// A search as the server asks it, after the bounds.
#[cfg(feature = "ssr")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Request {
    /// The query: trimmed, not blank, at most [`MAX_QUERY_CHARS`] characters.
    pub(crate) q: String,
    /// `en`, `fr` or `vi`.
    pub(crate) lang: String,
    /// The mode; an unknown code is [`Mode::Both`].
    pub(crate) mode: Mode,
    /// The collection; an unknown code is [`Collection::All`].
    pub(crate) collection: Collection,
}

#[cfg(feature = "ssr")]
impl Request {
    /// Applies the bounds to the raw arguments of the server function:
    /// `q` is trimmed, a blank one is no search (`None`), a long one is cut
    /// to [`MAX_QUERY_CHARS`] *characters* (never inside a character); an
    /// unknown `lang` is `en`, an unknown `mode` is both, an unknown
    /// `collection` is all (a stray value is a filter to default, never a
    /// refusal).
    pub(crate) fn from_args(q: &str, lang: &str, mode: &str, collection: &str) -> Option<Self> {
        let q = q.trim();
        if q.is_empty() {
            return None;
        }
        let lang = if LANGS.contains(&lang) { lang } else { "en" };
        Some(Request {
            q: q.chars().take(MAX_QUERY_CHARS).collect(),
            lang: lang.to_owned(),
            mode: Mode::parse(mode),
            collection: Collection::parse(collection),
        })
    }
}

/// Searches `source` for the bounded arguments: a blank query answers an
/// empty `Hits` and asks nothing. Never fails.
#[cfg(feature = "ssr")]
async fn search_passages_with(
    source: &Source,
    q: &str,
    lang: &str,
    mode: &str,
    collection: &str,
) -> SearchOutcome {
    let Some(request) = Request::from_args(q, lang, mode, collection) else {
        return SearchOutcome::Hits(SearchResponse::empty(q.trim()));
    };
    source
        .search(&request.q, &request.lang, request.mode, request.collection)
        .await
}

/// The log line of a missing source: a boot bug, distinct from an upstream
/// failure (both answer the page 502). Same channel as the other lines of
/// the search path (`real.rs`: stderr); no query, no token.
#[cfg(feature = "ssr")]
pub(crate) const MISSING_SOURCE_LOG: &str = "search: source missing";

/// The source the router put in the context. The error text names neither
/// a token nor a query: it reaches the browser as the 500's body.
#[cfg(feature = "ssr")]
fn context_source() -> Result<std::sync::Arc<Source>, ServerFnError> {
    source_or_log(use_context::<std::sync::Arc<Source>>(), |line| {
        eprintln!("{line}");
    })
}

/// `found`, or the error of a missing source after `log`ging
/// [`MISSING_SOURCE_LOG`] (the logger is given so a test can read it).
#[cfg(feature = "ssr")]
fn source_or_log(
    found: Option<std::sync::Arc<Source>>,
    log: impl FnOnce(&str),
) -> Result<std::sync::Arc<Source>, ServerFnError> {
    found.ok_or_else(|| {
        log(MISSING_SOURCE_LOG);
        ServerFnError::new("search source missing")
    })
}

/// The server function's body, with the source given: the outcome is always
/// `Ok` — only a missing source is an error.
#[cfg(feature = "ssr")]
async fn answer(
    source: Result<std::sync::Arc<Source>, ServerFnError>,
    q: &str,
    lang: &str,
    mode: &str,
    collection: &str,
) -> Result<SearchOutcome, ServerFnError> {
    let source = source?;
    Ok(search_passages_with(&source, q, lang, mode, collection).await)
}

/// Searches the passages: the page's only search path. `lang` is `en`,
/// `fr` or `vi`, `mode` is `words`, `meaning` or `both`, `collection` is a
/// nikāya code or `all` — each defaulted when unknown; `q` is trimmed and
/// capped at 200 characters, and a blank one asks nothing. A search that
/// does not answer is `Ok(Cold | RateLimited | Failed)`, never an error.
///
/// # Errors
/// Only when the server has no search source in its context (a boot bug).
#[server(endpoint = "search_passages")]
pub async fn search_passages(
    /// What the visitor typed.
    q: String,
    /// The reader's language: `en`, `fr` or `vi`.
    lang: String,
    /// `words`, `meaning` or `both`.
    mode: String,
    /// A nikāya code, or `all`.
    collection: String,
) -> Result<SearchOutcome, ServerFnError> {
    answer(context_source(), &q, &lang, &mode, &collection).await
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use super::*;
    use crate::search::real::{ApiToken, RealSource};
    use crate::search::test_support::{MockApi, Reply};

    const CANARY: &str = "pc_canary_token_5";

    fn real(api: &MockApi) -> Source {
        Source::Real(RealSource::new(
            api.url(),
            ApiToken::new(CANARY),
            Duration::from_secs(10),
        ))
    }

    async fn search(api: &MockApi, q: &str, lang: &str, mode: &str, c: &str) -> SearchOutcome {
        search_passages_with(&real(api), q, lang, mode, c).await
    }

    #[test]
    fn the_bounds_trim_default_and_cap_the_arguments() {
        assert_eq!(Request::from_args("   ", "en", "both", "all"), None);
        assert_eq!(Request::from_args("", "en", "both", "all"), None);
        let request = Request::from_args("  metta  ", "xx", "bogus", "zz").unwrap();
        assert_eq!(
            request,
            Request {
                q: "metta".to_owned(),
                lang: "en".to_owned(),
                mode: Mode::Both,
                collection: Collection::All,
            }
        );
        let request = Request::from_args("metta", "vi", "words", "mn").unwrap();
        assert_eq!(request.lang, "vi");
        assert_eq!(request.mode, Mode::Words);
        assert_eq!(request.collection, Collection::Mn);
    }

    #[test]
    fn the_cap_counts_characters_not_bytes() {
        let request = Request::from_args(&"ṭ".repeat(201), "en", "both", "all").unwrap();
        assert_eq!(request.q.chars().count(), MAX_QUERY_CHARS);
        assert_eq!(request.q, "ṭ".repeat(200), "the capped string is kept");
        let exact = "ṭ".repeat(200);
        assert_eq!(
            Request::from_args(&exact, "en", "both", "all").unwrap().q,
            exact
        );
    }

    #[tokio::test]
    async fn a_blank_query_is_an_empty_answer_and_asks_the_api_nothing() {
        let api = MockApi::start(vec![Reply::hits()]).await;
        for q in ["", " ", " \t\n "] {
            let outcome = search(&api, q, "en", "both", "all").await;
            assert_eq!(
                outcome,
                SearchOutcome::Hits(SearchResponse::empty("")),
                "{q:?}"
            );
        }
        assert_eq!(api.calls(), 0, "no API call for a blank query");
    }

    #[tokio::test]
    async fn a_long_query_reaches_the_api_as_200_characters() {
        let api = MockApi::start(vec![Reply::hits()]).await;
        search(&api, &"ṭ".repeat(201), "en", "both", "all").await;
        assert_eq!(api.calls(), 1);
        let received = &api.call(0).query["q"];
        assert_eq!(received.chars().count(), 200, "{received}");
        assert_eq!(received, &"ṭ".repeat(200));
    }

    #[tokio::test]
    async fn an_unknown_mode_lang_and_collection_reach_the_api_defaulted() {
        let api = MockApi::start(vec![Reply::hits()]).await;
        search(&api, "metta", "xx", "bogus", "zz").await;
        let call = api.call(0);
        assert_eq!(call.query["mode"], "hybrid");
        assert_eq!(call.query["lang"], "en");
        assert!(!call.query.contains_key("collections"), "{:?}", call.query);
    }

    #[tokio::test]
    async fn a_known_mode_lang_and_collection_reach_the_api_and_k_is_20() {
        let api = MockApi::start(vec![Reply::hits()]).await;
        search(&api, "metta", "fr", "words", "mn").await;
        let call = api.call(0);
        assert_eq!(call.query["mode"], "words");
        assert_eq!(call.query["lang"], "fr");
        assert_eq!(call.query["collections"], "mn");
        assert_eq!(call.query["k"], "20");
    }

    #[tokio::test]
    async fn an_answer_that_is_not_hits_comes_back_as_ok_never_an_error() {
        let cases = [
            (
                Reply::error(503, "engine_starting", "waking"),
                SearchOutcome::Cold { retry_after: None },
            ),
            (
                Reply::raw(429, "slow down").with_header("retry-after", "7"),
                SearchOutcome::RateLimited {
                    retry_after: Some(7),
                },
            ),
            (Reply::raw(500, "boom"), SearchOutcome::Failed),
        ];
        for (reply, expected) in cases {
            let api = MockApi::start(vec![reply]).await;
            let source = Arc::new(real(&api));
            let outcome = answer(Ok(source), "metta", "en", "both", "all").await;
            assert_eq!(outcome.ok(), Some(expected));
        }
    }

    #[tokio::test]
    async fn a_missing_source_is_the_only_error_and_its_text_leaks_nothing() {
        let error = answer(context_source(), "secret query", "en", "both", "all")
            .await
            .unwrap_err();
        let shown = error.to_string();
        assert!(shown.contains("search source missing"), "{shown}");
        assert!(!shown.contains(CANARY), "{shown}");
        assert!(!shown.contains("secret query"), "{shown}");
    }

    #[test]
    fn a_provided_source_is_found_in_the_context() {
        let owner = Owner::new();
        owner.set();
        provide_context(Arc::new(Source::Mock(crate::search::mock::MockSource)));
        assert!(context_source().is_ok());
    }

    /// T5: a missing source is loud on the server's log, once, and a source
    /// found logs nothing.
    #[test]
    fn a_missing_source_is_logged_and_a_found_one_is_not() {
        let mut lines = Vec::new();
        let missing = source_or_log(None, |line| lines.push(line.to_owned()));
        assert!(missing.is_err());
        assert_eq!(lines, ["search: source missing"], "one distinct line");
        let mut lines = Vec::new();
        let found = source_or_log(
            Some(Arc::new(Source::Mock(crate::search::mock::MockSource))),
            |l| {
                lines.push(l.to_owned());
            },
        );
        assert!(found.is_ok());
        assert_eq!(lines.len(), 0, "{lines:?}");
    }
}
