//! The Search fragment in a real browser (task 021): `SearchRoutes` mounted
//! inside `ParentRoute /search` of a host, a fake backend behind it, in a
//! headless Chrome. What the native tests cannot see: the first paint of the
//! home, the `/` shortcut, a submit that navigates with no document load,
//! the dropdowns, the listener that must go with the page, and a model
//! arriving while a search is pending (a live reactive graph).
//!
//! Run with `wasm-bindgen-test-runner` (the version of `Cargo.lock`'s
//! wasm-bindgen) and a chromedriver matching Chrome:
//! `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner
//! CHROMEDRIVER=<chromedriver> cargo test --locked -p search_front
//! --no-default-features --features csr --target wasm32-unknown-unknown
//! --test csr --test csr_mount`.
//!
//! Each test mounts its own host, after dropping the previous test's (its DOM
//! goes; dropping a mount handle does not run the fragment's cleanups, the
//! owner outlives it). The cleanups, the window listener among them, run
//! when the Search route is disposed: every test ends with `leave()`, which
//! navigates the host to its sibling route, and a test of the cleanups
//! switches one host's routes off with `shown`. A test that fails does not
//! reach its `leave()` (a wasm panic does not unwind) and can leave its
//! listener behind for the next, which then fails too: read the first
//! failure. The page's URL is global: each test sets it in `mount`.

#![cfg(all(target_arch = "wasm32", feature = "csr"))]

mod common;

use common::{
    Fake, count, find, leave, location, marker_survived, mount, mount_hosts, one_hit, plant_marker,
    press, sleep, text, until,
};
use leptos::prelude::*;
use search_front::search::{Collection, Mode, SearchOutcome};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

/// The first lines of the home's verses.
const VERSES: [&str; 4] = [
    "Defeat anger with kindness,",
    "For never is hatred",
    "Not to do any evil;",
    "Health is the ultimate blessing;",
];

/// The home renders at `/search`: the brand, the card, the verse of the day
/// from the browser's clock, three examples that stay under `/search`, and
/// nothing asked of the backend.
#[wasm_bindgen_test]
async fn the_home_renders_its_verse_at_search() {
    let fake = Fake::answering(true, one_hit("Canary Sutta"));
    mount("/search", fake.clone());
    until("the card", || find("#q").is_some()).await;
    assert!(
        VERSES.iter().any(|verse| text().contains(verse)),
        "{}",
        text()
    );
    assert!(text().contains("PaliCanon"), "{}", text());
    assert_eq!(count("a[href^='/search?q=']"), 3, "examples");
    assert_eq!(
        count("a[href='/?q=']") + count("a[href='/']"),
        0,
        "no link leaves /search"
    );
    assert_eq!(fake.asked(), vec![]);
    leave().await;
}

/// `/` focuses the field (the browser feature's shortcut: the same cfg
/// hydrate compiles) and prevents the slash from being typed.
#[wasm_bindgen_test]
async fn slash_focuses_the_field() {
    mount("/search", Fake::answering(true, one_hit("Canary Sutta")));
    until("the card", || find("#q").is_some()).await;
    let input = find("#q").expect("the field");
    input.blur().expect("blurred");
    assert_ne!(
        document().active_element().map(|e| e.id()),
        Some("q".to_owned())
    );
    assert!(press("/", false), "the slash is not typed");
    assert_eq!(
        document().active_element().map(|e| e.id()),
        Some("q".to_owned())
    );
    input.blur().expect("blurred");
    assert!(press("k", true), "Cmd-K works too");
    assert_eq!(
        document().active_element().map(|e| e.id()),
        Some("q".to_owned())
    );
    leave().await;
}

