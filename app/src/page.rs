//! The one page: reads `?q=&c=&mode=` (and the home's `?passage=`, which
//! defaults to the verse of the day) from the request's own URL, so the server renders the right state on first
//! paint — no JavaScript needed to see a shared link's results — and
//! follows it on a client-side navigation. With no query it is the home,
//! centred (brand, card, example queries, the passage); with one, today's
//! site's results page: a full-width header (the brand, the one-line
//! search bar beside it), the filter chips and the results in the bar's
//! column under it.

mod pending;
#[cfg(feature = "ssr")]
mod response;

#[cfg(all(test, feature = "ssr"))]
pub(crate) mod harness;
#[cfg(all(test, feature = "ssr"))]
mod tests_fake;
#[cfg(all(test, feature = "ssr"))]
mod tests_modes;
#[cfg(all(test, feature = "ssr"))]
mod tests_real;

use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use leptos_router::params::ParamsMap;
use palicanon_ui_lib::{Language, language_signal};

use crate::components::filters::{CollectionSelect, ModeSelect};
use crate::components::home::{Brand, BrandLink, DailyPassage, Examples};
use crate::components::result_list::ResultList;
use crate::components::search_bar::{ResultsSearchBar, SEARCH_FORM_ID, SearchBar};
use crate::components::states::{ColdState, ErrorState, RateLimitedState};
use crate::fragment::submit::InPlaceSubmit;
use crate::links::{Base, Retry, search_href};
use crate::search::backend::{Provided, SearchBackend, SearchQuery};
use crate::search::{Collection, Mode, SearchOutcome, SearchResponse};
use pending::{Area, pending_view};

/// What one search asks: the URL's `?q=&mode=&c=` and the app's language.
/// The page's one `Resource` is keyed on it, so a change of any of the four
/// (a client-side navigation, a language switch) asks again.
#[derive(Clone, Debug, PartialEq)]
struct SearchKey {
    /// The query, trimmed: `?q=metta` and `?q=%20metta%20` are one key, so
    /// one search is one fetch (the server fn caps it, and trims again).
    q: String,
    /// The app's language code.
    lang: &'static str,
    /// The mode `?mode=` resolved to.
    mode: Mode,
    /// The collection `?c=` resolved to.
    collection: Collection,
    /// The home's `?passage=`, raw: it picks the verse, asks nothing, and is
    /// kept by every link and submit so a shared URL survives them.
    passage: String,
}

impl SearchKey {
    /// The key of one URL's query parameters.
    fn read(params: &ParamsMap, language: Language) -> Self {
        SearchKey {
            q: params.get("q").unwrap_or_default().trim().to_owned(),
            lang: language.code(),
            mode: Mode::parse(params.get("mode").unwrap_or_default().as_str()),
            collection: Collection::parse(params.get("c").unwrap_or_default().as_str()),
            passage: params.get("passage").unwrap_or_default(),
        }
    }

    /// No query to search: the page is the home.
    fn is_blank(&self) -> bool {
        self.q.trim().is_empty()
    }

    /// Whether `other` asks the same search: the query, the language, the
    /// mode asked and the collection (not `?passage=`, which only picks the
    /// home's verse).
    fn same_search(&self, other: &SearchKey) -> bool {
        (&self.q, self.lang, self.mode, self.collection)
            == (&other.q, other.lang, other.mode, other.collection)
    }

    /// What the backend is asked for this key: the mode asked while
    /// `meaning` says Meaning and Both can be served, Words otherwise. The
    /// key itself keeps the mode asked, which every link is built from, so
    /// a downgrade never rewrites a shareable URL.
    fn query(&self, meaning: bool) -> SearchQuery {
        SearchQuery {
            q: self.q.clone(),
            lang: self.lang,
            mode: if meaning { self.mode } else { Mode::Words },
            collection: self.collection,
        }
    }

    /// This search as the page's own GET on `base`, what the retry links
    /// point at, built from the mode asked.
    fn href(&self, base: &str) -> String {
        search_href(base, &self.q, self.collection, self.mode, &self.passage)
    }
}

/// The page's only search path: the host's [`SearchBackend`], whichever
/// side runs it (the site's server function, the app's local engine). A
/// blank query asks nothing and answers an empty outcome here.
async fn fetch(backend: Arc<dyn SearchBackend>, query: SearchQuery) -> SearchOutcome {
    if query.q.is_empty() {
        return SearchOutcome::Hits(SearchResponse::empty(""));
    }
    backend.search(query).await
}

