//! The page through the [`SearchBackend`] trait (task 021): a fake backend
//! in a `SearchProvider` drives every outcome, with no server function, no
//! source and no network, and the queries it was asked are read back.

use super::harness::render_with;
use crate::search::backend::SearchQuery;
use crate::search::fake::{FakeBackend, hit, hits};
use crate::search::{Collection, Mode, SearchOutcome};

/// The query the page asks for `?q=metta&c=mn&mode=words` in English.
fn metta_words() -> SearchQuery {
    SearchQuery {
        q: "metta".to_owned(),
        lang: "en",
        mode: Mode::Words,
        collection: Collection::Mn,
    }
}

/// Hits: the fake's hit is in the first HTML, and the backend was asked
/// once, with the trimmed query, the app's language, the mode and the
/// collection of the URL.
#[tokio::test]
async fn the_hits_of_the_backend_are_shown_and_it_is_asked_once() {
    let fake = FakeBackend::answering(
        true,
        hits("metta", Mode::Words, vec![hit("mn10", "Canary Sutta")]),
    );
    let html = render_with("/?q=%20metta%20&c=mn&mode=words", fake.clone()).await;
    assert!(html.contains("Canary Sutta"), "{html}");
    assert!(html.contains("a snippet only the fake answers"), "{html}");
    assert_eq!(fake.asked(), vec![metta_words()], "{html}");
}

/// The home asks nothing: a blank `?q=` never reaches the backend.
#[tokio::test]
async fn a_blank_query_never_reaches_the_backend() {
    let fake = FakeBackend::answering(true, SearchOutcome::Failed);
    for url in ["/", "/?q=", "/?q=%20%20"] {
        let html = render_with(url, fake.clone()).await;
        assert!(html.contains(r#"role="search""#), "{url}: {html}");
    }
    assert_eq!(fake.asked(), vec![]);
}

/// Cold and rate limited: the alert, and a retry link that is the same GET.
#[tokio::test]
async fn cold_and_rate_limited_are_alerts_with_the_same_get() {
    for outcome in [
        SearchOutcome::Cold { retry_after: None },
        SearchOutcome::RateLimited {
            retry_after: Some(3),
        },
    ] {
        let fake = FakeBackend::answering(true, outcome.clone());
        let html = render_with("/?q=metta&c=mn&mode=words", fake).await;
        assert!(html.contains(r#"role="alert""#), "{outcome:?}: {html}");
        assert!(
            html.contains(r#"<a href="/?q=metta&amp;c=mn&amp;mode=words""#),
            "{outcome:?}: {html}"
        );
    }
}

/// Failed: the generic error, no link.
#[tokio::test]
async fn failed_is_the_generic_error() {
    let fake = FakeBackend::answering(true, SearchOutcome::Failed);
    let html = render_with("/?q=metta", fake).await;
    assert!(html.contains("Something went wrong"), "{html}");
    assert!(!html.contains("Search again"), "{html}");
}

/// No hit is "no result", not an error.
#[tokio::test]
async fn an_answer_with_no_hit_is_the_no_result_state() {
    let fake = FakeBackend::answering(true, hits("zzz", Mode::Both, vec![]));
    let html = render_with("/?q=zzz", fake).await;
    assert!(html.contains("No passage matches"), "{html}");
}