/// A query typed and submitted navigates to `/search?q=…` with no document
/// load (a window marker survives), the backend saw it, the hits are shown.
#[wasm_bindgen_test]
async fn a_submitted_query_navigates_in_place_and_shows_the_hits() {
    let fake = Fake::answering(true, one_hit("Canary Sutta"));
    mount("/search", fake.clone());
    until("the card", || find("#q").is_some()).await;
    plant_marker();
    let input = find("#q")
        .expect("the field")
        .unchecked_into::<web_sys::HtmlInputElement>();
    input.set_value(" metta ");
    input
        .form()
        .expect("in a form")
        .request_submit()
        .expect("submitted");
    until("the results", || text().contains("Canary Sutta")).await;
    assert_eq!(location(), "/search?q=metta");
    assert!(marker_survived(), "the document was not loaded again");
    let asked = fake.asked();
    assert_eq!(asked.len(), 1, "{asked:?}");
    assert_eq!(
        (asked[0].q.as_str(), asked[0].mode, asked[0].collection),
        ("metta", Mode::Both, Collection::All)
    );
    leave().await;
}

/// While meaning is off the mode dropdown is not there, so the submitted
/// form carries no mode: the mode the URL asked is kept (a shared
/// `?mode=meaning` link is not rewritten by the next query), though the
/// backend is asked Words.
#[wasm_bindgen_test]
async fn a_submit_without_the_mode_dropdown_keeps_the_asked_mode() {
    let fake = Fake::answering(false, one_hit("Canary Sutta"));
    mount("/search?q=metta&mode=meaning", fake.clone());
    until("the hits", || text().contains("Canary Sutta")).await;
    assert_eq!(
        count("button[aria-controls='mode-menu']"),
        0,
        "nothing to choose"
    );
    let input = find("#q")
        .expect("the field")
        .unchecked_into::<web_sys::HtmlInputElement>();
    input.set_value("karuna");
    input
        .form()
        .expect("in a form")
        .request_submit()
        .expect("submitted");
    until("the new query", || location().contains("q=karuna")).await;
    assert_eq!(location(), "/search?q=karuna&mode=meaning");
    until("its search", || fake.asked().len() == 2).await;
    assert_eq!(
        fake.asked()[1].mode,
        Mode::Words,
        "the backend is asked Words"
    );
    leave().await;
}

/// Submits the search form of the page with `value` typed in the field.
fn submit(value: &str) {
    let input = find("#q")
        .expect("the field")
        .unchecked_into::<web_sys::HtmlInputElement>();
    input.set_value(value);
    input
        .form()
        .expect("in a form")
        .request_submit()
        .expect("submitted");
}

/// A terminal state has a way out: after `Failed`, the same query submitted
/// again asks the backend again (the URL does not change, the search does),
/// and the hits it answers are shown.
#[wasm_bindgen_test]
async fn failed_then_the_same_query_again_asks_again() {
    let fake = Fake::sequence(true, vec![SearchOutcome::Failed, one_hit("Canary Sutta")]);
    mount("/search?q=metta", fake.clone());
    until("the error", || count("[role='alert']") == 1).await;
    assert_eq!(fake.asked().len(), 1);
    submit("metta");
    until("the hits", || text().contains("Canary Sutta")).await;
    assert_eq!(fake.asked().len(), 2, "asked again");
    assert_eq!(count("[role='alert']"), 0);
    leave().await;
}

/// Cold and rate limited: following the Retry link asks again, though it
/// points at the URL the page is already at.
#[wasm_bindgen_test]
async fn cold_and_rate_limited_retry_asks_again() {
    for first in [
        SearchOutcome::Cold { retry_after: None },
        SearchOutcome::RateLimited { retry_after: None },
    ] {
        let fake = Fake::sequence(true, vec![first, one_hit("Canary Sutta")]);
        mount("/search?q=metta", fake.clone());
        until("the alert", || count("[role='alert'] a") == 1).await;
        find("[role='alert'] a").expect("the retry link").click();
        until("the hits", || text().contains("Canary Sutta")).await;
        assert_eq!(fake.asked().len(), 2, "asked again");
        leave().await;
    }
}

/// G2: a submit keeps every field the form lacks from the URL, `?passage=`
/// included, and writes no field that equals its default (no `mode=both`
/// that the URL did not have).
#[wasm_bindgen_test]
async fn a_submit_keeps_the_passage_and_writes_no_default() {
    let fake = Fake::answering(true, one_hit("Canary Sutta"));
    mount("/search?passage=2", fake.clone());
    until("the card", || find("#q").is_some()).await;
    submit("metta");
    until("the hits", || text().contains("Canary Sutta")).await;
    assert_eq!(location(), "/search?q=metta&passage=2");
    leave().await;
}

