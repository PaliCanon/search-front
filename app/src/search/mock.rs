//! The in-repo mock [`SearchSource`]: a small fixed corpus, filtered by
//! substring match and by [`Mode`] compatibility — enough to show every
//! page state (results, no results, cold, rate-limited, failed) without a
//! backend. The real client answers through the same trait; the page's
//! view code reads [`SearchSource`] either way.

use super::{
    Collection, HitDto, Mode, SearchOutcome, SearchResponse, SearchSource, SnippetKind, Sources,
    Titles,
};

/// A query the mock answers [`SearchOutcome::Failed`] to on purpose, so
/// the page's error state has something to render. Not a value a visitor
/// would type by accident.
pub const ERROR_QUERY: &str = "__search_front_mock_error__";

/// A query the mock answers [`SearchOutcome::Cold`] to (with a
/// `retry_after` of 10 seconds): the engine-is-waking state.
pub const COLD_QUERY: &str = "__search_front_mock_cold__";

/// A query the mock answers [`SearchOutcome::RateLimited`] to (with a
/// `retry_after` of 10 seconds): the too-many-searches state.
pub const RATE_LIMITED_QUERY: &str = "__search_front_mock_rate_limited__";

/// The `retry_after` the mock's cold and rate-limited answers carry.
const MOCK_RETRY_AFTER: u64 = 10;

/// The mock's fixed corpus: title, Pāli title, the other languages' titles,
/// a snippet, and which branches "found" it — varied on purpose so every
/// [`Mode`] has at least one hit that matches it alone (one hit per
/// function, `too-many-lines-threshold` in `clippy.toml`).
fn fixture() -> Vec<HitDto> {
    vec![satipatthana(), mahasatipatthana(), dhammacakkappavattana()]
}

/// Found by both branches: the words-and-meaning fixture.
fn satipatthana() -> HitDto {
    HitDto {
        sutta_uid: "mn10".to_owned(),
        display_id: "MN 10".to_owned(),
        title: "Satipatthana Sutta".to_owned(),
        pali_title: Some("Satipaṭṭhānasutta".to_owned()),
        titles: Titles {
            en: Some("The Foundations of Mindfulness".to_owned()),
            fr: Some("Les fondements de l'attention".to_owned()),
            vi: Some("Kinh Niệm Xứ".to_owned()),
        },
        snippet: "A bhikkhu understands mindfulness as a path to liberation.".to_owned(),
        snippet_kind: SnippetKind::Passage,
        url: "https://suttacentral.net/mn10".to_owned(),
        sources: Sources {
            dense: true,
            sparse: true,
        },
        score: 0.92,
    }
}

/// Found by the dense branch alone: the meaning-only fixture.
fn mahasatipatthana() -> HitDto {
    HitDto {
        sutta_uid: "dn22".to_owned(),
        display_id: "DN 22".to_owned(),
        title: "Mahasatipatthana Sutta".to_owned(),
        pali_title: Some("Mahāsatipaṭṭhānasutta".to_owned()),
        titles: Titles {
            en: Some("The Great Discourse on the Foundations of Mindfulness".to_owned()),
            fr: None,
            vi: None,
        },
        snippet: "The longer version of the same mindfulness teaching.".to_owned(),
        snippet_kind: SnippetKind::Blurb,
        url: "https://suttacentral.net/dn22".to_owned(),
        sources: Sources {
            dense: true,
            sparse: false,
        },
        score: 0.61,
    }
}

/// Found by the sparse branch alone: the words-only fixture.
fn dhammacakkappavattana() -> HitDto {
    HitDto {
        sutta_uid: "sn56.11".to_owned(),
        display_id: "SN 56.11".to_owned(),
        title: "Dhammacakkappavattana Sutta".to_owned(),
        pali_title: Some("Dhammacakkappavattanasutta".to_owned()),
        titles: Titles {
            en: Some("Setting the Wheel of Dhamma in Motion".to_owned()),
            fr: None,
            vi: None,
        },
        snippet: "The Buddha's first teaching: the four noble truths.".to_owned(),
        snippet_kind: SnippetKind::Blurb,
        url: "https://suttacentral.net/sn56.11".to_owned(),
        sources: Sources {
            dense: false,
            sparse: true,
        },
        score: 0.38,
    }
}

