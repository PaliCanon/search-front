//! The results: one `<ol>` of hits, each with its per-language title, a
//! relevance bar, a highlighted snippet (a quotation for a passage, a
//! paragraph for a blurb) and the link to the sutta — today's site's
//! `ResultList.tsx`, without its AI entries and timings. Zero hits is the
//! no-result state, [`NoResultState`].

use leptos::prelude::*;
use leptos_i18n::td_string;
use palicanon_ui_lib::{Language, language_signal};

use crate::catalogue::strings;
use crate::components::filters::mode_label;
use crate::highlight::highlight;
use crate::search::{HitDto, SearchResponse, SnippetKind};

/// The hits, best first; the no-result state when there are none. Wrapped
/// in one `aria-live="polite"` region (Δ6, review 001) so a screen reader
/// announces whichever of the two this renders — a fresh `GET` always
/// replaces the whole page, so "polite" (not "assertive") is enough: nothing
/// here updates without a navigation the reader already follows.
#[component]
pub fn ResultList(
    /// The finished search to show.
    response: SearchResponse,
) -> impl IntoView {
    let query = response.query;
    let hits = response.hits;
    let count = hits.len();
    let top = hits.iter().map(|h| h.score).fold(0.0_f32, f32::max);
    view! {
        <section aria-live="polite">
            {response.fell_back.then(|| view! { <FellBackNote /> })}
            {if hits.is_empty() {
                view! { <NoResultState query=query /> }.into_any()
            } else {
                view! {
                    <div class="mt-6">
                        <p class="flex justify-between border-b border-line pb-3 text-sm text-muted tabular-nums">
                            <span>{move || results_count_line(count)}</span>
                        </p>
                        <ol class="divide-y divide-line">
                            {hits
                                .into_iter()
                                .map(|hit| {
                                    view! { <ResultItem hit=hit query=query.clone() top=top /> }
                                })
                                .collect_view()}
                        </ol>
                    </div>
                }
                    .into_any()
            }}
        </section>
    }
}

/// The language asked for had no index: the English texts were searched
/// (today's site's `fell_back`).
#[component]
fn FellBackNote() -> impl IntoView {
    let t = strings();
    view! {
        <p role="status" class="mt-6 text-sm text-muted">
            {move || td_string!(t.locale(), fell_back)}
        </p>
    }
}

/// "`{count}` passages" — "1 passage" for one (Δ7, review 001): no
/// interpolation in the catalogue (every key here is a plain string,
/// `catalogue.rs`), so the count is built in Rust around the one
/// translated word.
fn results_count_line(count: usize) -> String {
    let t = strings();
    let label = if count == 1 {
        td_string!(t.locale(), results_one)
    } else {
        td_string!(t.locale(), results_label)
    };
    format!("{count} {label}")
}

/// One hit: reference and titles with the relevance bar at the right, the
/// highlighted snippet, the link to read it.
#[component]
fn ResultItem(hit: HitDto, query: String, top: f32) -> impl IntoView {
    let t = strings();
    let title = localized_title(&hit, language_signal().get()).to_owned();
    let snippet = snippet_view(&hit, &query);
    view! {
        <li class="py-7">
            <div class="flex items-start justify-between gap-4">
                <p class="flex min-w-0 flex-wrap items-baseline gap-x-3 gap-y-1">
                    <span class="font-mono text-[11px] text-gold">{hit.display_id}</span>
                    {hit
                        .pali_title
                        .clone()
                        .map(|p| {
                            view! {
                                <span lang="pi" class="font-serif text-[17px] text-heading">
                                    {p}
                                </span>
                            }
                        })}
                    <span class="text-[15px] text-muted">{title}</span>
                </p>
                <span class="mt-2 shrink-0">
                    <Relevance score=hit.score top=top found_via=hit.sources.found_via() />
                </span>
            </div>
            {snippet}
            <div class="mt-4 flex flex-wrap items-center justify-between gap-2">
                <a
                    href=hit.url.clone()
                    target="_blank"
                    rel="noopener"
                    class="inline-flex items-center gap-1.5 rounded-full border border-line bg-chip px-3 py-1 text-[13px] text-muted transition hover:border-gold/60 hover:text-heading"
                >
                    {move || td_string!(t.locale(), read_link)}
                    <span aria-hidden="true" class="text-[11px]">
                        "↗"
                    </span>
                </a>
            </div>
        </li>
    }
}

/// The snippet with its matches marked: a passage is quoted (a gold rule
/// at its left, serif), a blurb is a plain paragraph.
fn snippet_view(hit: &HitDto, query: &str) -> AnyView {
    let text = highlight(&hit.snippet, query)
        .into_iter()
        .map(|segment| {
            if segment.matched {
                view! { <mark>{segment.text}</mark> }.into_any()
            } else {
                view! { {segment.text} }.into_any()
            }
        })
        .collect_view();
    match hit.snippet_kind {
        SnippetKind::Passage => view! {
            <figure class="mt-3 max-w-[66ch] border-l-2 border-gold/60 pl-4">
                <blockquote class="font-serif text-[17px] leading-[1.7] break-words text-ink">
                    {text}
                </blockquote>
            </figure>
        }
        .into_any(),
        SnippetKind::Blurb => view! { <p class="mt-3 max-w-[66ch] text-[16px] leading-7 break-words text-muted">{text}</p> }
        .into_any(),
    }
}

