//! A scripted [`SearchBackend`] for the tests of the page: it answers what
//! the test says, keeps every query it was asked and has a `meaning()`
//! signal the test flips. It lives in a `SearchProvider` like any host's
//! backend, so a test drives the page through the real trait.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use leptos::prelude::*;

use super::backend::{SearchBackend, SearchFuture, SearchQuery};
use super::{HitDto, Mode, SearchOutcome, SearchResponse, SnippetKind, Sources, Titles};

/// What a fake answers for a query.
type Reply = dyn Fn(&SearchQuery) -> SearchFuture + Send + Sync;

/// A [`SearchBackend`] that answers by a function and records the queries.
/// Cloning shares the record and the `meaning` flag, so a test keeps a
/// handle after giving one to the provider. The flag is an `ArcRwSignal`,
/// which belongs to no reactive arena, so the fake is made outside the
/// render (each render has its own sandboxed arena) and `meaning()` derives
/// its `Signal` in the owner that asks.
#[derive(Clone)]
pub(crate) struct FakeBackend {
    /// Every query the page asked, in order.
    asked: Arc<Mutex<Vec<SearchQuery>>>,
    /// The answer for a query.
    reply: Arc<Reply>,
    /// Whether Meaning and Both can be served now.
    meaning: ArcRwSignal<bool>,
    /// How many times `meaning()` was called.
    meaning_calls: Arc<AtomicUsize>,
}

impl FakeBackend {
    /// A fake whose `meaning()` is `meaning` and which answers `reply`.
    pub(crate) fn new(
        meaning: bool,
        reply: impl Fn(&SearchQuery) -> SearchFuture + Send + Sync + 'static,
    ) -> Self {
        FakeBackend {
            asked: Arc::default(),
            reply: Arc::new(reply),
            meaning: ArcRwSignal::new(meaning),
            meaning_calls: Arc::default(),
        }
    }

    /// A fake that answers `outcome` at once, to every query.
    pub(crate) fn answering(meaning: bool, outcome: SearchOutcome) -> Self {
        FakeBackend::new(meaning, move |_| {
            let outcome = outcome.clone();
            Box::pin(async move { outcome })
        })
    }

    /// The queries asked so far.
    pub(crate) fn asked(&self) -> Vec<SearchQuery> {
        self.asked.lock().expect("the record lock").clone()
    }

    /// How many times `meaning()` was called.
    pub(crate) fn meaning_calls(&self) -> usize {
        self.meaning_calls.load(Ordering::SeqCst)
    }

    /// Turns `meaning()` on or off, as the app's model arriving would.
    pub(crate) fn set_meaning(&self, on: bool) {
        self.meaning.set(on);
    }
}

impl SearchBackend for FakeBackend {
    fn search(&self, query: SearchQuery) -> SearchFuture {
        let future = (self.reply)(&query);
        self.asked.lock().expect("the record lock").push(query);
        future
    }

    fn meaning(&self) -> Signal<bool> {
        self.meaning_calls.fetch_add(1, Ordering::SeqCst);
        let meaning = self.meaning.clone();
        Signal::derive(move || meaning.get())
    }
}

/// One hit, titled `title`, found by both branches.
pub(crate) fn hit(uid: &str, title: &str) -> HitDto {
    HitDto {
        sutta_uid: uid.to_owned(),
        display_id: uid.to_uppercase(),
        title: title.to_owned(),
        pali_title: None,
        titles: Titles::default(),
        snippet: format!("{title}, a snippet only the fake answers"),
        snippet_kind: SnippetKind::Passage,
        url: format!("https://suttacentral.net/{uid}"),
        sources: Sources {
            dense: true,
            sparse: true,
        },
        score: 0.9,
    }
}

/// An answer with `hits`, for `query`, as searched in `mode`.
pub(crate) fn hits(query: &str, mode: Mode, hits: Vec<HitDto>) -> SearchOutcome {
    SearchOutcome::Hits(SearchResponse {
        query: query.to_owned(),
        mode,
        hits,
        fell_back: false,
    })
}
