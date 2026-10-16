//! What the two browser test files share (task 021): a scripted backend, a
//! host with a sibling route, and the helpers that read and drive the page.
//! The fragment is mounted the way the PaliCanon app mounts it: inside its
//! own `ParentRoute` at `/search`, next to a route of the host's.

#![allow(dead_code)] // each test file uses part of it

use std::any::Any;
use std::sync::{Arc, Mutex};

use futures::channel::oneshot::{self, Sender};
use leptos::prelude::*;
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes};
use leptos_router::path;
use search_front::fragment::{SearchProvider, SearchRoutes};
use search_front::search::backend::{SearchBackend, SearchFuture, SearchQuery};
use search_front::search::{
    HitDto, Mode, SearchOutcome, SearchResponse, SnippetKind, Sources, Titles,
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlElement;

/// The senders of the searches a gated fake holds, in the order asked.
type Held = Arc<Mutex<Vec<Option<Sender<SearchOutcome>>>>>;

/// A [`SearchBackend`] the test scripts: it answers at once or holds every
/// search until [`release`](Fake::release), records what it was asked, and
/// has a `meaning()` the test flips. Clones share all of it.
#[derive(Clone)]
pub struct Fake {
    asked: Arc<Mutex<Vec<SearchQuery>>>,
    held: Held,
    /// The answers, one per query in the order asked (the last repeats);
    /// empty: every query is held until [`Fake::release`].
    script: Vec<SearchOutcome>,
    meaning: ArcRwSignal<bool>,
}

impl Fake {
    /// Answers `outcome` at once, to every query.
    pub fn answering(meaning: bool, outcome: SearchOutcome) -> Self {
        Fake {
            asked: Arc::default(),
            held: Arc::default(),
            script: vec![outcome],
            meaning: ArcRwSignal::new(meaning),
        }
    }

    /// Answers the queries `outcomes` in the order asked, the last one
    /// repeating.
    pub fn sequence(meaning: bool, outcomes: Vec<SearchOutcome>) -> Self {
        Fake {
            script: outcomes,
            ..Fake::answering(meaning, SearchOutcome::Failed)
        }
    }

    /// Holds every query until the test answers it.
    pub fn gated(meaning: bool) -> Self {
        Fake {
            script: Vec::new(),
            ..Fake::answering(meaning, SearchOutcome::Failed)
        }
    }

    /// The queries asked so far.
    pub fn asked(&self) -> Vec<SearchQuery> {
        self.asked.lock().expect("the record").clone()
    }

    /// Answers the `index`-th held query; `false` when nobody waits for it
    /// any more (the page dropped it as superseded).
    pub fn release(&self, index: usize, outcome: SearchOutcome) -> bool {
        let sender = self
            .held
            .lock()
            .expect("the held")
            .get_mut(index)
            .and_then(Option::take);
        sender.is_some_and(|tx| tx.send(outcome).is_ok())
    }

    /// Turns `meaning()` on or off, as the app's model arriving would.
    pub fn set_meaning(&self, on: bool) {
        self.meaning.set(on);
    }
}

impl SearchBackend for Fake {
    fn search(&self, query: SearchQuery) -> SearchFuture {
        let n = {
            let mut asked = self.asked.lock().expect("the record");
            asked.push(query);
            asked.len() - 1
        };
        if let Some(outcome) = self.script.get(n).or(self.script.last()).cloned() {
            return Box::pin(async move { outcome });
        }
        let (tx, rx) = oneshot::channel();
        self.held.lock().expect("the held").push(Some(tx));
        Box::pin(async move { rx.await.unwrap_or(SearchOutcome::Failed) })
    }

    fn meaning(&self) -> Signal<bool> {
        let meaning = self.meaning.clone();
        Signal::derive(move || meaning.get())
    }
}

/// An answer with one hit titled `title`.
pub fn one_hit(title: &str) -> SearchOutcome {
    SearchOutcome::Hits(SearchResponse {
        query: "metta".to_owned(),
        mode: Mode::Both,
        hits: vec![HitDto {
            sutta_uid: "mn10".to_owned(),
            display_id: "MN 10".to_owned(),
            title: title.to_owned(),
            pali_title: None,
            titles: Titles::default(),
            snippet: "a snippet only the fake answers".to_owned(),
            snippet_kind: SnippetKind::Passage,
            url: "https://suttacentral.net/mn10".to_owned(),
            sources: Sources {
                dense: true,
                sparse: true,
            },
            score: 0.9,
        }],
        fell_back: false,
    })
}

/// The host: the fragment inside `ParentRoute /search` beside a sibling
/// route. `shown` switches the routes off and on (its owner is disposed, so
/// the fragment's cleanups run) without touching the URL: two hosts can then
/// be mounted at once and one of them left.
#[component]
pub fn Host(backend: Fake, shown: ArcRwSignal<bool>) -> impl IntoView {
    palicanon_ui_lib::provide_language();
    view! {
        <Router>
            <SearchProvider backend=backend>
                {move || {
                    shown
                        .get()
                        .then(|| {
                            view! {
                                <Routes fallback=|| view! { <p id="not-found">"not found"</p> }>
                                    <ParentRoute path=path!("/search") view=Outlet>
                                        <SearchRoutes />
                                    </ParentRoute>
                                    <Route
                                        path=path!("/other")
                                        view=|| view! { <p id="other">"other"</p> }
                                    />
                                </Routes>
                            }
                        })
                }}
            </SearchProvider>
        </Router>
    }
}

/// A mounted host; dropping it takes its DOM out and removes its element. It
/// does not run the fragment's cleanups (the owner outlives the handle):
/// those run when the Search route is disposed, see [`leave`] and [`Host`]'s
/// `shown`.
struct Mounted {
    handle: Option<Box<dyn Any>>,
    root: web_sys::Element,
}

impl Drop for Mounted {
    fn drop(&mut self) {
        drop(self.handle.take());
        self.root.remove();
    }
}

/// Puts the page at `url` (same origin) and the stored language at English.
pub fn visit(url: &str) {
    window()
        .history()
        .expect("history")
        .replace_state_with_url(&JsValue::NULL, "", Some(url))
        .expect("the url is same-origin");
    if let Ok(Some(storage)) = window().local_storage() {
        storage.set_item("pc_lang", "en").expect("storage");
    }
}

thread_local! {
    /// The hosts mounted last. The next [`mount`] drops them first (their DOM
    /// goes; their window listeners go only with [`leave`]).
    static LIVE: std::cell::RefCell<Vec<Mounted>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Navigates the host's router to `url` as the browser's back button would
/// (`pushState`, then `popstate`): the router follows the location. A click
/// on a link would also reach the routers of the earlier tests, which keep
/// their window listeners (see [`leave`]) and would swallow it.
pub fn go(url: &str) {
    window()
        .history()
        .expect("history")
        .push_state_with_url(&JsValue::NULL, "", Some(url))
        .expect("the url is same-origin");
    let event = web_sys::Event::new("popstate").expect("an event");
    window().dispatch_event(&event).expect("dispatched");
}

/// Ends a test: navigates to the host's sibling route, which disposes the
/// Search page (its cleanups run: the window listener is removed). Dropping
/// the mounted tree does not run them, with this Leptos (the owner outlives
/// its handle), so a test that did not leave would leave its `/` shortcut
/// behind for the next one.
pub async fn leave() {
    go("/other");
    until("the sibling route", || find("#other").is_some()).await;
}

/// Mounts [`Host`] at `url` in a fresh element of `<body>`, with what
/// `csr::mount` provides (the local store), after dropping the previous
/// test's host (`csr::mount` leaks its tree and its window listeners:
/// `tests/csr_mount.rs` has the page to itself).
pub fn mount(url: &str, backend: Fake) {
    mount_hosts(url, vec![(backend, ArcRwSignal::new(true))]);
}

/// Mounts one [`Host`] per entry, each in its own element and with its own
/// router, all at `url`; `shown` of each switches its routes on and off.
pub fn mount_hosts(url: &str, hosts: Vec<(Fake, ArcRwSignal<bool>)>) {
    LIVE.with_borrow_mut(Vec::clear);
    visit(url);
    for (backend, shown) in hosts {
        let root = document().create_element("div").expect("a div");
        root.set_attribute("data-test-root", "")
            .expect("an attribute");
        document()
            .body()
            .expect("a body")
            .append_child(&root)
            .expect("appended");
        let handle =
            leptos::mount::mount_to(root.clone().unchecked_into::<HtmlElement>(), move || {
                provide_context(palicanon_ui_lib::Store::Local);
                view! { <Host backend=backend shown=shown /> }
            });
        let mounted = Mounted {
            handle: Some(Box::new(handle)),
            root,
        };
        LIVE.with_borrow_mut(|live| live.push(mounted));
    }
}

/// The first element matching `selector`, if any.
pub fn find(selector: &str) -> Option<HtmlElement> {
    document()
        .query_selector(selector)
        .expect("a valid selector")
        .map(JsCast::unchecked_into)
}

/// How many elements match `selector`.
pub fn count(selector: &str) -> u32 {
    document()
        .query_selector_all(selector)
        .expect("a valid selector")
        .length()
}

/// The page's text.
pub fn text() -> String {
    document()
        .body()
        .and_then(|body| body.text_content())
        .unwrap_or_default()
}

/// `location.pathname + location.search`.
pub fn location() -> String {
    let location = window().location();
    format!(
        "{}{}",
        location.pathname().unwrap_or_default(),
        location.search().unwrap_or_default()
    )
}

/// Waits `ms` milliseconds.
pub async fn sleep(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        window()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
            .expect("a timer");
    });
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("the timer fires");
}