/// The page's one `Resource`, keyed on the query the backend is asked
/// ([`SearchKey::query`]: the mode resolved against `meaning`) and on the
/// retry count. A model arriving flips `meaning`, which asks the same search
/// again as asked; a [`Retry`] bump asks the same search again whatever the
/// outcome was. Leptos runs a `Resource`'s fetches one after the other, so
/// an answer in flight lands first (shown dimmed while the next is pending)
/// and can never overwrite a fresher one.
fn search_resource(
    backend: Arc<dyn SearchBackend>,
    key: Signal<SearchKey>,
    meaning: Signal<bool>,
    retry: Retry,
) -> Resource<SearchOutcome> {
    Resource::new(
        move || (key.get().query(meaning.get()), retry.get()),
        move |(query, _)| fetch(Arc::clone(&backend), query),
    )
}

/// The search page: the home while `?q=` is blank, the results otherwise.
///
/// The page follows the URL and the app's language: one `Resource`, keyed
/// on them ([`SearchKey`]), holds the search's outcome, and the route is
/// rendered `SsrMode::Async` so the server's first paint already carries
/// it — no JavaScript needed to see a shared link's results. The home
/// never reads the `Resource` (its blank key asks nothing), so it renders
/// as before; the results area sits under one `Transition`, so a new
/// search keeps the previous hits (dimmed, `aria-busy`) only while it is
/// pending.
#[component]
pub fn SearchPage() -> impl IntoView {
    let Provided { backend, meaning } = use_context::<Provided>()
        .expect("SearchPage needs a SearchProvider above it (the host's backend)");
    let params = use_query_map();
    let key = Memo::new(move |_| SearchKey::read(&params.get(), language_signal().get()));
    let retry = Retry::new();
    provide_context(retry);
    let outcome = search_resource(backend, key.into(), meaning, retry);
    let base = Base::matched();
    let home = Memo::new(move |_| key.get().is_blank());
    let language = language_signal();
    let same_search = Callback::new(move |sent: ParamsMap| {
        SearchKey::read(&sent, language.get_untracked()).same_search(&key.get_untracked())
    });
    view! {
        <InPlaceSubmit base=base retry=retry same_search=same_search>
            {move || {
                if home.get() {
                    home_view(params, meaning, base).into_any()
                } else {
                    results_view(key, outcome, meaning, base)
                }
            }}
        </InPlaceSubmit>
    }
}

/// The home: centred, the brand, the card, the example queries and the
/// passage. It follows `?passage=` and the card's own filters.
fn home_view(params: Memo<ParamsMap>, meaning: Signal<bool>, base: Base) -> impl IntoView {
    move || {
        let params = params.get();
        let query = params.get("q").unwrap_or_default();
        let mode = Mode::parse(params.get("mode").unwrap_or_default().as_str());
        let collection = Collection::parse(params.get("c").unwrap_or_default().as_str());
        let passage = params.get("passage").unwrap_or_default();
        view! {
            <div class="flex flex-col">
                <div class="flex flex-col items-center px-6 pt-[max(4rem,20vh)] pb-12">
                    <div class="w-full max-w-[42rem]">
                        <Brand />
                        <SearchBar value=query mode=mode collection=collection meaning=meaning />
                        <div class="mt-5">
                            <Examples base=base />
                        </div>
                    </div>
                </div>
                <section class="px-6 pt-16 pb-24 sm:pt-28">
                    <DailyPassage base=base raw=passage />
                </section>
            </div>
        }
    }
}

/// The results page: a full-width header (the brand, the bar), the chips
/// and the results area in the bar's column. The header and chips follow
/// the key; the results area, which holds the `Transition`, stays mounted
/// across searches so the stale hits can stay while one is pending.
fn results_view(
    key: Memo<SearchKey>,
    outcome: Resource<SearchOutcome>,
    meaning: Signal<bool>,
    base: Base,
) -> AnyView {
    view! {
        {move || {
            let q = key.get().q;
            view! {
                <ResultsHeader base=base>
                    <ResultsSearchBar value=q />
                </ResultsHeader>
            }
        }}
        <main class=MAIN>
            <div class="min-w-0 sm:col-start-2">
                {move || {
                    let key = key.get();
                    let mode = meaning
                        .get()
                        .then(|| {
                            view! {
                                <ModeSelect
                                    value=key.mode
                                    form=SEARCH_FORM_ID
                                    submit_on_change=true
                                />
                            }
                        });
                    view! {
                        <div class="flex flex-wrap items-center gap-2 pt-3">
                            <CollectionSelect
                                value=key.collection
                                form=SEARCH_FORM_ID
                                submit_on_change=true
                            />
                            {mode}
                        </div>
                    }
                }} <ResultsArea key=key outcome=outcome base=base />
            </div>
        </main>
    }
    .into_any()
}