/// Picks the item of the dropdown controlling `menu` whose label is `label`.
async fn choose(menu: &str, label: &str) {
    let button = format!("button[aria-controls='{menu}']");
    until("the dropdown", || find(&button).is_some()).await;
    find(&button).expect("the dropdown").click();
    until("its menu", || count("[role='menuitemradio']") > 0).await;
    let items = document()
        .query_selector_all("[role='menuitemradio']")
        .expect("a valid selector");
    let item = (0..items.length())
        .filter_map(|i| items.item(i))
        .map(wasm_bindgen::JsCast::unchecked_into::<web_sys::HtmlElement>)
        .find(|item| item.text_content().is_some_and(|t| t.trim() == label))
        .unwrap_or_else(|| panic!("no menu item {label}: {}", text()));
    item.click();
}

/// A dropdown choice on the results page re-runs the search the same way:
/// in place, the choice in the URL, the backend asked with it.
#[wasm_bindgen_test]
async fn a_dropdown_choice_navigates_in_place_too() {
    let fake = Fake::answering(true, one_hit("Canary Sutta"));
    mount("/search?q=metta", fake.clone());
    until("the hits", || text().contains("Canary Sutta")).await;
    plant_marker();
    choose("collection-menu", "MN — Majjhima Nikāya").await;
    until("the collection in the url", || location().contains("c=mn")).await;
    assert_eq!(location(), "/search?q=metta&c=mn");
    until("the second search", || fake.asked().len() == 2).await;
    choose("mode-menu", "Words").await;
    until("the mode in the url", || location().contains("mode=words")).await;
    assert_eq!(location(), "/search?q=metta&c=mn&mode=words");
    until("the third search", || fake.asked().len() == 3).await;
    let asked = fake.asked();
    let modes: Vec<_> = asked.iter().map(|q| (q.collection, q.mode)).collect();
    assert_eq!(
        modes,
        [
            (Collection::All, Mode::Both),
            (Collection::Mn, Mode::Both),
            (Collection::Mn, Mode::Words),
        ]
    );
    assert!(marker_survived(), "the document was not loaded again");
    leave().await;
}

/// Search left (navigated to a sibling route): `/` and Cmd-K are not
/// prevented any more, the listener went with the page, nothing panics.
#[wasm_bindgen_test]
async fn once_unmounted_the_shortcut_is_gone() {
    mount("/search", Fake::answering(true, one_hit("Canary Sutta")));
    until("the card", || find("#q").is_some()).await;
    assert!(press("/", false), "mounted: the shortcut works");
    leave().await;
    assert!(find("#q").is_none(), "the page is gone");
    assert!(!press("/", false), "unmounted: the slash is left alone");
    assert!(!press("k", true), "unmounted: Cmd-K is left alone");
    assert!(find("#not-found").is_none());
}

/// A model arriving while the Words search is pending: the dropdown
/// appears at once and the URL still says what was asked; the search in
/// flight is not cancelled (Leptos runs a `Resource`'s fetches one after
/// the other), so the same search is asked again as Both when it lands, and
/// the Both answer is the one that stays: a superseded answer can never
/// overwrite a fresher one.
#[wasm_bindgen_test]
async fn meaning_arriving_asks_the_same_search_again_as_asked() {
    let fake = Fake::gated(false);
    mount("/search?q=metta&mode=both", fake.clone());
    until("the first search", || fake.asked().len() == 1).await;
    assert_eq!(fake.asked()[0].mode, Mode::Words, "meaning is off");
    assert_eq!(count("button[aria-controls='collection-menu']"), 1);
    assert_eq!(
        count("button[aria-controls='mode-menu']"),
        0,
        "nothing to choose"
    );
    fake.set_meaning(true);
    until("the dropdown", || {
        count("button[aria-controls='mode-menu']") == 1
    })
    .await;
    assert_eq!(
        fake.asked().len(),
        1,
        "the search in flight is not cancelled"
    );
    assert!(fake.release(0, one_hit("Words Sutta")));
    until("the second search", || fake.asked().len() == 2).await;
    assert_eq!(fake.asked()[1].mode, Mode::Both, "asked as asked");
    assert!(fake.release(1, one_hit("Both Sutta")));
    until("the Both answer", || text().contains("Both Sutta")).await;
    assert!(!text().contains("Words Sutta"), "{}", text());
    assert_eq!(
        location(),
        "/search?q=metta&mode=both",
        "the url is never rewritten"
    );
    leave().await;
}

