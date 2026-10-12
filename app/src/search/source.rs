//! The `Source` enum: the mock or the real source behind the one trait (slice 4 of task 005).
//!
//! Built once at boot from the environment ([`crate::boot::Config`]) and
//! shared by the server for every search; the page's code asks it through
//! [`SearchSource`] and never learns which one answers. `reqwest`'s client
//! is not `Clone` through `palicanon-api-client`, so the source is shared
//! behind an `Arc` (task 005 slice 7), not cloned.

use super::mock::MockSource;
use super::real::RealSource;
use super::{Collection, Mode, SearchOutcome, SearchSource};

/// The source the service searches with.
#[derive(Debug)]
pub enum Source {
    /// The in-repo fixture: no network, no token.
    Mock(MockSource),
    /// The PaliCanon API.
    Real(RealSource),
}

impl SearchSource for Source {
    async fn search(
        &self,
        query: &str,
        lang: &str,
        mode: Mode,
        collection: Collection,
    ) -> SearchOutcome {
        match self {
            Source::Mock(mock) => mock.search(query, lang, mode, collection).await,
            Source::Real(real) => real.search(query, lang, mode, collection).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::search::real::ApiToken;
    use crate::search::test_support::{MockApi, Reply};

    const LONG: Duration = Duration::from_secs(10);

    /// Which variant a source is: an exhaustive match, so a third variant
    /// fails to compile here until a test says how it answers.
    fn variant(source: &Source) -> &'static str {
        match source {
            Source::Mock(_) => "mock",
            Source::Real(_) => "real",
        }
    }

    fn real(api: &MockApi) -> Source {
        Source::Real(RealSource::new(api.url(), ApiToken::new("pc_canary"), LONG))
    }

    #[tokio::test]
    async fn the_mock_variant_answers_through_the_trait() {
        let source = Source::Mock(MockSource);
        assert_eq!(variant(&source), "mock");
        let SearchOutcome::Hits(res) = source
            .search("satipatthana", "en", Mode::Both, Collection::All)
            .await
        else {
            panic!("expected Hits from the mock");
        };
        assert_eq!(res.hits.len(), 2, "the mock's fixture: mn10 and dn22");
    }

    #[tokio::test]
    async fn the_mock_variant_keeps_its_sentinels() {
        let source = Source::Mock(MockSource);
        let outcome = source
            .search(
                crate::search::mock::COLD_QUERY,
                "en",
                Mode::Both,
                Collection::All,
            )
            .await;
        assert_eq!(
            outcome,
            SearchOutcome::Cold {
                retry_after: Some(10)
            }
        );
    }

    #[tokio::test]
    async fn the_real_variant_answers_through_the_trait_from_the_api() {
        let api = MockApi::start(vec![Reply::hits()]).await;
        let source = real(&api);
        assert_eq!(variant(&source), "real");
        let SearchOutcome::Hits(res) = source
            .search("metta", "en", Mode::Both, Collection::All)
            .await
        else {
            panic!("expected Hits from the API");
        };
        assert_eq!(api.calls(), 1, "the call went to the API");
        assert_eq!(res.hits.len(), 1, "the API's one hit, not the mock's");
        assert_eq!(res.hits[0].sutta_uid, "mn10");
    }

    #[tokio::test]
    async fn the_real_variant_maps_the_apis_refusals() {
        let api = MockApi::start(vec![
            Reply::raw(429, "slow down").with_header("retry-after", "7"),
        ])
        .await;
        let outcome = real(&api)
            .search("metta", "en", Mode::Both, Collection::All)
            .await;
        assert_eq!(
            outcome,
            SearchOutcome::RateLimited {
                retry_after: Some(7)
            }
        );
    }
}
