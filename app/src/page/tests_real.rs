//! The page through the real path: the `Resource`, the server fn, the
//! source in the context, the states and the response (task 005, slice 6).

use std::sync::Arc;
use std::time::Duration;

use leptos_axum::{ResponseOptions, ResponseParts};
use palicanon_ui_lib::Language;

use super::harness::{Setup, render, render_at};
use crate::search::mock::{COLD_QUERY, ERROR_QUERY, RATE_LIMITED_QUERY};
use crate::search::real::{ApiToken, RealSource};
use crate::search::source::Source;
use crate::search::test_support::{MockApi, Reply, full_wire_body};

/// A source on the mock API: what it answers is no fixture of the mock
/// source, so a page that never asked it cannot show it.
fn real(api: &MockApi) -> Arc<Source> {
    Arc::new(Source::Real(RealSource::new(
        api.url(),
        ApiToken::new("pc_canary_token_6"),
        Duration::from_secs(10),
    )))
}

/// A 200 whose one hit carries a snippet only the API says.
fn canary_hits() -> Reply {
    let mut body = full_wire_body();
    body["hits"][0]["snippet"] = "a canary snippet only the API answers".into();
    Reply::json(200, &body)
}

/// A5: the unit render waits for the `Resource`: the hit the source in the
/// context answered is in the first HTML, and it was asked for once.
#[tokio::test]
async fn the_resource_resolves_in_the_first_html() {
    let api = MockApi::start(vec![canary_hits()]).await;
    let html = render(
        "/?q=metta",
        Setup {
            source: real(&api),
            ..Setup::default()
        },
    )
    .await;
    assert!(
        html.contains("a canary snippet only the API answers"),
        "the API's hit is in the first HTML: {html}"
    );
    assert_eq!(api.calls(), 1, "one search, one call");
}

/// Renders `path` with fresh response options; returns the HTML and what
/// the page set on them.
async fn render_with_options(path: &str) -> (String, ResponseParts) {
    let options = ResponseOptions::default();
    let html = render(
        path,
        Setup {
            options: Some(options.clone()),
            ..Setup::default()
        },
    )
    .await;
    let parts = options.0.read().expect("the options lock").clone();
    (html, parts)
}

/// One header's value, as text.
fn header<'a>(parts: &'a ResponseParts, name: &str) -> Option<&'a str> {
    parts.headers.get(name).and_then(|v| v.to_str().ok())
}

