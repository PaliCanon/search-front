//! The shape of a search: mirrors the wire format of the API's
//! `GET /v1/search` (`PaliCanon/api`'s `api::{SearchResponse, HitDto}`,
//! `search_core::{Mode, Sources}`) closely enough that the real client
//! can plug in behind [`SearchSource`] without this service's view code
//! changing. A source answers with a [`SearchOutcome`]: hits, or one of
//! the three ways a search does not answer. The mock
//! (`mock::MockSource`, server-only: the WASM never holds it) is the
//! in-repo source; nothing in this file calls the network or holds a token.

#[cfg(feature = "ssr")]
pub(crate) mod adapt;
#[cfg(feature = "ssr")]
pub mod mock;
#[cfg(feature = "ssr")]
pub(crate) mod real;
pub mod server;
#[cfg(feature = "ssr")]
pub(crate) mod source;
#[cfg(all(test, feature = "ssr"))]
pub(crate) mod test_support;

use serde::{Deserialize, Serialize};

/// `words` (BM25 only) · `meaning` (dense only) · `both` (both, fused —
/// the API calls this `hybrid`; the UI's own word for it is "both").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    /// Both branches: the UI's default.
    #[default]
    Both,
    /// Exact words, Pāli spellings.
    Words,
    /// What the passage means.
    Meaning,
}

impl Mode {
    /// The `?mode=` code: `both`, `words`, `meaning` — never refused, an
    /// unknown value just falls back to [`Mode::Both`] ([`Mode::parse`]).
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Mode::Both => "both",
            Mode::Words => "words",
            Mode::Meaning => "meaning",
        }
    }

    /// Parses a `?mode=` value; anything but `words` or `meaning` is
    /// [`Mode::Both`] (never a refusal: a stray `?mode=` is not a query
    /// to reject, only a filter to default).
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        match raw {
            "words" => Mode::Words,
            "meaning" => Mode::Meaning,
            _ => Mode::Both,
        }
    }

    /// Every value, in the order the mode chips show them.
    #[must_use]
    pub fn all() -> [Mode; 3] {
        [Mode::Words, Mode::Meaning, Mode::Both]
    }
}

/// Which collection a search is narrowed to: every sutta, or one of the
/// five nikāyas. `?c=` carries its [`Collection::code`]; anything else is
/// [`Collection::All`] (a stray value is a filter to default, never a
/// refusal — like [`Mode::parse`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Collection {
    /// No narrowing: the default.
    #[default]
    All,
    /// Dīgha Nikāya, the long discourses.
    Dn,
    /// Majjhima Nikāya, the middle-length discourses.
    Mn,
    /// Saṁyutta Nikāya, the linked discourses.
    Sn,
    /// Aṅguttara Nikāya, the numbered discourses.
    An,
    /// Khuddaka Nikāya, the minor collection.
    Kn,
}

impl Collection {
    /// The `?c=` code: `all`, `dn`, `mn`, `sn`, `an`, `kn`.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Collection::All => "all",
            Collection::Dn => "dn",
            Collection::Mn => "mn",
            Collection::Sn => "sn",
            Collection::An => "an",
            Collection::Kn => "kn",
        }
    }

    /// Parses a `?c=` value; anything unknown is [`Collection::All`].
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        Collection::all()
            .into_iter()
            .find(|c| c.code() == raw)
            .unwrap_or_default()
    }

    /// Every value, in the order the dropdown shows them.
    #[must_use]
    pub fn all() -> [Collection; 6] {
        [
            Collection::All,
            Collection::Dn,
            Collection::Mn,
            Collection::Sn,
            Collection::An,
            Collection::Kn,
        ]
    }

    /// Whether a sutta (by its SuttaCentral uid, `mn10`) belongs here.
    #[must_use]
    pub fn admits(self, sutta_uid: &str) -> bool {
        match self {
            Collection::All => true,
            Collection::Kn => KHUDDAKA_BOOKS.iter().any(|book| numbered(sutta_uid, book)),
            nikaya => numbered(sutta_uid, nikaya.code()),
        }
    }
}

/// The Khuddaka Nikāya's books, by their SuttaCentral uid prefix (none of
/// them starts with `kn`).
const KHUDDAKA_BOOKS: [&str; 19] = [
    "kp", "dhp", "ud", "iti", "snp", "vv", "pv", "thag", "thig", "ja", "mnd", "cnd", "ps", "ne",
    "pe", "mil", "bv", "cp", "ap",
];

/// Whether `uid` is `prefix` followed by a number (`sn56.11` is SN, but
/// `snp1.8` — the Sutta Nipāta — is not).
fn numbered(uid: &str, prefix: &str) -> bool {
    uid.strip_prefix(prefix)
        .and_then(|rest| rest.chars().next())
        .is_some_and(|c| c.is_ascii_digit())
}

