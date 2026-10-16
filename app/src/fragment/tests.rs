//! The fragment's tests: the router question first (what `use_matched()`
//! says in a relative route under a parent), then the fragment itself.

use leptos::prelude::*;
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes};
use leptos_router::hooks::use_matched;
use leptos_router::location::RequestUrl;
use leptos_router::path;

use crate::page::harness::render_view;

/// A page that only says what `use_matched()` is where it is mounted.
#[component]
fn Probe() -> impl IntoView {
    let matched = use_matched();
    view! { <p id="probe">{move || format!("matched=[{}]", matched.get())}</p> }
}

/// Renders the probe at `url`, as the app does: a relative empty route
/// inside a parent at `/search`.
async fn probe_under_search(url: &str) -> String {
    render_view(|| {
        provide_context(RequestUrl::new(url));
        view! {
            <Router>
                <Routes fallback=|| "not found">
                    <ParentRoute path=path!("/search") view=Outlet>
                        <Route path=path!("") view=Probe />
                    </ParentRoute>
                </Routes>
            </Router>
        }
    })
    .await
}

/// 021, the router question: `/search`, `/search/` and `/search?q=x` all
/// match the empty route under `/search`, and `use_matched()` is `/search`
/// (no trailing slash) in each, so a link built from it is `/search?q=…`.
#[tokio::test]
async fn an_empty_route_under_search_matches_and_use_matched_is_search() {
    for url in ["/search", "/search/", "/search?q=x", "/search/?q=x"] {
        let html = probe_under_search(url).await;
        assert!(!html.contains("not found"), "{url}: {html}");
        assert!(html.contains("matched=[/search]"), "{url}: {html}");
    }
}

/// The same empty route at the root: `use_matched()` is empty, so the
/// site's links stay `/?q=…`.
#[tokio::test]
async fn an_empty_route_at_the_root_has_an_empty_matched() {
    let html = render_view(|| {
        provide_context(RequestUrl::new("/?q=x"));
        view! {
            <Router>
                <Routes fallback=|| "not found">
                    <Route path=path!("") view=Probe />
                </Routes>
            </Router>
        }
    })
    .await;
    assert!(!html.contains("not found"), "{html}");
    assert!(html.contains("matched=[]"), "{html}");
}

/// The provider reads `meaning()` once, at mount, however many times the
/// page renders and whatever the signal does after.
#[tokio::test]
async fn the_provider_asks_for_the_meaning_signal_once() {
    use crate::fragment::SearchProvider;
    use crate::search::SearchOutcome;
    use crate::search::fake::FakeBackend;

    let fake = FakeBackend::answering(true, SearchOutcome::Failed);
    let backend = fake.clone();
    let html = render_view(move || {
        provide_context(RequestUrl::new("/"));
        view! {
            <Router>
                <SearchProvider backend=backend>
                    <Routes fallback=|| "not found">
                        <Route path=path!("") view=Probe />
                    </Routes>
                </SearchProvider>
            </Router>
        }
    })
    .await;
    assert!(html.contains("matched=[]"), "{html}");
    fake.set_meaning(false);
    assert_eq!(fake.meaning_calls(), 1);
}

// ---- the fragment under a host (task 021) ----

use leptos_router::components::Outlet as HostOutlet;

use crate::fragment::{SearchProvider as Provider, SearchRoutes};
use crate::search::fake::{FakeBackend, hit, hits};
use crate::search::{Mode, SearchOutcome};

/// Renders the fragment as the PaliCanon app mounts it: inside its own
/// `ParentRoute` at `/search`, a fake backend behind it, English.
async fn in_the_app(url: &str, backend: FakeBackend) -> String {
    render_view(move || {
        palicanon_ui_lib::provide_language().set(palicanon_ui_lib::Language::En);
        provide_context(RequestUrl::new(url));
        view! {
            <Router>
                <Provider backend=backend>
                    <Routes fallback=|| "not found">
                        <ParentRoute path=path!("/search") view=HostOutlet>
                            <SearchRoutes />
                        </ParentRoute>
                    </Routes>
                </Provider>
            </Router>
        }
    })
    .await
}

/// Renders the fragment as the site mounts it: at the root.
async fn at_the_root(url: &str, backend: FakeBackend) -> String {
    render_view(move || {
        palicanon_ui_lib::provide_language().set(palicanon_ui_lib::Language::En);
        provide_context(RequestUrl::new(url));
        view! {
            <Router>
                <Provider backend=backend>
                    <Routes fallback=|| "not found">
                        <SearchRoutes />
                    </Routes>
                </Provider>
            </Router>
        }
    })
    .await
}

/// A fake that answers one hit, meaning served.
fn one_hit() -> FakeBackend {
    FakeBackend::answering(
        true,
        hits("metta", Mode::Both, vec![hit("mn10", "Canary Sutta")]),
    )
}