/// The relevance bar: the hit's score as a share of the page's best (never
/// under 8 %, so the weakest hit still shows a bar); how it was found
/// (words, meaning, both) is in its tooltip and accessible name.
#[component]
fn Relevance(score: f32, top: f32, found_via: crate::search::Mode) -> impl IntoView {
    let t = strings();
    let share = if top > 0.0 {
        (score / top).max(0.08)
    } else {
        0.0
    };
    let width = format!("width:{:.0}%", share * 100.0);
    let name = move || {
        format!(
            "{} · {}",
            td_string!(t.locale(), relevance_label),
            mode_label(t, found_via)
        )
    };
    view! {
        <span class="flex items-center" title=name aria-label=name>
            <span class="h-1 w-10 overflow-hidden rounded-full bg-line" aria-hidden="true">
                <span class="block h-full rounded-full bg-gold" style=width></span>
            </span>
        </span>
    }
}

/// `hit`'s title in `language` when it has one, else its own (searched)
/// title — English is only ever `hit.title` (the mock's one query
/// language), as today's site also falls back for it.
fn localized_title(hit: &HitDto, language: Language) -> &str {
    let localized = match language {
        Language::En => None,
        Language::Fr => hit.titles.fr.as_deref(),
        Language::Vi => hit.titles.vi.as_deref(),
    };
    localized.unwrap_or(&hit.title)
}

