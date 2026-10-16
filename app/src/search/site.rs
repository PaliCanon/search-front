//! The site's [`SearchBackend`]: today's server function. On the server a
//! direct call (the source the router put in the context answers), in the
//! browser the HTTP call to `POST /api/search_passages`, so the WASM holds
//! neither the mock nor a token. Compiled with the `site` feature only
//! (`ssr` or `hydrate`): the PaliCanon app's `csr` build has no
//! `search_passages` and no endpoint string.

use leptos::prelude::*;

use super::SearchOutcome;
use super::backend::{SearchBackend, SearchFuture, SearchQuery};
use super::server::search_passages;

/// The site's backend: the server function. A server function that itself
/// fails (the browser lost the network, no source in the server's context)
/// is `Failed`, the page's generic error. It serves meaning always.
pub struct SiteBackend {
    /// `meaning()`: a constant true, made once.
    meaning: Signal<bool>,
}

impl SiteBackend {
    /// The site's backend. Call it inside the app's reactive owner (it
    /// makes the constant `meaning()` signal there).
    #[must_use]
    pub fn new() -> Self {
        SiteBackend {
            meaning: Signal::stored(true),
        }
    }
}

impl Default for SiteBackend {
    fn default() -> Self {
        SiteBackend::new()
    }
}

impl SearchBackend for SiteBackend {
    fn search(&self, query: SearchQuery) -> SearchFuture {
        Box::pin(async move {
            search_passages(
                query.q,
                query.lang.to_owned(),
                query.mode.code().to_owned(),
                query.collection.code().to_owned(),
            )
            .await
            .unwrap_or(SearchOutcome::Failed)
        })
    }

    fn meaning(&self) -> Signal<bool> {
        self.meaning
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use std::sync::Arc;

    use leptos::reactive::computed::ScopedFuture;

    use super::*;
    use crate::search::backend::{SearchBackend, SearchQuery};
    use crate::search::mock::{ERROR_QUERY, MockSource};
    use crate::search::source::Source;
    use crate::search::{Collection, Mode, SearchOutcome};

    fn query(q: &str, mode: Mode) -> SearchQuery {
        SearchQuery {
            q: q.to_owned(),
            lang: "en",
            mode,
            collection: Collection::All,
        }
    }

    /// The backend's answer for `query` in `owner`, polled inside it (the
    /// server function reads its source from the context at its first poll,
    /// as a `Resource` polls it).
    async fn answer(owner: &Owner, backend: &SiteBackend, query: SearchQuery) -> SearchOutcome {
        owner
            .with(|| ScopedFuture::new(backend.search(query)))
            .await
    }

    /// The site's backend is today's server function: on the server a
    /// direct call, so the source in the context answers.
    #[tokio::test]
    async fn the_site_backend_asks_the_source_in_the_context() {
        let owner = Owner::new();
        owner.with(|| provide_context(Arc::new(Source::Mock(MockSource))));
        let backend = owner.with(SiteBackend::new);
        let outcome = answer(&owner, &backend, query("mindfulness", Mode::Both)).await;
        let SearchOutcome::Hits(response) = outcome else {
            panic!("hits expected: {outcome:?}");
        };
        assert!(response.hits.iter().any(|h| h.display_id == "MN 10"));
    }

    /// The source's own non-hit states cross the trait unchanged.
    #[tokio::test]
    async fn the_site_backend_keeps_the_source_s_states() {
        let owner = Owner::new();
        owner.with(|| provide_context(Arc::new(Source::Mock(MockSource))));
        let backend = owner.with(SiteBackend::new);
        let outcome = answer(&owner, &backend, query(ERROR_QUERY, Mode::Both)).await;
        assert_eq!(outcome, SearchOutcome::Failed);
    }

    /// A server function that itself fails (no source in the context here;
    /// in the browser, the lost network) is `Failed`, never an error.
    #[tokio::test]
    async fn a_failing_server_function_is_failed() {
        let owner = Owner::new();
        let backend = owner.with(SiteBackend::new);
        let outcome = answer(&owner, &backend, query("metta", Mode::Both)).await;
        assert_eq!(outcome, SearchOutcome::Failed);
    }

    /// `Default` is `new`.
    #[test]
    fn the_default_site_backend_serves_meaning() {
        Owner::new().with(|| assert!(SiteBackend::default().meaning().get_untracked()));
    }

    /// The site always serves meaning, and `meaning()` hands out the one
    /// signal it made at construction.
    #[test]
    fn the_site_serves_meaning_always_through_one_signal() {
        Owner::new().with(|| {
            let backend = SiteBackend::new();
            let (first, second) = (backend.meaning(), backend.meaning());
            assert!(first.get_untracked());
            assert_eq!(first, second, "the same handle on every call");
        });
    }
}