/// The sutta's title in each language with a text of it (api's
/// `corpus::Titles`), so the page can show the reader's own language.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Titles {
    /// English title, when a text exists in English.
    pub en: Option<String>,
    /// French title, when a text exists in French.
    pub fr: Option<String>,
    /// Vietnamese title, when a text exists in Vietnamese.
    pub vi: Option<String>,
}

/// Which branches found this hit (api's `search_core::Sources`): the
/// found-via tag reads this, never the raw score.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sources {
    /// The dense (meaning) branch found it.
    pub dense: bool,
    /// The sparse (words) branch found it.
    pub sparse: bool,
}

impl Sources {
    /// The found-via [`Mode`] this combination reads as: both branches is
    /// [`Mode::Both`], one alone is its own mode; neither (never produced
    /// by a real backend, but not a value to panic on) also reads as
    /// [`Mode::Both`] rather than inventing a fourth case.
    #[must_use]
    pub fn found_via(self) -> Mode {
        match (self.dense, self.sparse) {
            (true, false) => Mode::Meaning,
            (false, true) => Mode::Words,
            _ => Mode::Both,
        }
    }
}

/// What a hit's snippet is (api's `snippet_kind`): the sutta's best
/// passage for the query, quoted; or its blurb, a plain paragraph.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SnippetKind {
    /// A passage of the text itself: shown as a quotation.
    Passage,
    /// The sutta's blurb: shown as a paragraph.
    #[default]
    Blurb,
}

/// One hit, wire format (api's `HitDto`, trimmed to what the page shows).
/// `PartialEq` only: the score is a float.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HitDto {
    /// SuttaCentral uid, e.g. `mn10`.
    pub sutta_uid: String,
    /// The reference shown beside the title, e.g. `MN 10`.
    pub display_id: String,
    /// The sutta's title in the language actually searched.
    pub title: String,
    /// The Pāli title, when there is one.
    pub pali_title: Option<String>,
    /// The sutta's title in every language with a text of it.
    pub titles: Titles,
    /// What the card shows below the title: the blurb, or the sutta's
    /// best passage for the query.
    pub snippet: String,
    /// Which of the two `snippet` is.
    pub snippet_kind: SnippetKind,
    /// The link to the sutta, as SuttaCentral serves it today.
    pub url: String,
    /// Which branches found it.
    pub sources: Sources,
    /// The fused score, higher is better; the relevance bar shows it
    /// relative to the page's best hit.
    pub score: f32,
}

/// Wire format of `GET /v1/search` (api's `SearchResponse`, trimmed to
/// what the page shows: no timings, no cache or degraded flags — nothing
/// a mock can honestly report).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SearchResponse {
    /// The query as asked.
    pub query: String,
    /// The mode actually searched.
    pub mode: Mode,
    /// The hits, best first.
    pub hits: Vec<HitDto>,
    /// The language asked for had no index, so the English texts were
    /// searched instead and the page says so.
    #[serde(default)]
    pub fell_back: bool,
}

impl SearchResponse {
    /// An answer with no hit, for `query`, in the default [`Mode`]: what a
    /// blank search answers without asking the API.
    #[must_use]
    pub fn empty(query: impl Into<String>) -> Self {
        SearchResponse {
            query: query.into(),
            mode: Mode::default(),
            hits: Vec::new(),
            fell_back: false,
        }
    }
}

/// Why a search did not answer with hits: the three non-hit outcomes
/// (the engine is waking, the API is rate-limiting, anything else), kept
/// as an error type for a fallible step before it becomes a
/// [`SearchOutcome`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchFailure {
    /// The engine is waking up: try again in a moment.
    Cold {
        /// Seconds the API asked to wait, when it said.
        retry_after: Option<u64>,
    },
    /// Too many searches: wait a moment.
    RateLimited {
        /// Seconds the API asked to wait, when it said.
        retry_after: Option<u64>,
    },
    /// Anything else: the generic failed state.
    Failed,
}

impl From<SearchFailure> for SearchOutcome {
    fn from(failure: SearchFailure) -> Self {
        match failure {
            SearchFailure::Cold { retry_after } => SearchOutcome::Cold { retry_after },
            SearchFailure::RateLimited { retry_after } => {
                SearchOutcome::RateLimited { retry_after }
            }
            SearchFailure::Failed => SearchOutcome::Failed,
        }
    }
}

/// What a search answers: always a value, never an error, so a cold
/// engine is a state the page renders and not a 500 in monitoring. It
/// crosses the wire between the server function and the browser.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SearchOutcome {
    /// The search answered, with hits or none.
    Hits(SearchResponse),
    /// The engine is waking up (or a call timed out waiting for it).
    Cold {
        /// Seconds the API asked to wait, when it said.
        retry_after: Option<u64>,
    },
    /// The API is rate-limiting this front.
    RateLimited {
        /// Seconds the API asked to wait, when it said.
        retry_after: Option<u64>,
    },
    /// Anything else went wrong.
    Failed,
}