/// The results under one `Transition`: the first load shows its fallback,
/// a later search keeps the previous hits ([`pending_view`]) while it is
/// pending, and the outcome that lands always replaces them. On the server
/// the outcome also sets the response (status, `Retry-After`,
/// `Cache-Control`).
#[component]
fn ResultsArea(
    /// The search being shown.
    key: Memo<SearchKey>,
    /// Its outcome.
    outcome: Resource<SearchOutcome>,
    /// Where the fragment is mounted: the retry links go to its page.
    base: Base,
) -> impl IntoView {
    let (pending, set_pending) = signal(false);
    view! {
        <Transition
            fallback=|| view! { <div aria-busy="true" class=pending::DIM></div> }
            set_pending=set_pending
        >
            {move || {
                let held = outcome.get();
                area_view(&pending_view(held.as_ref(), pending.get()), &key.get(), &base.get())
            }}
        </Transition>
    }
}

/// One state of the results area: its wrapper (busy or not) around the
/// outcome's view. A settled outcome is also written to the response.
fn area_view(area: &Area<'_>, key: &SearchKey, base: &str) -> AnyView {
    #[cfg(feature = "ssr")]
    if let Some(outcome) = area.outcome
        && !area.busy
    {
        response::apply(outcome, key.is_blank());
    }
    let shown = area
        .outcome
        .map(|outcome| outcome_view(outcome, key.href(base)));
    view! {
        <div aria-busy=area.aria_busy() class=area.class()>
            {shown}
        </div>
    }
    .into_any()
}

/// What the page shows for an outcome: the hits (or none), or the state
/// that says why there are none.
fn outcome_view(outcome: &SearchOutcome, href: String) -> AnyView {
    match outcome {
        SearchOutcome::Hits(response) => {
            view! { <ResultList response=response.clone() /> }.into_any()
        }
        SearchOutcome::Cold { .. } => view! { <ColdState href=href /> }.into_any(),
        SearchOutcome::RateLimited { .. } => view! { <RateLimitedState href=href /> }.into_any(),
        SearchOutcome::Failed => view! { <ErrorState /> }.into_any(),
    }
}

/// A results page's header, the page's full width with a rule under it:
/// a grid of the brand (a fixed first column, hidden on a phone), the
/// search bar (`children`, capped at the reading width) and a free last
/// column — today's site's header, whose last column held the language
/// and settings buttons the shell's rail carries here.
#[component]
fn ResultsHeader(
    /// Where the fragment is mounted: the brand links to its page.
    base: Base,
    children: Children,
) -> impl IntoView {
    view! {
        <header class="border-b border-line">
            <div class=HEADER_GRID>
                <BrandLink base=base />
                <div class="col-span-2 min-w-0 sm:col-span-1">{children()}</div>
            </div>
        </header>
    }
}

/// The header's grid: brand column, bar column, free column (desktop);
/// the bar alone, full width, on a phone.
const HEADER_GRID: &str = "mx-auto grid w-full max-w-[64rem] grid-cols-[1fr_auto] items-center gap-x-6 gap-y-3 px-6 py-4 \
    sm:grid-cols-[11rem_minmax(0,40rem)_1fr]";