/// Whether `hit` matches `query`, case-insensitively, by its title, Pāli
/// title, snippet or uid.
fn matches(hit: &HitDto, query: &str) -> bool {
    let query = query.to_lowercase();
    hit.title.to_lowercase().contains(&query)
        || hit
            .pali_title
            .as_deref()
            .unwrap_or_default()
            .to_lowercase()
            .contains(&query)
        || hit.snippet.to_lowercase().contains(&query)
        || hit.sutta_uid.to_lowercase().contains(&query)
}

/// Whether `hit` was found through `mode`: [`Mode::Both`] accepts
/// anything, [`Mode::Words`] needs the sparse branch, [`Mode::Meaning`]
/// the dense one — the same pair this mock's [`Sources`] varies the
/// fixture on, so each mode actually narrows the results.
fn matches_mode(hit: &HitDto, mode: Mode) -> bool {
    match mode {
        Mode::Both => true,
        Mode::Words => hit.sources.sparse,
        Mode::Meaning => hit.sources.dense,
    }
}

/// The in-repo mock: fixed hits, filtered by `query`, `mode` and
/// `collection`; the language is ignored (the fixture holds every title).
#[derive(Clone, Copy, Debug, Default)]
pub struct MockSource;

impl MockSource {
    /// The mock's answer, computed at once: the page (which renders
    /// synchronously until the server function replaces its call, task
    /// 005) reads it directly; [`SearchSource::search`] is this, awaited.
    #[must_use]
    pub fn answer(&self, query: &str, mode: Mode, collection: Collection) -> SearchOutcome {
        match query {
            ERROR_QUERY => return SearchOutcome::Failed,
            COLD_QUERY => {
                return SearchOutcome::Cold {
                    retry_after: Some(MOCK_RETRY_AFTER),
                };
            }
            RATE_LIMITED_QUERY => {
                return SearchOutcome::RateLimited {
                    retry_after: Some(MOCK_RETRY_AFTER),
                };
            }
            _ => {}
        }
        let hits: Vec<HitDto> = fixture()
            .into_iter()
            .filter(|hit| {
                matches(hit, query) && matches_mode(hit, mode) && collection.admits(&hit.sutta_uid)
            })
            .collect();
        SearchOutcome::Hits(SearchResponse {
            query: query.to_owned(),
            mode,
            hits,
            fell_back: false,
        })
    }
}

