//! What answers a search for the page: a [`SearchBackend`] a host gives to
//! the fragment's `SearchProvider` (task 021). The site plugs today's
//! server function ([`site::SiteBackend`](super::site::SiteBackend)); the
//! PaliCanon app plugs its local engine, never the network. The page asks
//! it for a [`SearchQuery`] and shows the [`SearchOutcome`]; it never learns
//! which side answered. This module holds no endpoint, no client and no
//! token, so it builds in every mode, `csr` alone included.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use leptos::prelude::Signal;

use super::{Collection, Mode, SearchOutcome};

/// What one search asks: the URL's `?q=&c=` and `?mode=` (resolved against
/// what the backend can serve now) and the app's language. The page's
/// `Resource` is keyed on it, so a change of any field asks again.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchQuery {
    /// The query, trimmed and never blank: a blank search is the home and
    /// asks nothing.
    pub q: String,
    /// The app's language code: `en`, `fr` or `vi`.
    pub lang: &'static str,
    /// The mode to search in: the one asked, or `Words` while the
    /// backend's [`meaning`](SearchBackend::meaning) is false.
    pub mode: Mode,
    /// The collection to narrow to.
    pub collection: Collection,
}

/// The answer of one search, boxed so a backend is a trait object and its
/// future is `Send` (the page's `Resource` requires it, on the server and
/// in the browser alike): a backend whose own future is not `Send` wraps
/// it (`send_wrapper`) inside its implementation.
pub type SearchFuture = Pin<Box<dyn Future<Output = SearchOutcome> + Send + 'static>>;

/// What a host gives to `SearchProvider` to answer the page's searches.
///
/// An implementation never fails as a function: a search that could not
/// answer is one of the [`SearchOutcome`] states (`Failed` for any failure
/// of its own, `Cold` while its engine is starting), each mapped by the
/// implementation, so no error type crosses this trait.
pub trait SearchBackend: Send + Sync + 'static {
    /// Every hit for `query`, best first, or the reason there are none to
    /// show. Never asked for a blank query, nor for [`Mode::Meaning`] or
    /// [`Mode::Both`] while [`meaning`](SearchBackend::meaning) is false
    /// (a race that does ask searches words and shows the hits, never an
    /// error).
    fn search(&self, query: SearchQuery) -> SearchFuture;

    /// Whether [`Mode::Meaning`] and [`Mode::Both`] can be served now
    /// (`Words` always can). The implementation creates the signal once, at
    /// construction, and returns the same handle on every call; the
    /// provider reads it once at mount. It may turn true later (a model
    /// arriving): the page then asks the same search again as asked.
    fn meaning(&self) -> Signal<bool>;
}

/// What `SearchProvider` puts in the context for the page: the backend and
/// its `meaning()` signal, read once at mount.
#[derive(Clone)]
pub(crate) struct Provided {
    /// The host's backend.
    pub(crate) backend: Arc<dyn SearchBackend>,
    /// The signal the backend's `meaning()` gave, read once at mount. The page
    /// tracks it, so a later change follows: the mode dropdown comes and goes
    /// and the search is asked again for the mode it can serve.
    pub(crate) meaning: Signal<bool>,
}

impl Provided {
    /// The context value for `backend`; reads its `meaning()` now, once.
    pub(crate) fn new(backend: impl SearchBackend) -> Self {
        let meaning = backend.meaning();
        Provided {
            backend: Arc::new(backend),
            meaning,
        }
    }
}