/// What finds hits for a query: the mock (`mock::MockSource`) or the real
/// `/v1/search` client — the page's view code does not change either way.
#[allow(async_fn_in_trait)] // one concrete enum implements it: no `Send` bound to name
pub trait SearchSource {
    /// Every hit for `query` in `lang`, `mode` and `collection`, best
    /// first, or the reason there are none to show. Never fails as a
    /// function: an empty result is `Hits` with no hits, and a backend
    /// that could not answer is one of the other [`SearchOutcome`]s.
    async fn search(
        &self,
        query: &str,
        lang: &str,
        mode: Mode,
        collection: Collection,
    ) -> SearchOutcome;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_code_round_trips_through_parse() {
        for mode in Mode::all() {
            assert_eq!(Mode::parse(mode.code()), mode);
        }
    }

    #[test]
    fn an_unknown_mode_value_defaults_to_both_not_a_refusal() {
        assert_eq!(Mode::parse(""), Mode::Both);
        assert_eq!(Mode::parse("hybrid"), Mode::Both);
        assert_eq!(Mode::parse("bogus"), Mode::Both);
    }

    #[test]
    fn found_via_reads_one_branch_as_its_own_mode() {
        assert_eq!(
            Sources {
                dense: true,
                sparse: false
            }
            .found_via(),
            Mode::Meaning
        );
        assert_eq!(
            Sources {
                dense: false,
                sparse: true
            }
            .found_via(),
            Mode::Words
        );
    }

    #[test]
    fn found_via_reads_both_branches_as_both() {
        assert_eq!(
            Sources {
                dense: true,
                sparse: true
            }
            .found_via(),
            Mode::Both
        );
    }

    #[test]
    fn collection_code_round_trips_through_parse() {
        for collection in Collection::all() {
            assert_eq!(Collection::parse(collection.code()), collection);
        }
    }

    #[test]
    fn an_unknown_collection_defaults_to_all_not_a_refusal() {
        assert_eq!(Collection::parse(""), Collection::All);
        assert_eq!(Collection::parse("vinaya"), Collection::All);
    }

    #[test]
    fn a_nikaya_admits_its_own_suttas_only() {
        assert!(Collection::Mn.admits("mn10"));
        assert!(!Collection::Mn.admits("dn22"));
        assert!(Collection::Sn.admits("sn56.11"));
        assert!(Collection::All.admits("dn22"));
    }

    /// The Khuddaka's books have their own uids (`dhp`, `snp`, `ud`…), never
    /// a `kn` prefix: a plain prefix test would admit nothing.
    #[test]
    fn the_khuddaka_admits_its_books() {
        assert!(Collection::Kn.admits("dhp221-234"));
        assert!(Collection::Kn.admits("snp1.8"));
        assert!(!Collection::Kn.admits("mn10"));
    }

    fn hits_outcome() -> SearchOutcome {
        SearchOutcome::Hits(SearchResponse {
            query: "satipaṭṭhāna".to_owned(),
            mode: Mode::Both,
            hits: Vec::new(),
            fell_back: true,
        })
    }

    #[test]
    fn outcome_serde_round_trip() {
        let outcomes = [
            hits_outcome(),
            SearchOutcome::Cold {
                retry_after: Some(7),
            },
            SearchOutcome::Cold { retry_after: None },
            SearchOutcome::RateLimited {
                retry_after: Some(30),
            },
            SearchOutcome::RateLimited { retry_after: None },
            SearchOutcome::Failed,
        ];
        for outcome in outcomes {
            let json = serde_json::to_string(&outcome).unwrap();
            let back: SearchOutcome = serde_json::from_str(&json).unwrap();
            assert_eq!(back, outcome, "{json}");
        }
    }

    #[test]
    fn a_response_without_fell_back_reads_as_false() {
        let json = r#"{"query":"q","mode":"Both","hits":[]}"#;
        let response: SearchResponse = serde_json::from_str(json).unwrap();
        assert!(!response.fell_back);
    }

    #[test]
    fn a_failure_becomes_the_outcome_of_the_same_name() {
        assert_eq!(
            SearchOutcome::from(SearchFailure::Cold {
                retry_after: Some(7)
            }),
            SearchOutcome::Cold {
                retry_after: Some(7)
            }
        );
        assert_eq!(
            SearchOutcome::from(SearchFailure::RateLimited { retry_after: None }),
            SearchOutcome::RateLimited { retry_after: None }
        );
        assert_eq!(
            SearchOutcome::from(SearchFailure::Failed),
            SearchOutcome::Failed
        );
    }
}
