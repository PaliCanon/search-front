//! The modes against `meaning()` (task 021): the URL keeps the mode that
//! was asked, the backend is asked the mode it can serve, the dropdown
//! shows only when there is something to choose. A model arriving while a
//! search is pending (the re-ask, the superseded answer dropped) needs a live
//! reactive graph, which the server build does not run (its render effects
//! are no-ops): the browser test, `tests/csr.rs`, watches it.

use leptos_router::params::ParamsMap;
use palicanon_ui_lib::Language;

use super::SearchKey;
use super::harness::render_with;
use crate::search::backend::SearchQuery;
use crate::search::fake::{FakeBackend, hits};
use crate::search::{Collection, Mode, SearchOutcome};

/// The query the page asks for `?q=metta` in `mode`.
fn metta(mode: Mode) -> SearchQuery {
    SearchQuery {
        q: "metta".to_owned(),
        lang: "en",
        mode,
        collection: Collection::All,
    }
}

/// Meaning off: the backend is asked Words whatever `?mode=` says (both is
/// the default, meaning is explicit), and a link to the same search still
/// says the mode that was asked.
#[tokio::test]
async fn without_meaning_the_backend_is_asked_words_and_the_links_keep_the_asked_mode() {
    for asked in ["both", "meaning", "words"] {
        let fake = FakeBackend::answering(false, SearchOutcome::Cold { retry_after: None });
        let html = render_with(&format!("/?q=metta&mode={asked}"), fake.clone()).await;
        assert_eq!(fake.asked(), vec![metta(Mode::Words)], "{asked}: {html}");
        // Both is the default: not written; the others are, as asked.
        let retry = if asked == "both" {
            r#"<a href="/?q=metta""#.to_owned()
        } else {
            format!(r#"<a href="/?q=metta&amp;mode={asked}""#)
        };
        assert!(html.contains(&retry), "{asked}: {html}");
    }
}

/// Meaning off: nothing to choose, so no mode dropdown, on the results
/// page and on the home; the collection dropdown stays.
#[tokio::test]
async fn without_meaning_there_is_no_mode_dropdown() {
    for url in ["/?q=metta", "/"] {
        let fake = FakeBackend::answering(false, hits("metta", Mode::Words, vec![]));
        let html = render_with(url, fake).await;
        assert!(!html.contains(r#"name="mode""#), "{url}: {html}");
        assert!(!html.contains("Words + meaning"), "{url}: {html}");
        assert!(html.contains(r#"name="c""#), "{url}: {html}");
    }
}

/// Meaning on: the mode asked is the mode the backend is asked, and the
/// dropdown is there with it selected.
#[tokio::test]
async fn with_meaning_the_asked_mode_is_asked_and_the_dropdown_is_shown() {
    for (asked, mode) in [
        ("both", Mode::Both),
        ("meaning", Mode::Meaning),
        ("words", Mode::Words),
    ] {
        let fake = FakeBackend::answering(true, hits("metta", mode, vec![]));
        let html = render_with(&format!("/?q=metta&mode={asked}"), fake.clone()).await;
        assert_eq!(fake.asked(), vec![metta(mode)], "{asked}: {html}");
        let selected = format!(r#"<option value="{asked}" selected"#);
        assert!(html.contains(&selected), "{asked}: {html}");
    }
    let fake = FakeBackend::answering(true, hits("", Mode::Both, vec![]));
    let html = render_with("/", fake).await;
    assert!(html.contains(r#"name="mode""#), "{html}");
}

/// The key of `?q=metta&mode=both` in English: Both is asked.
fn key_of_metta_both() -> SearchKey {
    let mut params = ParamsMap::new();
    params.insert("q", "metta".to_owned());
    params.insert("mode", "both".to_owned());
    SearchKey::read(&params, Language::En)
}

/// The key keeps the mode asked; only the backend's query is resolved.
#[test]
fn the_key_keeps_the_asked_mode_and_the_query_resolves_it() {
    let key = key_of_metta_both();
    assert_eq!(key.mode, Mode::Both);
    assert_eq!(key.query(true), metta(Mode::Both));
    assert_eq!(key.query(false), metta(Mode::Words));
    assert_eq!(key.mode, Mode::Both, "resolving does not rewrite the key");
}