/// `/search` and its variants are the home: the brand, the card, no
/// results, and nothing asked of the backend.
#[tokio::test]
async fn the_home_renders_at_search_and_when_the_query_is_blank() {
    for url in ["/search", "/search/", "/search?q=", "/search?q=%20"] {
        let fake = one_hit();
        let html = in_the_app(url, fake.clone()).await;
        assert!(!html.contains("not found"), "{url}: {html}");
        assert!(html.contains(">PaliCanon</h1>"), "{url}: {html}");
        assert!(html.contains(r#"role="search""#), "{url}: {html}");
        assert_eq!(fake.asked(), vec![], "{url}");
    }
}

/// A query is the results page: the backend's hit, asked once.
#[tokio::test]
async fn a_query_at_search_is_the_results_page() {
    let fake = one_hit();
    let html = in_the_app("/search?q=metta", fake.clone()).await;
    assert!(html.contains("Canary Sutta"), "{html}");
    assert_eq!(fake.asked().len(), 1, "{html}");
}

/// A zero-hit answer at `/search` is the no-result state, not an error and
/// not the home.
#[tokio::test]
async fn a_zero_hit_answer_at_search_is_the_no_result_state() {
    let fake = FakeBackend::answering(true, hits("zzz", Mode::Both, vec![]));
    let html = in_the_app("/search?q=zzz", fake.clone()).await;
    assert!(html.contains("No passage matches"), "{html}");
    assert!(!html.contains(r#"role="alert""#), "{html}");
    assert!(!html.contains(">PaliCanon</h1>"), "{html}");
    assert_eq!(fake.asked().len(), 1);
}

/// Every link the fragment builds is under `/search` in the app: the brand
/// of the results page, the retry of a cold engine, the examples and the
/// verse's refresh on the home.
#[tokio::test]
async fn the_links_carry_the_matched_base_in_the_app() {
    let cold = FakeBackend::answering(true, SearchOutcome::Cold { retry_after: None });
    let html = in_the_app("/search?q=metta&c=mn&mode=words", cold).await;
    assert!(html.contains(r#"<a href="/search" class"#), "brand: {html}");
    assert!(
        html.contains(r#"<a href="/search?q=metta&amp;c=mn&amp;mode=words""#),
        "retry: {html}"
    );
    let html = in_the_app("/search", one_hit()).await;
    assert_eq!(html.matches(r#"href="/search?q="#).count(), 3, "{html}");
    assert!(html.contains(r#"href="/search?passage="#), "{html}");
    assert!(
        !html.contains(r#"href="/?"#),
        "no link leaves /search: {html}"
    );
    assert!(
        !html.contains(r#"href="/""#),
        "no link leaves /search: {html}"
    );
}

/// The same fragment at the root keeps the site's links: `/`, `/?q=`.
#[tokio::test]
async fn the_links_stay_at_the_root_on_the_site() {
    let cold = FakeBackend::answering(true, SearchOutcome::Cold { retry_after: None });
    let html = at_the_root("/?q=metta", cold).await;
    assert!(html.contains(r#"<a href="/" class"#), "brand: {html}");
    assert!(html.contains(r#"<a href="/?q=metta""#), "{html}");
    let html = at_the_root("/", one_hit()).await;
    assert_eq!(html.matches(r#"href="/?q="#).count(), 3, "{html}");
    assert!(html.contains(r#"href="/?passage="#), "{html}");
    assert!(!html.contains("/search"), "{html}");
}

/// Outside its parent the fragment is not there: `/` and `/nope` are the
/// host's.
#[tokio::test]
async fn outside_search_the_host_answers() {
    for url in ["/", "/nope", "/searching"] {
        let html = in_the_app(url, one_hit()).await;
        assert!(html.contains("not found"), "{url}: {html}");
        assert!(!html.contains(r#"role="search""#), "{url}: {html}");
    }
}

/// The fragment follows the host's one language signal.
#[tokio::test]
async fn the_fragment_follows_the_host_s_language() {
    let html = render_view(|| {
        palicanon_ui_lib::provide_language().set(palicanon_ui_lib::Language::Fr);
        provide_context(RequestUrl::new("/search"));
        view! {
            <Router>
                <Provider backend=one_hit()>
                    <Routes fallback=|| "not found">
                        <ParentRoute path=path!("/search") view=HostOutlet>
                            <SearchRoutes />
                        </ParentRoute>
                    </Routes>
                </Provider>
            </Router>
        }
    })
    .await;
    assert!(html.contains("Chercher dans le Canon pāli"), "{html}");
}

/// Hidden dependencies, tested: the assets are root-served (`/lotus.png`,
/// `/brand/*.svg`), whatever the base; the page holds one search form; an
/// external link opens in a new tab (the host takes it outside the window).
#[tokio::test]
async fn the_assets_the_form_and_the_external_links_are_as_documented() {
    let html = in_the_app("/search", one_hit()).await;
    for asset in ["/lotus.png", "/brand/app-store.svg", "/brand/kofi.svg"] {
        assert!(
            html.contains(&format!(r#"src="{asset}""#)),
            "{asset}: {html}"
        );
    }
    assert!(
        html.contains(r#"href="https://suttacentral.net/"#),
        "{html}"
    );
    let html = in_the_app("/search?q=metta", one_hit()).await;
    assert_eq!(html.matches(r#"id="search-form""#).count(), 1, "{html}");
    let link = html
        .find(r#"href="https://suttacentral.net/mn10""#)
        .expect(&html);
    let tag_end = link + html[link..].find('>').expect(&html);
    assert!(html[link..tag_end].contains(r#"target="_blank""#), "{html}");
}

/// Without a provider the page says what it needs, not a bare unwrap.
#[tokio::test]
#[should_panic(expected = "SearchProvider")]
async fn a_page_without_a_provider_names_it() {
    let _ = render_view(|| {
        provide_context(RequestUrl::new("/"));
        view! {
            <Router>
                <Routes fallback=|| "not found">
                    <SearchRoutes />
                </Routes>
            </Router>
        }
    })
    .await;
}