impl SearchSource for MockSource {
    // The trait is async because the real source awaits the network; the
    // mock has nothing to wait for.
    #[allow(clippy::unused_async_trait_impl)]
    async fn search(
        &self,
        query: &str,
        _lang: &str,
        mode: Mode,
        collection: Collection,
    ) -> SearchOutcome {
        self.answer(query, mode, collection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The hits of an answer that must be `Hits`, or a failed test.
    fn hits(outcome: SearchOutcome) -> Vec<HitDto> {
        match outcome {
            SearchOutcome::Hits(response) => response.hits,
            other => panic!("expected hits, got {other:?}"),
        }
    }

    async fn ask(query: &str, mode: Mode, collection: Collection) -> SearchOutcome {
        MockSource.search(query, "en", mode, collection).await
    }

    #[tokio::test]
    async fn a_matching_query_returns_its_hits() {
        let found = hits(ask("satipatthana", Mode::Both, Collection::All).await);
        assert_eq!(found.len(), 2, "mn10 and dn22 both mention it");
    }

    #[tokio::test]
    async fn an_unmatched_query_is_hits_with_none_not_a_failure() {
        let found = hits(ask("qqzzqq-nothing-like-this", Mode::Both, Collection::All).await);
        assert_eq!(found, Vec::<HitDto>::new());
    }

    #[tokio::test]
    async fn the_words_mode_keeps_only_sparse_hits() {
        let found = hits(ask("sutta", Mode::Words, Collection::All).await);
        assert!(found.iter().all(|h| h.sources.sparse), "{found:?}");
        assert!(
            found.iter().any(|h| h.sutta_uid == "sn56.11"),
            "the words-only fixture should still show up"
        );
    }

    #[tokio::test]
    async fn the_meaning_mode_keeps_only_dense_hits() {
        let found = hits(ask("sutta", Mode::Meaning, Collection::All).await);
        assert!(found.iter().all(|h| h.sources.dense), "{found:?}");
        assert!(!found.iter().any(|h| h.sutta_uid == "sn56.11"));
    }

    #[tokio::test]
    async fn the_error_sentinel_is_failed() {
        assert_eq!(
            ask(ERROR_QUERY, Mode::Both, Collection::All).await,
            SearchOutcome::Failed
        );
    }

    #[tokio::test]
    async fn the_cold_sentinel_is_cold_with_a_retry_after() {
        assert_eq!(
            ask(COLD_QUERY, Mode::Both, Collection::All).await,
            SearchOutcome::Cold {
                retry_after: Some(10)
            }
        );
    }

    #[tokio::test]
    async fn the_rate_limited_sentinel_is_rate_limited_with_a_retry_after() {
        assert_eq!(
            ask(RATE_LIMITED_QUERY, Mode::Both, Collection::All).await,
            SearchOutcome::RateLimited {
                retry_after: Some(10)
            }
        );
    }

    /// T4 (review 001): the sentinels are checked before mode-filtering —
    /// each must answer under every `Mode` and every collection, not only
    /// the defaults, or a filtered path would ship untested.
    #[tokio::test]
    async fn the_sentinels_answer_under_every_mode_and_collection() {
        for mode in Mode::all() {
            for collection in Collection::all() {
                assert_eq!(
                    ask(ERROR_QUERY, mode, collection).await,
                    SearchOutcome::Failed,
                    "{mode:?} {collection:?}"
                );
                assert!(
                    matches!(
                        ask(COLD_QUERY, mode, collection).await,
                        SearchOutcome::Cold { .. }
                    ),
                    "{mode:?} {collection:?}"
                );
                assert!(
                    matches!(
                        ask(RATE_LIMITED_QUERY, mode, collection).await,
                        SearchOutcome::RateLimited { .. }
                    ),
                    "{mode:?} {collection:?}"
                );
            }
        }
    }

    #[tokio::test]
    async fn a_collection_keeps_only_its_own_hits() {
        let found = hits(ask("sutta", Mode::Both, Collection::Mn).await);
        let uids: Vec<&str> = found.iter().map(|h| h.sutta_uid.as_str()).collect();
        assert_eq!(uids, ["mn10"]);
    }

    /// Collection and mode together: both narrow, neither overrides the
    /// other (SN 56.11 is words-only, so `meaning` within SN finds nothing).
    #[tokio::test]
    async fn collection_and_mode_narrow_together() {
        let words = hits(ask("sutta", Mode::Words, Collection::Sn).await);
        assert_eq!(words.len(), 1, "{words:?}");
        let meaning = hits(ask("sutta", Mode::Meaning, Collection::Sn).await);
        assert!(meaning.is_empty(), "{meaning:?}");
    }

    /// The language is the real source's business: the mock answers the
    /// same hits whatever it is.
    #[tokio::test]
    async fn the_mock_ignores_the_language() {
        let en = MockSource
            .search("sutta", "en", Mode::Both, Collection::All)
            .await;
        let fr = MockSource
            .search("sutta", "fr", Mode::Both, Collection::All)
            .await;
        assert_eq!(en, fr);
    }
}
