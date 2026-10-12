//! The page's render harness for tests: the page as the server renders it
//! under `SsrMode::Async` — its `Resource` awaited, the stream collected
//! (`to_html_async` does not exist in this Leptos; task 005, A5) — with the
//! search source and, when asked, the response options in the context, as
//! the router provides them.

use std::sync::Arc;

use futures::StreamExt;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::location::RequestUrl;
use leptos_router::path;
use palicanon_ui_lib::Language;

use super::SearchPage;
use crate::search::mock::MockSource;
use crate::search::source::Source;

/// What a render is given besides the URL: the search source, the app's
/// language, and the response options the server would hand the page
/// (`None` on the hydrate path, which has none).
pub(super) struct Setup {
    /// The source the server fn searches; the mock unless a test brings one.
    pub(super) source: Arc<Source>,
    /// The app's language signal's value.
    pub(super) language: Language,
    /// The response options, kept by the test to read what the page set.
    pub(super) options: Option<leptos_axum::ResponseOptions>,
}

impl Default for Setup {
    fn default() -> Self {
        Setup {
            source: Arc::new(Source::Mock(MockSource)),
            language: Language::En,
            options: None,
        }
    }
}

/// Renders [`SearchPage`] at `path` (its query string included) and
/// collects the whole in-order stream: what the browser receives.
pub(super) async fn render(path: &str, setup: Setup) -> String {
    render_view(|| {
        palicanon_ui_lib::provide_language().set(setup.language);
        provide_context(RequestUrl::new(path));
        provide_context(setup.source);
        if let Some(options) = setup.options {
            provide_context(options);
        }
        view! {
            <Router>
                <Routes fallback=|| "not found">
                    <Route path=path!("/") view=SearchPage />
                </Routes>
            </Router>
        }
    })
    .await
}

/// One render, or one walk of the app's route list, at a time in the test
/// binary: they run the page's reactive graph on several test threads at
/// once, and a `Resource` then never resolves (its task is never polled)
/// while the route walk is in progress on another thread.
static ONE_RENDER: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Takes the one-at-a-time lock; a poisoned lock is one test's failed
/// assertion, not a broken render.
pub(crate) fn exclusive() -> std::sync::MutexGuard<'static, ()> {
    ONE_RENDER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Renders the view `build` makes (in a fresh owner, where it may provide
/// its context) through the in-order stream and collects it; a stream that
/// stalls for 10 s fails the test with what it had rendered, instead of
/// hanging the run.
// The guard is held across the awaits on purpose: it is what keeps the
// renders of the test binary one at a time (each test has its own runtime
// thread, so no task of this thread waits for the lock).
#[allow(clippy::await_holding_lock)]
pub(crate) async fn render_view<V: RenderHtml>(build: impl FnOnce() -> V) -> String {
    let _one = exclusive();
    // A second initialisation is refused: the first one stands for the
    // whole test binary (each test's runtime is reached through it).
    let _ = any_spawner::Executor::init_tokio();
    // The `Resource`'s futures are spawned local: they need a `LocalSet`.
    tokio::task::LocalSet::new()
        .run_until(async {
            let owner = Owner::new();
            let stream = owner.with(|| build().to_html_stream_in_order());
            let mut stream = Box::pin(stream);
            let mut html = String::new();
            loop {
                match tokio::time::timeout(std::time::Duration::from_secs(10), stream.next()).await
                {
                    Ok(Some(chunk)) => html.push_str(&chunk),
                    Ok(None) => break,
                    Err(elapsed) => panic!("render stalled ({elapsed}) after: {html}"),
                }
            }
            drop(owner);
            html
        })
        .await
}

/// [`render`] with the mock source, in English.
pub(super) async fn render_at(path: &str) -> String {
    render(path, Setup::default()).await
}

/// [`render`] with the mock source and the app switched to `language`.
pub(super) async fn render_in(language: Language, path: &str) -> String {
    render(
        path,
        Setup {
            language,
            ..Setup::default()
        },
    )
    .await
}