/// The results page's main area: the same columns as the header, the
/// chips and results under the bar.
const MAIN: &str = "mx-auto w-full max-w-[64rem] flex-1 px-6 pb-20 sm:grid sm:grid-cols-[11rem_minmax(0,40rem)] sm:gap-x-6";

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::harness::{render_at, render_in};
    use super::*;

    fn key_of(q: &str) -> SearchKey {
        let mut params = ParamsMap::new();
        params.insert("q", q.to_owned());
        SearchKey::read(&params, Language::En)
    }

    /// The Resource refetches when its key changes: the padded and the
    /// plain query must be the same key, so one search is one fetch.
    #[test]
    fn the_key_is_the_trimmed_query() {
        assert_eq!(key_of(" metta "), key_of("metta"));
        assert_eq!(key_of(" metta ").q, "metta");
        assert_eq!(key_of("  ").q, "");
    }

    /// T2 (review 005, round 2): the key carries what the URL and the app's
    /// language say — the mode, the collection and the language code — not
    /// the defaults, so a change of any of them asks again.
    #[test]
    fn the_key_carries_the_mode_the_collection_and_the_language() {
        let mut params = ParamsMap::new();
        params.insert("q", "metta".to_owned());
        params.insert("mode", "words".to_owned());
        params.insert("c", "mn".to_owned());
        let key = SearchKey::read(&params, Language::Fr);
        assert_eq!(key.q, "metta");
        assert_eq!(key.mode, Mode::Words);
        assert_eq!(key.collection, Collection::Mn);
        assert_eq!(key.lang, "fr");
        let default = SearchKey::read(&ParamsMap::new(), Language::En);
        assert_ne!(key, default);
        assert_eq!(default.mode, Mode::Both);
        assert_eq!(default.collection, Collection::All);
        assert_eq!(default.lang, "en");
    }

    /// G1 (review 021, round 1): two keys ask the same search when the query,
    /// the language, the mode asked and the collection are equal; the
    /// home's `?passage=` is not part of it.
    #[test]
    fn the_same_search_ignores_the_passage_and_nothing_else() {
        let key = |q: &str, c: &str, mode: &str, passage: &str| {
            let mut params = ParamsMap::new();
            for (name, value) in [("q", q), ("c", c), ("mode", mode), ("passage", passage)] {
                params.insert(name, value.to_owned());
            }
            SearchKey::read(&params, Language::En)
        };
        let asked = key("metta", "mn", "words", "");
        assert!(asked.same_search(&key(" metta ", "mn", "words", "3")));
        for other in [
            key("karuna", "mn", "words", ""),
            key("metta", "sn", "words", ""),
            key("metta", "mn", "both", ""),
        ] {
            assert!(!asked.same_search(&other), "{other:?}");
        }
        let french = SearchKey::read(&ParamsMap::new(), Language::Fr);
        assert!(!SearchKey::read(&ParamsMap::new(), Language::En).same_search(&french));
    }

    /// T5 (review 005, round 3): the language alone rekeys the resource:
    /// two keys built from the same `ParamsMap` with different languages
    /// are never equal, so a language switch asks again.
    #[test]
    fn the_language_alone_rekeys_the_resource() {
        let mut params = ParamsMap::new();
        params.insert("q", "metta".to_owned());
        params.insert("mode", "words".to_owned());
        params.insert("c", "mn".to_owned());
        let key_fr = SearchKey::read(&params, Language::Fr);
        let key_en = SearchKey::read(&params, Language::En);
        assert_ne!(key_fr, key_en, "language change must trigger a new search");
        assert_eq!(key_fr.q, key_en.q);
        assert_eq!(key_fr.mode, key_en.mode);
        assert_eq!(key_fr.collection, key_en.collection);
        assert_eq!(key_fr.lang, "fr");
        assert_eq!(key_en.lang, "en");
    }

    /// G2 (review 021, round 1): one builder. A submitted form's raw values
    /// and the key read from the same parameters spell the same href, a
    /// field equal to its default is not written, and `?passage=` (a field
    /// the form never carries) is kept.
    #[test]
    fn the_form_href_and_the_key_href_are_one_string() {
        use crate::fragment::submit::form_href;
        // (base, q, c, mode, passage) → the href both spell.
        let table = [
            ("", "metta", "", "", "", "/?q=metta"),
            ("", "a b", "bogus", "bogus", "", "/?q=a+b"),
            (
                "/search",
                " metta ",
                "mn",
                "words",
                "",
                "/search?q=metta&c=mn&mode=words",
            ),
            ("/search", "metta", "all", "both", "", "/search?q=metta"),
            (
                "/search",
                "metta",
                "",
                "meaning",
                "",
                "/search?q=metta&mode=meaning",
            ),
            (
                "/search",
                "karuna",
                "sn",
                "",
                "2",
                "/search?q=karuna&c=sn&passage=2",
            ),
            ("/search", "", "", "", "2", "/search?passage=2"),
            ("/search", "  ", "", "", "", "/search"),
            ("", "", "", "", "", "/"),
        ];
        for (base, q, c, mode, passage, want) in table {
            let mut params = ParamsMap::new();
            for (name, value) in [("q", q), ("c", c), ("mode", mode), ("passage", passage)] {
                params.insert(name, value.to_owned());
            }
            let key = SearchKey::read(&params, Language::En);
            assert_eq!(
                key.href(base),
                want,
                "key {base} {q:?} {c} {mode} {passage}"
            );
            assert_eq!(
                form_href(base, q, c, mode, passage),
                want,
                "form {base} {q:?} {c} {mode} {passage}"
            );
        }
    }

    use crate::catalogue::strings;
    use crate::components::home::{POOL, VERSES, day_index, example_labels, form_encode, now_ms};

    #[tokio::test]
    async fn no_query_shows_the_home_on_first_paint() {
        let html = render_at("/").await;
        assert!(
            html.contains(r#"placeholder="Search the Pāli Canon""#),
            "{html}"
        );
        assert!(html.contains(r#"src="/lotus.png""#), "{html}");
        assert!(html.contains(">PaliCanon</h1>"), "{html}");
    }

    /// Round 3, revised by the user: the home stays in the middle of the
    /// page — the brand centred above the card, no header row.
    #[tokio::test]
    async fn the_home_stays_centred_with_the_brand_above_the_card() {
        let html = render_at("/").await;
        assert!(html.contains("pt-[max(4rem,20vh)]"), "{html}");
        let brand = html.find(">PaliCanon</h1>").expect(&html);
        let card = html.find(r#"role="search""#).expect(&html);
        assert!(brand < card, "{html}");
        assert!(!html.contains(r#"<a href="/""#), "{html}");
    }

    /// The opening tag of the brand's home link.
    fn brand_tag(html: &str) -> &str {
        let start = html.find(r#"<a href="/""#).expect(html);
        let end = start + html[start..].find('>').expect(html);
        &html[start..=end]
    }
    /// Round 3: on a phone the shell's top bar already shows the brand,
    /// so the results row's own is hidden below `sm:`.
    #[tokio::test]
    async fn the_brand_is_hidden_on_a_phone() {
        let html = render_at("/?q=metta").await;
        let tag = brand_tag(&html);
        assert!(tag.contains("hidden") && tag.contains("sm:flex"), "{tag}");
        assert!(tag.contains("text-lg"), "{tag}");
    }

    /// Round 5 (user check, 2026-10-05: the same as today's site): a
    /// results page opens on a full-width header with a rule under it —
    /// the brand, then the one-line bar in the reading column — and the
    /// filter chips and results sit in that same column under it.
    #[tokio::test]
    async fn the_results_page_has_todays_header_and_column() {
        let html = render_at("/?q=metta").await;
        let header = html.find("<header").expect(&html);
        let header_end = html.find("</header>").expect(&html);
        let head = &html[header..header_end];
        assert!(head.contains("border-b border-line"), "{head}");
        assert!(head.contains("max-w-[64rem]"), "{head}");
        assert!(
            head.contains("sm:grid-cols-[11rem_minmax(0,40rem)_1fr]"),
            "{head}"
        );
        let brand = head.find(r#"<a href="/""#).expect(head);
        let bar = head.find(r#"role="search""#).expect(head);
        assert!(brand < bar, "{head}");
        assert!(head.contains(r#"id="search-form""#), "{head}");
        let main = &html[header_end..];
        assert!(
            main.contains("sm:grid-cols-[11rem_minmax(0,40rem)]"),
            "{main}"
        );
        let chips = main.find(r#"name="c""#).expect(main);
        let results = main.find(r#"aria-live="polite""#).expect(main);
        assert!(chips < results, "{main}");
        assert!(!html.contains("<h1"), "{html}");
    }

    /// The chips sit under the header, outside the bar's form: they join
    /// its GET through the `form` attribute.
    #[tokio::test]
    async fn the_results_chips_belong_to_the_header_s_form() {
        let html = render_at("/?q=metta&c=mn&mode=words").await;
        let form_end = html.find("</form>").expect(&html);
        let chips = &html[form_end..];
        assert!(
            chips.contains(r#"<select name="c" form="search-form""#),
            "{chips}"
        );
        assert!(
            chips.contains(r#"<select name="mode" form="search-form""#),
            "{chips}"
        );
        assert!(chips.contains(r#"<option value="mn" selected"#), "{chips}");
    }
    /// The card's two dropdowns, native without JavaScript, defaulted.
    #[tokio::test]
    async fn the_home_card_holds_the_collection_and_mode_dropdowns() {
        let html = render_at("/").await;
        assert!(html.contains(r#"<select name="c""#), "{html}");
        assert!(
            html.contains(r#"<option value="all" selected>All suttas</option>"#),
            "{html}"
        );
        assert!(html.contains(r#"<select name="mode""#), "{html}");
        assert!(
            html.contains(r#"<option value="both" selected>Words + meaning</option>"#),
            "{html}"
        );
        assert!(html.contains(r#"aria-label="Search""#), "{html}");
    }

    /// Three consecutive entries of the pool, from today's index: today's
    /// site's wording, `/?q=` links (the clock is the test's own, as the
    /// page's; the day turning mid-test is the only way to fail).
    #[tokio::test]
    async fn the_home_shows_example_queries_as_links() {
        let html = render_at("/").await;
        let labels = Owner::new().with(|| example_labels(strings().locale()));
        let today = day_index(now_ms(), POOL);
        for offset in 0..3 {
            let label = labels[(today + offset) % POOL];
            let link = format!(r#"href="/?q={}""#, form_encode(label));
            assert!(html.contains(&link), "{label}: {html}");
        }
        assert_eq!(html.matches(r#"href="/?q="#).count(), 3, "{html}");
    }

    #[tokio::test]
    async fn the_home_shows_the_passage_with_the_app_and_social_links() {
        let html = render_at("/").await;
        let today = day_index(now_ms(), VERSES.len());
        let verse = VERSES[today];
        assert!(html.contains(verse.lines[0]), "{html}");
        assert!(
            html.contains(&format!(">{}</span>", verse.citation)),
            "{html}"
        );
        let another = format!(r#"href="/?passage={}""#, (today + 1) % VERSES.len());
        assert!(html.contains(&another), "{html}");
        assert!(html.contains(r#"src="/brand/app-store.svg""#), "{html}");
        assert!(html.contains(r#"src="/brand/instagram.svg""#), "{html}");
        assert!(html.contains(r#"src="/brand/kofi.svg""#), "{html}");
    }

    /// `?passage=` picks the verse and its refresh link points past it,
    /// wrapping at the end; anything else is the verse of the day.
    #[tokio::test]
    async fn passage_in_the_url_picks_the_verse_and_wraps() {
        let html = render_at("/?passage=1").await;
        assert!(!html.contains(VERSES[0].lines[0]), "{html}");
        assert!(html.contains(VERSES[1].lines[0]), "{html}");
        assert!(html.contains(r#"href="/?passage=2""#), "{html}");
        let last = VERSES.len() - 1;
        let html = render_at(&format!("/?passage={last}")).await;
        assert!(html.contains(r#"href="/?passage=0""#), "{html}");
        let html = render_at("/?passage=bogus").await;
        let today = day_index(now_ms(), VERSES.len());
        assert!(html.contains(VERSES[today].lines[0]), "{html}");
    }

    /// No AI in v1: neither the chat icon nor "Explore with AI".
    #[tokio::test]
    async fn the_home_has_no_ai_entry() {
        let html = render_at("/").await;
        assert!(!html.contains("Explore with AI"), "{html}");
        assert!(!html.to_lowercase().contains("chat"), "{html}");
    }

    /// With a query: the results under the card, no home extras.
    #[tokio::test]
    async fn a_query_shows_results_under_the_card_without_the_home_extras() {
        let html = render_at("/?q=mindfulness").await;
        let card = html.find(r#"role="search""#).expect(&html);
        let hit = html.find("MN 10").expect(&html);
        assert!(card < hit, "{html}");
        assert!(!html.contains("how to deal with anger"), "{html}");
        assert!(!html.contains("Defeat anger"), "{html}");
    }

    #[tokio::test]
    async fn a_query_in_the_url_renders_its_hits_with_no_javascript() {
        let html = render_at("/?q=mindfulness").await;
        assert!(html.contains("MN 10"), "{html}");
        assert!(html.contains(r#"value="mindfulness""#), "{html}");
    }

    #[tokio::test]
    async fn mode_in_the_url_narrows_the_hits_and_selects_its_option() {
        let html = render_at("/?q=sutta&mode=words").await;
        assert!(html.contains(r#"<option value="words" selected"#), "{html}");
        assert!(
            html.contains("SN 56.11"),
            "words-only hit should show: {html}"
        );
        assert!(
            !html.contains("DN 22"),
            "dense-only hit should be filtered out: {html}"
        );
    }

    /// The no-result and error states read the app's one language
    /// signal (not a fresh one of their own): switched to French, both
    /// hints are French.
    #[tokio::test]
    async fn the_no_result_and_error_hints_follow_the_app_s_language() {
        use palicanon_ui_lib::Language;
        let html = render_in(Language::Fr, "/?q=qqzzqq-nothing-like-this").await;
        assert!(
            html.contains("Essayez avec moins de mots, le terme anglais"),
            "{html}"
        );
        assert!(!html.contains("Try different words"), "{html}");
        let sentinel = crate::search::mock::ERROR_QUERY;
        let html = render_in(Language::Vi, &format!("/?q={sentinel}")).await;
        assert!(html.contains("Vui lòng thử lại sau một chút."), "{html}");
    }

    #[tokio::test]
    async fn the_sentinel_query_renders_the_error_state() {
        let html = render_at(&format!("/?q={}", crate::search::mock::ERROR_QUERY)).await;
        assert!(html.contains(r#"role="alert""#), "{html}");
    }

    #[tokio::test]
    async fn an_unmatched_query_renders_the_no_result_state() {
        let html = render_at("/?q=qqzzqq-nothing-like-this").await;
        assert!(html.contains("No passage matches"), "{html}");
    }

    /// T1 (review 001): `Mode::parse`'s own unit tests cover the fallback
    /// in isolation; this drives an unknown `?mode=` through the whole
    /// page, the pair the design implies — the chip shown and the hits
    /// returned must agree with each other, not just with `Mode::parse`.
    #[tokio::test]
    async fn an_unknown_mode_falls_back_to_both_end_to_end() {
        let html = render_at("/?q=sutta&mode=bogus").await;
        assert!(html.contains(r#"<option value="both" selected"#), "{html}");
        assert!(
            html.contains("MN 10") && html.contains("DN 22") && html.contains("SN 56.11"),
            "{html}"
        );
    }

    /// T2 (review 001): `maxlength="200"` on the input is a client-side
    /// hint only — a crafted URL can carry a longer `?q=` than that, and
    /// the page must still render something sane, not panic.
    #[tokio::test]
    async fn a_query_over_200_characters_renders_without_panicking() {
        let long_query = "a".repeat(250);
        let html = render_at(&format!("/?q={long_query}")).await;
        assert!(html.contains("No passage matches"), "{html}");
        assert!(html.contains(&long_query), "{html}");
    }

    /// T3 (review 001): the query comes straight from the URL — it must
    /// be escaped everywhere it is echoed back (the input's own `value`
    /// attribute, and the no-result heading's quoted echo), or a crafted
    /// link could inject markup into the first paint.
    #[tokio::test]
    async fn html_special_characters_in_the_query_are_escaped() {
        let raw = "<script>\"&alert(1)</script>";
        let encoded = raw
            .replace('&', "%26")
            .replace('<', "%3C")
            .replace('>', "%3E")
            .replace('"', "%22");
        let html = render_at(&format!("/?q={encoded}")).await;
        assert!(!html.contains("<script>"), "unescaped tag leaked: {html}");
        assert!(html.contains("&lt;script&gt;"), "{html}");
        assert!(html.contains("&quot;") || html.contains("&#34;"), "{html}");
        assert!(html.contains("&amp;"), "{html}");
    }

    #[tokio::test]
    async fn collection_in_the_url_narrows_the_hits_and_selects_its_option() {
        let html = render_at("/?q=sutta&c=mn").await;
        assert!(html.contains(r#"<option value="mn" selected"#), "{html}");
        assert!(html.contains("MN 10"), "{html}");
        assert!(
            !html.contains("DN 22") && !html.contains("SN 56.11"),
            "{html}"
        );
    }

    /// Collection and mode from the URL together: both selected, both
    /// narrowing (SN 56.11 is words-only).
    #[tokio::test]
    async fn collection_and_mode_in_the_url_narrow_together() {
        let html = render_at("/?q=sutta&c=sn&mode=meaning").await;
        assert!(html.contains(r#"<option value="sn" selected"#), "{html}");
        assert!(
            html.contains(r#"<option value="meaning" selected"#),
            "{html}"
        );
        assert!(html.contains("No passage matches"), "{html}");
    }
}