/// Two hosts mounted at once, one of them left (its routes switched off, its
/// cleanups run): one `/` press focuses the field of the other, once, and
/// nothing panics. A listener left behind by the first would act on its
/// disposed field and panic.
#[wasm_bindgen_test]
async fn one_host_left_and_the_slash_focuses_the_other_once() {
    let (left, stays) = (ArcRwSignal::new(true), ArcRwSignal::new(true));
    let fake = Fake::answering(true, one_hit("Canary Sutta"));
    mount_hosts(
        "/search",
        vec![(fake.clone(), left.clone()), (fake, stays.clone())],
    );
    until("both fields", || count("#q") == 2).await;
    left.set(false);
    until("one field", || count("#q") == 1).await;
    let focused = std::rc::Rc::new(std::cell::Cell::new(0_u32));
    let seen = std::rc::Rc::clone(&focused);
    let counter = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
        seen.set(seen.get() + 1);
    });
    document()
        .add_event_listener_with_callback("focusin", counter.as_ref().unchecked_ref())
        .expect("a listener");
    find("#q").expect("the field").blur().expect("blurred");
    assert!(press("/", false), "the remaining host answers");
    assert_eq!(focused.get(), 1, "focused once");
    assert_eq!(
        document().active_element().map(|e| e.id()),
        Some("q".to_owned())
    );
    document()
        .remove_event_listener_with_callback("focusin", counter.as_ref().unchecked_ref())
        .expect("removed");
    stays.set(false);
    until("no field", || count("#q") == 0).await;
    assert!(!press("/", false), "both left: the slash is left alone");
}

/// A blank or whitespace query never reaches the backend, from the home and
/// from a results page: the page is the home at `/search`, nothing is asked.
#[wasm_bindgen_test]
async fn a_blank_submit_never_reaches_the_backend() {
    let fake = Fake::answering(true, one_hit("Canary Sutta"));
    mount("/search", fake.clone());
    until("the card", || find("#q").is_some()).await;
    submit("   ");
    sleep(200).await;
    assert_eq!(fake.asked(), vec![], "the home asks nothing");
    assert_eq!(location(), "/search");
    submit("metta");
    until("the hits", || text().contains("Canary Sutta")).await;
    assert_eq!(fake.asked().len(), 1);
    submit("  ");
    until("the home", || location() == "/search").await;
    sleep(200).await;
    assert_eq!(fake.asked().len(), 1, "the blank query asked nothing");
    assert!(!text().contains("Canary Sutta"), "{}", text());
    leave().await;
}

/// Meaning turning on after the first paint and off again: the mode dropdown
/// follows, the URL keeps the mode that was asked, and the backend is asked
/// for the mode it can serve each time.
#[wasm_bindgen_test]
async fn meaning_on_then_off_the_dropdown_follows_and_the_asked_mode_stays() {
    let fake = Fake::answering(false, one_hit("Canary Sutta"));
    mount("/search?q=metta&mode=meaning", fake.clone());
    until("the hits", || text().contains("Canary Sutta")).await;
    let dropdown = || count("button[aria-controls='mode-menu']");
    let label = || {
        find("button[aria-controls='mode-menu']")
            .and_then(|button| button.text_content())
            .unwrap_or_default()
    };
    assert_eq!(dropdown(), 0);
    for (round, on) in [(2, true), (3, false), (4, true)] {
        fake.set_meaning(on);
        until("the dropdown", || (dropdown() == 1) == on).await;
        until("the search", || fake.asked().len() == round).await;
        assert_eq!(location(), "/search?q=metta&mode=meaning", "round {round}");
        if on {
            assert!(label().contains("Meaning"), "the asked mode: {}", label());
        }
    }
    let modes: Vec<_> = fake.asked().iter().map(|q| q.mode).collect();
    assert_eq!(
        modes,
        [Mode::Words, Mode::Meaning, Mode::Words, Mode::Meaning]
    );
    leave().await;
}