/// No hits matched the query: not an error, just nothing to show.
#[component]
fn NoResultState(query: String) -> impl IntoView {
    let t = strings();
    view! {
        <div class="mt-12 max-w-[60ch] space-y-2">
            <p class="font-serif text-xl text-heading">
                {move || td_string!(t.locale(), no_result_title)} " \u{201c}"{query.clone()}
                "\u{201d}."
            </p>
            <p class="leading-6 text-muted">{move || td_string!(t.locale(), no_result_hint)}</p>
        </div>
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::search::{Sources, Titles};

    fn hit(sources: Sources) -> HitDto {
        HitDto {
            sutta_uid: "mn10".to_owned(),
            display_id: "MN 10".to_owned(),
            title: "Satipatthana Sutta".to_owned(),
            pali_title: Some("Satipaṭṭhānasutta".to_owned()),
            titles: Titles {
                en: None,
                fr: Some("Les fondements".to_owned()),
                vi: None,
            },
            snippet: "A path to liberation through mindfulness.".to_owned(),
            snippet_kind: SnippetKind::Blurb,
            url: "https://suttacentral.net/mn10".to_owned(),
            sources,
            score: 1.0,
        }
    }

    fn render(response: SearchResponse) -> String {
        Owner::new().with(|| view! { <ResultList response=response /> }.to_html())
    }

    fn response(hits: Vec<HitDto>) -> SearchResponse {
        SearchResponse {
            query: "mindfulness".to_owned(),
            mode: crate::search::Mode::Both,
            hits,
            fell_back: false,
        }
    }

    #[test]
    fn zero_hits_is_the_no_result_state_not_an_empty_list() {
        let html = render(SearchResponse {
            query: "qqzzqq".to_owned(),
            mode: crate::search::Mode::Both,
            hits: Vec::new(),
            fell_back: false,
        });
        assert!(html.contains("No passage matches"), "{html}");
        assert!(html.contains("qqzzqq") && html.contains("”.</p>"), "{html}");
        assert!(!html.contains("<ol"), "{html}");
    }

    #[test]
    fn a_hit_shows_its_reference_snippet_and_read_link() {
        let html = render(response(vec![hit(Sources {
            dense: true,
            sparse: true,
        })]));
        assert!(html.contains("MN 10"), "{html}");
        assert!(
            html.contains("<mark>mindfulness</mark>") || html.contains("<mark>Mindfulness</mark>"),
            "{html}"
        );
        assert!(html.contains("suttacentral.net/mn10"), "{html}");
        assert!(html.contains("Read on SuttaCentral"), "{html}");
    }

    /// Task 002: the reference goes one step below the Pāli title and the
    /// title.
    #[test]
    fn the_reference_is_set_one_step_smaller_than_the_titles() {
        let html = render(response(vec![hit(Sources::default())]));
        assert!(html.contains("font-mono text-[11px] text-gold"), "{html}");
        assert!(!html.contains("font-mono text-xs text-gold"), "{html}");
        assert!(!html.contains("font-mono text-sm text-gold"), "{html}");
    }

    /// Today's site: a passage is a quotation with a gold rule, a blurb a
    /// plain paragraph.
    #[test]
    fn a_passage_is_quoted_and_a_blurb_is_a_paragraph() {
        let mut passage = hit(Sources::default());
        passage.snippet_kind = SnippetKind::Passage;
        let html = render(response(vec![passage]));
        assert!(html.contains("<blockquote"), "{html}");
        assert!(html.contains("border-l-2 border-gold/60"), "{html}");
        let html = render(response(vec![hit(Sources::default())]));
        assert!(!html.contains("<blockquote"), "{html}");
        assert!(html.contains("leading-7"), "{html}");
    }

    /// The relevance bar: full for the best hit, proportional for the
    /// others, never under 8 %; how the hit was found is its name.
    #[test]
    fn the_relevance_bar_is_relative_to_the_best_hit() {
        let mut weak = hit(Sources {
            dense: true,
            sparse: false,
        });
        weak.score = 0.01;
        let html = render(response(vec![hit(Sources::default()), weak]));
        assert!(html.contains("width:100%"), "{html}");
        assert!(html.contains("width:8%"), "{html}");
        assert!(
            html.contains("Relevance, relative to the best match · Meaning"),
            "{html}"
        );
    }

    #[test]
    fn the_found_via_tag_names_only_the_branches_that_matched() {
        let html = render(response(vec![hit(Sources {
            dense: true,
            sparse: false,
        })]));
        assert!(html.contains("· Meaning"), "{html}");
        assert!(!html.contains("· Words"), "{html}");
    }

    /// Δ6 (review 001): today's site wraps its results in `aria-live`
    /// so a screen reader announces a changed result set; both the hit
    /// list and the no-result state are "the results" for this purpose.
    #[test]
    fn results_are_announced_through_an_aria_live_region() {
        let html = render(response(vec![hit(Sources::default())]));
        assert!(html.contains(r#"aria-live="polite""#), "{html}");
        let html = render(response(Vec::new()));
        assert!(html.contains(r#"aria-live="polite""#), "{html}");
    }

    /// Δ7 (review 001): the results-count line, "passages" — "passage"
    /// for one, like today's site.
    #[test]
    fn the_results_count_line_shows_how_many_hits() {
        let html = render(response(vec![
            hit(Sources::default()),
            hit(Sources::default()),
        ]));
        assert!(html.contains("2 passages"), "{html}");
        let html = render(response(vec![hit(Sources::default())]));
        assert!(html.contains("1 passage<"), "{html}");
    }

    /// T5 (review 001): only `highlight.rs`'s own unit tests covered the
    /// diacritic folding in isolation; this drives it end to end through
    /// `ResultItem`'s rendering of a diacritic snippet.
    #[test]
    fn t5_a_diacritic_snippet_highlights_against_a_plain_query_through_result_item() {
        let mut diacritic_hit = hit(Sources::default());
        diacritic_hit.snippet = "A teaching on satipaṭṭhāna, mindfulness itself.".to_owned();
        let html = render(SearchResponse {
            query: "satipatthana".to_owned(),
            mode: crate::search::Mode::Both,
            hits: vec![diacritic_hit],
            fell_back: false,
        });
        assert!(html.contains("<mark>satipaṭṭhāna</mark>"), "{html}");
    }

    fn render_in(language: palicanon_ui_lib::Language, response: SearchResponse) -> String {
        Owner::new().with(|| {
            palicanon_ui_lib::provide_language().set(language);
            view! { <ResultList response=response /> }.to_html()
        })
    }

    /// The English-fallback note (today's site's `fell_back`): shown above
    /// the hits only when the response says so, as a status, in the app's
    /// language.
    #[test]
    fn the_fell_back_note_shows_only_when_set_in_every_language() {
        use palicanon_ui_lib::Language;
        for (language, note) in [
            (
                Language::En,
                "Searching the English texts: this language’s index isn’t available yet.",
            ),
            (
                Language::Fr,
                "Recherche dans les textes anglais : l’index en français n’est pas encore disponible.",
            ),
            (
                Language::Vi,
                "Đang tìm trong bản tiếng Anh: chỉ mục tiếng Việt chưa sẵn sàng.",
            ),
        ] {
            let mut fell_back = response(vec![hit(Sources::default())]);
            fell_back.fell_back = true;
            let html = render_in(language, fell_back);
            let status = html.find(r#"role="status""#).expect(&html);
            let first_hit = html.find("MN 10").expect(&html);
            assert!(html.contains(note), "{language:?}: {html}");
            assert!(status < first_hit, "above the hits: {html}");
            let html = render_in(language, response(vec![hit(Sources::default())]));
            assert!(!html.contains(note), "{language:?}: {html}");
            assert!(!html.contains(r#"role="status""#), "{language:?}: {html}");
        }
    }

    /// The note also reads above the no-result state: the English index
    /// was searched whatever it found.
    #[test]
    fn the_fell_back_note_shows_above_the_no_result_state_too() {
        let mut empty = response(Vec::new());
        empty.fell_back = true;
        let html = render(empty);
        assert!(html.contains(r#"role="status""#), "{html}");
        assert!(html.contains("No passage matches"), "{html}");
    }
}