/// Every non-hit state, in every language: its string and an alert (the
/// mock's sentinels stand for the three outcomes, through the server fn).
#[tokio::test]
async fn each_state_shows_its_string_and_an_alert_in_every_language() {
    let states = [
        (
            COLD_QUERY,
            [
                "The search engine is still starting. Search again in a few seconds.",
                "Le moteur de recherche démarre encore. Relancez la recherche dans quelques secondes.",
                "Công cụ tìm kiếm vẫn đang khởi động. Hãy tìm lại sau vài giây.",
            ],
        ),
        (
            RATE_LIMITED_QUERY,
            [
                "Too many searches at once. Wait a moment and try again.",
                "Trop de recherches à la fois. Patientez un instant, puis réessayez.",
                "Có quá nhiều lượt tìm cùng lúc. Vui lòng chờ một chút rồi thử lại.",
            ],
        ),
        (
            ERROR_QUERY,
            [
                "Something went wrong",
                "Une erreur est survenue",
                "Đã có lỗi xảy ra",
            ],
        ),
    ];
    for (query, strings) in states {
        for (language, text) in Language::ALL.into_iter().zip(strings) {
            let html = crate::page::harness::render_in(language, &format!("/?q={query}")).await;
            assert!(html.contains(text), "{query} {language:?}: {html}");
            assert!(
                html.contains(r#"role="alert""#),
                "{query} {language:?}: {html}"
            );
        }
    }
}

/// The cold and rate-limited states link to the same GET, plain: the search
/// as the URL carries it, narrowed by its collection and mode.
#[tokio::test]
async fn the_cold_and_rate_limited_states_link_to_the_same_get() {
    for query in [COLD_QUERY, RATE_LIMITED_QUERY] {
        let html = render_at(&format!("/?q={query}&c=mn&mode=words")).await;
        let link = format!(r#"<a href="/?q={query}&amp;c=mn&amp;mode=words""#);
        assert!(html.contains(&link), "{query}: {html}");
    }
}

/// The `Failed` state is today's generic error, without a link.
#[tokio::test]
async fn the_failed_state_is_the_generic_error() {
    let html = render_at(&format!("/?q={ERROR_QUERY}")).await;
    assert!(html.contains("Something went wrong"), "{html}");
    assert!(!html.contains("Search again"), "{html}");
}

/// The route is rendered `SsrMode::Async`: the first HTML holds the hits.
#[tokio::test]
async fn the_search_route_is_rendered_in_async_mode() {
    use leptos_router::SsrMode;
    let _ = any_spawner::Executor::init_tokio();
    let routes = {
        let _one = crate::page::harness::exclusive();
        leptos_axum::generate_route_list(crate::app::App)
    };
    let home = routes
        .iter()
        .find(|route| route.path() == "/")
        .expect("the search route is listed");
    assert_eq!(home.mode(), &SsrMode::Async, "{routes:?}");
}

/// Cold: 503 and `Retry-After` (the mock's sentinel says 7 seconds).
#[tokio::test]
async fn a_cold_engine_answers_503_with_retry_after() {
    let (_, parts) = render_with_options(&format!("/?q={COLD_QUERY}")).await;
    assert_eq!(parts.status.map(|s| s.as_u16()), Some(503), "{parts:?}");
    assert!(header(&parts, "retry-after").is_some(), "{parts:?}");
    assert_eq!(header(&parts, "cache-control"), Some("no-store"));
}

/// Rate-limited: 429 and `Retry-After`.
#[tokio::test]
async fn a_rate_limit_answers_429_with_retry_after() {
    let (_, parts) = render_with_options(&format!("/?q={RATE_LIMITED_QUERY}")).await;
    assert_eq!(parts.status.map(|s| s.as_u16()), Some(429), "{parts:?}");
    assert!(header(&parts, "retry-after").is_some(), "{parts:?}");
    assert_eq!(header(&parts, "cache-control"), Some("no-store"));
}

/// Failed: 502, no `Retry-After`.
#[tokio::test]
async fn a_failure_answers_502() {
    let (_, parts) = render_with_options(&format!("/?q={ERROR_QUERY}")).await;
    assert_eq!(parts.status.map(|s| s.as_u16()), Some(502), "{parts:?}");
    assert_eq!(header(&parts, "retry-after"), None, "{parts:?}");
    assert_eq!(header(&parts, "cache-control"), Some("no-store"));
}

/// Hits keep the usual 200 (no status set) and are never cached.
#[tokio::test]
async fn hits_are_not_cached_and_keep_the_usual_status() {
    let (html, parts) = render_with_options("/?q=mindfulness").await;
    assert!(html.contains("MN 10"), "{html}");
    assert_eq!(parts.status, None, "{parts:?}");
    assert_eq!(header(&parts, "cache-control"), Some("no-store"));
    assert_eq!(header(&parts, "retry-after"), None, "{parts:?}");
}

/// The home and a blank `?q=` set nothing on the response.
#[tokio::test]
async fn the_home_and_a_blank_query_set_nothing() {
    for path in ["/", "/?q=", "/?q=%20%20"] {
        let (_, parts) = render_with_options(path).await;
        assert_eq!(parts.status, None, "{path}: {parts:?}");
        assert_eq!(header(&parts, "cache-control"), None, "{path}: {parts:?}");
        assert_eq!(header(&parts, "retry-after"), None, "{path}: {parts:?}");
    }
}

/// The hydrate path (and a bare render) has no `ResponseOptions`: a cold
/// outcome renders without one and does not panic.
#[tokio::test]
async fn a_cold_outcome_renders_with_no_response_options_in_context() {
    let html = render_at(&format!("/?q={COLD_QUERY}")).await;
    assert!(html.contains(r#"role="alert""#), "{html}");
}

/// The home asks the source nothing, blank or not: zero calls.
#[tokio::test]
async fn the_home_makes_zero_calls() {
    let api = MockApi::start(vec![canary_hits()]).await;
    for path in ["/", "/?q=", "/?q=%20%20", "/?c=mn&mode=words"] {
        let html = render(
            path,
            Setup {
                source: real(&api),
                ..Setup::default()
            },
        )
        .await;
        assert!(html.contains(">PaliCanon</h1>"), "{path}: {html}");
    }
    assert_eq!(api.calls(), 0, "the home never searches");
}

/// A pin of ui-lib's: both bars cap the query at 200 characters.
#[tokio::test]
async fn both_search_inputs_cap_the_query_at_200_characters() {
    for path in ["/", "/?q=metta"] {
        let html = render_at(path).await;
        assert!(html.contains(r#"maxlength="200""#), "{path}: {html}");
    }
}

/// The padded and the plain query are one search (the Resource's key is the
/// trimmed query): the API is asked once, with the same query, and the bar
/// shows the same value either way.
#[tokio::test]
async fn a_padded_query_and_its_trimmed_twin_are_one_search() {
    for path in ["/?q=%20mindfulness%20", "/?q=mindfulness"] {
        let api = MockApi::start(vec![canary_hits()]).await;
        let html = render(
            path,
            Setup {
                source: real(&api),
                ..Setup::default()
            },
        )
        .await;
        assert_eq!(api.calls(), 1, "{path}: one call");
        assert_eq!(api.call(0).query["q"], "mindfulness", "{path}");
        assert!(html.contains(r#"value="mindfulness""#), "{path}: {html}");
    }
}

/// The page's `Retry-After` for what a real API answered: the API's header,
/// parsed by the client into a `Duration`, then the adapter's whole seconds,
/// then the clamp.
async fn retry_after_for(reply: Reply) -> (u16, Option<String>) {
    let api = MockApi::start(vec![reply]).await;
    let options = ResponseOptions::default();
    render(
        "/?q=metta",
        Setup {
            source: real(&api),
            options: Some(options.clone()),
            ..Setup::default()
        },
    )
    .await;
    let parts = options.0.read().expect("the options lock").clone();
    let status = parts.status.map_or(0, |s| s.as_u16());
    (status, header(&parts, "retry-after").map(str::to_owned))
}

/// T1: from the API's header to the page's, end to end: a value passes
/// through, 0 and a huge one are clamped, an absent one is the cold default
/// (10) or none for a rate limit.
#[tokio::test]
async fn the_apis_retry_after_reaches_the_page_clamped() {
    let with = |reply: Reply, after: Option<&str>| match after {
        Some(value) => reply.with_header("retry-after", value),
        None => reply,
    };
    let cold = |after| with(Reply::error(503, "engine_starting", "waking"), after);
    let limited = |after| with(Reply::raw(429, "slow down"), after);
    let rows = [
        (cold(Some("4")), 503, Some("4")),
        (cold(Some("0")), 503, Some("1")),
        (cold(Some("99999")), 503, Some("300")),
        (cold(None), 503, Some("10")),
        (limited(Some("30")), 429, Some("30")),
        (limited(Some("99999")), 429, Some("300")),
        (limited(None), 429, None),
    ];
    for (index, (reply, status, want)) in rows.into_iter().enumerate() {
        let (got_status, got) = retry_after_for(reply).await;
        assert_eq!(got_status, status, "row {index}");
        assert_eq!(got.as_deref(), want, "row {index}");
    }
}

/// T1 (review 005, round 2): the English fallback, through the page. The API
/// has no French index (`404 lang_unavailable`), so the page — in French —
/// shows the English search's hit under the French fallback note, and the
/// API was asked twice: `fr`, then `en`.
#[tokio::test]
async fn an_unavailable_language_falls_back_to_english_on_the_page_and_says_so() {
    let api = MockApi::start(vec![
        Reply::error(404, "lang_unavailable", "no store for fr"),
        canary_hits(),
    ])
    .await;
    let html = render(
        "/?q=deuil",
        Setup {
            source: real(&api),
            language: Language::Fr,
            ..Setup::default()
        },
    )
    .await;
    assert!(
        html.contains("a canary snippet only the API answers"),
        "the English hit is in the first HTML: {html}"
    );
    assert!(
        html.contains("Recherche dans les textes anglais"),
        "the French fallback note: {html}"
    );
    assert_eq!(api.calls(), 2, "the language, then English");
    assert_eq!(api.call(0).query["lang"], "fr");
    assert_eq!(api.call(1).query["lang"], "en");
    assert_eq!(api.call(1).query["q"], "deuil");
}