/// Waits until `check` holds (3 s at most), else fails with `what` and the
/// page's text.
pub async fn until(what: &str, check: impl Fn() -> bool) {
    for _ in 0..150 {
        if check() {
            return;
        }
        sleep(20).await;
    }
    let html = document()
        .query_selector("[data-test-root]")
        .ok()
        .flatten()
        .map(|root| root.inner_html())
        .unwrap_or_default();
    panic!(
        "timed out waiting for {what}; at {}; the app: {html}",
        location()
    );
}

/// Dispatches a `keydown` of `key` on the window (`meta`: ⌘ held) and says
/// whether the page prevented its default.
pub fn press(key: &str, meta: bool) -> bool {
    let init = web_sys::KeyboardEventInit::new();
    init.set_key(key);
    init.set_meta_key(meta);
    init.set_bubbles(true);
    init.set_cancelable(true);
    let event = web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("a keyboard event");
    window().dispatch_event(&event).expect("dispatched");
    event.default_prevented()
}

/// Sets a marker on `window` that only the same document keeps: a document
/// load would lose it.
pub fn plant_marker() {
    js_sys::Reflect::set(&window(), &"__search_front_marker".into(), &JsValue::TRUE)
        .expect("a marker");
}

/// Whether the marker survived.
pub fn marker_survived() -> bool {
    js_sys::Reflect::get(&window(), &"__search_front_marker".into())
        .is_ok_and(|value| value.is_truthy())
}
