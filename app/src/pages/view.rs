//! A footer page, rendered like today's site's `Pages.tsx`: a header (a
//! back link home, the brand), the pages' nav with the current one in
//! gold, then in a 44rem column the title, the intro, Support's two
//! showcases, the sections and the "Last updated" line.
//!
//! Inside the shell, above the same `SiteFooter` as the search page. A
//! phone: the shell's top bar carries the brand (the header's own is
//! hidden below `sm:`, [`BrandLink`]), the nav wraps, a wide table
//! scrolls sideways in its own box.

use leptos::prelude::*;
use leptos_i18n::td_string;
use leptos_meta::{Meta, Title};

use super::rich::{Span, spans};
use super::{AppBlock, InstagramBlock, PageKind, PageText, Section, Table, pages};
use crate::catalogue::{Strings, strings};
use crate::components::home::BrandLink;

const H2: &str = "mt-10 font-serif text-xl font-semibold text-heading";
const P: &str = "mt-3 max-w-[66ch] text-[15px] leading-7 text-ink";
const UL: &str = "mt-3 max-w-[66ch] list-disc space-y-1.5 pl-5 text-[15px] leading-7 text-ink";
const A: &str = "text-gold underline underline-offset-4 hover:text-gold-hover";
const TH: &str = "py-2 pr-4 text-left text-xs font-semibold uppercase tracking-wide text-subtle";
const TD: &str = "py-2 pr-4 align-top text-[14px] leading-6 text-ink break-words";
const BTN: &str = "mt-4 inline-flex items-center gap-2 rounded-lg border border-gold/60 px-4 py-2 \
    text-sm font-semibold text-gold transition hover:bg-gold hover:text-on-gold";

/// The App Store link from Support, tagged with its own campaign (today's
/// site's `appStoreUrl("PaliWebSupport")`).
const APP_STORE: &str =
    "https://apps.apple.com/app/apple-store/id6760995639?pt=128678815&ct=PaliWebSupport&mt=8";
/// The `@daily_sutta` account.
const INSTAGRAM: &str = "https://www.instagram.com/daily_sutta";
/// The app's screenshots, in the order of [`AppBlock::alts`].
const APP_SHOTS: [&str; 5] = [
    "/support/app-main.jpg",
    "/support/app-reader.jpg",
    "/support/app-search.jpg",
    "/support/app-concept-graph.jpg",
    "/support/app-notes.jpg",
];

/// The footer page `kind`, in the app's language.
#[component]
pub(crate) fn FooterPage(kind: PageKind) -> impl IntoView {
    let t = strings();
    move || {
        let locale = t.locale();
        let page = kind.text(locale);
        view! {
            <Title text=format!("{} — PaliCanon", page.title) />
            <Meta name="description" content=page.description />
            <div class="mx-auto w-full max-w-[64rem] px-6 pb-20">
                <Header t=t />
                <PagesNav t=t current=kind />
                <main class="mx-auto max-w-[44rem] pt-8">
                    <Body page=page />
                    <p class="mt-10 text-xs text-subtle">{pages(locale).updated}</p>
                </main>
            </div>
        }
    }
}

/// A back link home (an arrow, named) and the brand.
#[component]
fn Header(t: Strings) -> impl IntoView {
    view! {
        <header class="flex items-center gap-3 py-5">
            <a
                href="/"
                aria-label=move || td_string!(t.locale(), pages_home)
                class="grid size-9 place-items-center rounded-full text-muted transition hover:bg-chip hover:text-heading"
            >
                <svg
                    width="18"
                    height="18"
                    viewBox="0 0 16 16"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    aria-hidden="true"
                >
                    <path d="M13 8H3M7 4 3 8l4 4" />
                </svg>
            </a>
            <BrandLink />
        </header>
    }
}

/// The four pages, the current one gold (and `aria-current`).
#[component]
fn PagesNav(t: Strings, current: PageKind) -> impl IntoView {
    let links = move || {
        PageKind::ALL
            .into_iter()
            .map(|kind| {
                let here = kind == current;
                let class = if here {
                    "text-gold"
                } else {
                    "text-muted transition hover:text-heading"
                };
                view! {
                    <a href=kind.path() aria-current=here.then_some("page") class=class>
                        {kind.label(t.locale())}
                    </a>
                }
            })
            .collect_view()
    };
    view! {
        <nav
            aria-label=move || td_string!(t.locale(), pages_nav)
            class="mx-auto flex max-w-[44rem] flex-wrap gap-x-8 gap-y-2 border-b border-line pt-4 pb-3 text-[15px]"
        >
            {links}
        </nav>
    }
}

/// The title, the intro, Support's showcases, then the sections.
#[component]
fn Body(page: &'static PageText) -> impl IntoView {
    view! {
        <h1 class="font-serif text-3xl font-semibold text-heading">{page.title}</h1>
        {page.intro.iter().map(|p| view! { <p class=P>{rich(p)}</p> }).collect_view()}
        {page.app.as_ref().map(app_block)}
        {page.instagram.as_ref().map(instagram_block)}
        {page.sections.iter().map(section).collect_view()}
    }
}

/// Support's showcase of the Tipiṭaka app: icon, name, tagline, the App
/// Store button, the screenshots.
fn app_block(app: &'static AppBlock) -> impl IntoView {
    let shots = APP_SHOTS.iter().zip(app.alts).map(|(src, alt)| {
        view! {
            <img
                src=*src
                alt=*alt
                loading="lazy"
                class="h-auto w-[150px] shrink-0 rounded-lg sm:w-[180px]"
            />
        }
    });
    view! {
        <section>
            <h2 class=H2>{app.h}</h2>
            <div class="mt-3 flex items-center gap-3">
                <img src="/support/app-icon.jpg" alt="" width="48" height="48" class="rounded-xl" />
                <p class="font-serif text-lg font-semibold text-heading">"Tipiṭaka"</p>
            </div>
            <p class=P>{app.tagline}</p>
            <a href=APP_STORE target="_blank" rel="noopener" class=BTN>
                {format!("{} ↗", app.cta)}
            </a>
            <div class="mt-4 flex gap-3 overflow-x-auto pb-2">{shots.collect_view()}</div>
        </section>
    }
}

/// Support's showcase of `@daily_sutta`.
fn instagram_block(instagram: &'static InstagramBlock) -> impl IntoView {
    view! {
        <section>
            <h2 class=H2>{instagram.h}</h2>
            <div class="mt-3 flex items-center gap-3">
                <img src="/brand/instagram.svg" alt="" width="28" height="28" />
                <p class="font-serif text-lg font-semibold text-heading">"@daily_sutta"</p>
            </div>
            <p class=P>{instagram.tagline}</p>
            <a href=INSTAGRAM target="_blank" rel="noopener" class=BTN>
                {format!("{} ↗", instagram.cta)}
            </a>
        </section>
    }
}

/// One section: its heading, then its table, list, paragraphs, button.
fn section(s: &'static Section) -> impl IntoView {
    view! {
        <section>
            <h2 class=H2>{s.h}</h2>
            {s.table.as_ref().map(table)}
            {(!s.list.is_empty())
                .then(|| {
                    view! {
                        <ul class=UL>
                            {s.list.iter().map(|li| view! { <li>{rich(li)}</li> }).collect_view()}
                        </ul>
                    }
                })}
            {s.p.iter().map(|p| view! { <p class=P>{rich(p)}</p> }).collect_view()}
            {s
                .cta
                .as_ref()
                .map(|cta| {
                    view! {
                        <a href=cta.href target="_blank" rel="noopener" class=BTN>
                            {format!("{} ↗", cta.label)}
                        </a>
                    }
                })}
        </section>
    }
}

/// A section's table, scrolling sideways in its own box on a phone.
fn table(t: &'static Table) -> impl IntoView {
    let rows = t.rows.iter().map(|row| {
        view! { <tr>{row.iter().map(|c| view! { <td class=TD>{rich(c)}</td> }).collect_view()}</tr> }
    });
    view! {
        <div class="mt-3 overflow-x-auto">
            <table class="w-full max-w-[66ch] border-collapse">
                <thead>
                    <tr class="border-b border-line">
                        {t.head.iter().map(|h| view! { <th class=TH>{*h}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody class="divide-y divide-line">{rows.collect_view()}</tbody>
            </table>
        </div>
    }
}

/// A rich text as markup: bold in the heading colour, a link on this site
/// in the same tab, any other in a new one.
fn rich(text: &'static str) -> impl IntoView {
    spans(text)
        .into_iter()
        .map(|span| match span {
            Span::Text(text) => text.into_any(),
            Span::Bold(text) => view! { <strong class="text-heading">{text}</strong> }.into_any(),
            Span::Link { label, href } if href.starts_with('/') => view! {
                <a href=href class=A>
                    {label}
                </a>
            }
            .into_any(),
            Span::Link { label, href } => view! {
                <a href=href target="_blank" rel="noopener" class=A>
                    {label}
                </a>
            }
            .into_any(),
        })
        .collect_view()
}

#[cfg(test)]
mod tests {
    use palicanon_ui_lib::{Language, provide_language};

    use super::*;

    fn render(language: Language, kind: PageKind) -> String {
        Owner::new().with(|| {
            provide_language().set(language);
            leptos_meta::provide_meta_context();
            view! { <FooterPage kind=kind /> }.to_html()
        })
    }

    /// The opening tag of the element holding `needle`.
    fn tag_of<'a>(html: &'a str, needle: &str) -> &'a str {
        let at = html.find(needle).expect(html);
        let start = html[..at].rfind('<').expect(html);
        let end = at + html[at..].find('>').expect(html);
        &html[start..=end]
    }

    #[test]
    fn every_page_shows_its_title_in_every_language() {
        for (language, kind, title) in [
            (Language::En, PageKind::Acknowledgements, "Acknowledgements"),
            (Language::Fr, PageKind::Terms, "Conditions d’utilisation"),
            (Language::Vi, PageKind::Privacy, "Quyền riêng tư"),
            (Language::Fr, PageKind::Support, "Soutenir"),
        ] {
            let html = render(language, kind);
            assert!(html.contains(&format!(">{title}</h1>")), "{html}");
        }
    }

    /// The pages' nav: the four pages, in the footer's order; the current
    /// one gold and `aria-current`, the others muted — exactly one gold.
    #[test]
    fn the_nav_lists_the_four_pages_with_only_the_current_one_gold() {
        let html = render(Language::En, PageKind::Privacy);
        let nav = &html[html.find("<nav").expect(&html)..html.find("</nav>").expect(&html)];
        let mut last = 0;
        for kind in PageKind::ALL {
            let at = nav.find(&format!(r#"href="{}""#, kind.path())).expect(nav);
            assert!(at > last, "in the footer's order: {nav}");
            last = at;
        }
        assert!(!nav.contains("/legal"), "{nav}");
        let current = tag_of(nav, r#"href="/privacy""#);
        assert!(current.contains("text-gold"), "{current}");
        assert!(current.contains(r#"aria-current="page""#), "{current}");
        assert_eq!(nav.matches("text-gold").count(), 1, "{nav}");
        assert_eq!(nav.matches("aria-current").count(), 1, "{nav}");
        assert!(
            tag_of(nav, r#"href="/terms""#).contains("text-muted"),
            "{nav}"
        );
    }

    /// The header: a back link home, named, and the brand link.
    #[test]
    fn the_header_has_a_back_link_home_and_the_brand() {
        let html = render(Language::Fr, PageKind::Terms);
        assert!(html.contains(r#"aria-label="Accueil""#), "{html}");
        assert!(html.contains(r#"src="/lotus.png""#), "{html}");
        let h1 = html.find("<h1").expect(&html);
        let home_links = html[..h1].matches(r#"href="/""#).count();
        assert_eq!(home_links, 2, "back and brand, before the title: {html}");
    }

    #[test]
    fn the_privacy_page_has_its_table() {
        let html = render(Language::En, PageKind::Privacy);
        assert!(html.contains("<table"), "{html}");
        assert!(html.contains("<thead>"), "{html}");
        let rows = PageKind::Privacy
            .text(crate::catalogue::Locale::en)
            .sections[0]
            .table
            .as_ref()
            .expect("the privacy table")
            .rows
            .len();
        assert!(html.matches("<td ").count() > rows, "{html}");
    }

    #[test]
    fn the_support_page_has_the_app_store_link_instagram_and_ko_fi() {
        let html = render(Language::En, PageKind::Support);
        assert!(html.contains("ct=PaliWebSupport"), "{html}");
        assert!(html.contains(">Get the iPhone app ↗</a>"), "{html}");
        assert!(html.contains(r#"src="/support/app-icon.jpg""#), "{html}");
        assert_eq!(html.matches(r#"src="/support/app-"#).count(), 6, "{html}");
        assert!(
            html.contains(r#"alt="Taking notes on a passage""#),
            "{html}"
        );
        assert!(
            html.contains(r#"href="https://www.instagram.com/daily_sutta""#),
            "{html}"
        );
        assert!(html.contains(">@daily_sutta</p>"), "{html}");
        assert!(
            html.contains(r#"href="https://ko-fi.com/tipitaka""#),
            "{html}"
        );
        let app = html.find("The Tipiṭaka app").expect(&html);
        let first_section = html.find("What it costs").expect(&html);
        assert!(app < first_section, "showcases before the sections: {html}");
    }

    /// Rich text: bold, an outside link (new tab), a link on this site
    /// (same tab).
    #[test]
    fn rich_text_renders_bold_and_both_kinds_of_links() {
        let html = render(Language::En, PageKind::Acknowledgements);
        assert!(
            html.contains(r#"<strong class="text-heading">Bhikkhu Sujato</strong>"#),
            "{html}"
        );
        let outside = tag_of(&html, r#"href="https://suttacentral.net""#);
        assert!(outside.contains(r#"target="_blank""#), "{outside}");
        assert!(outside.contains(r#"rel="noopener""#), "{outside}");
        let html = render(Language::En, PageKind::Terms);
        let inside = tag_of(
            &html,
            r#"href="/acknowledgements" class="text-gold underline"#,
        );
        assert!(!inside.contains("target"), "{inside}");
    }

    #[test]
    fn every_page_ends_with_its_last_updated_line() {
        let html = render(Language::Vi, PageKind::Acknowledgements);
        assert!(
            html.contains(">Cập nhật lần cuối: 2026-10-01.</p>"),
            "{html}"
        );
    }

    /// The text of a page's `<main>`, without its markup's attributes
    /// mattering: what a reader sees, for word checks.
    fn main_of(html: &str) -> &str {
        let start = html.find("<main").expect(html);
        &html[start..html.find("</main>").expect(html)]
    }

    /// The `<h2>` titles of a rendered page, in order.
    fn h2s(html: &str) -> Vec<&str> {
        html.split("<h2")
            .skip(1)
            .map(|part| {
                let open = part.find('>').expect(part) + 1;
                &part[open..open + part[open..].find("</h2>").expect(part)]
            })
            .collect()
    }

    /// Whether `word` stands in `text` as a word of its own ("AI" in
    /// "the AI key", not in "MAIN").
    fn mentions(text: &str, word: &str) -> bool {
        text.match_indices(word).any(|(at, _)| {
            let before = text[..at].chars().next_back();
            let after = text[at + word.len()..].chars().next();
            !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
        })
    }

    /// The Terms' section titles after task 004, per language: the former
    /// 1, 2, 4, 5, 6, renumbered.
    const TERMS_TITLES: [(Language, [&str; 5]); 3] = [
        (
            Language::En,
            [
                "1. The Service",
                "2. Acceptance",
                "3. Texts and intellectual property",
                "4. Availability and liability",
                "5. Governing law and changes",
            ],
        ),
        (
            Language::Fr,
            [
                "1. Le Service",
                "2. Acceptation",
                "3. Textes et propriété intellectuelle",
                "4. Disponibilité et responsabilité",
                "5. Droit applicable et modifications",
            ],
        ),
        (
            Language::Vi,
            [
                "1. Dịch vụ",
                "2. Chấp nhận",
                "3. Kinh văn và sở hữu trí tuệ",
                "4. Tính sẵn có và trách nhiệm",
                "5. Luật áp dụng và thay đổi",
            ],
        ),
    ];

    /// What no legal page says any more, per language (as a word, in lower
    /// case or capitalised). "provider" is not in the list: the hosting
    /// providers stay.
    const AI_WORDS: [(Language, [&str; 6]); 3] = [
        (
            Language::En,
            [
                "AI",
                "assistant",
                "API key",
                "relay",
                "article 3",
                "conversation",
            ],
        ),
        (
            Language::Fr,
            [
                "IA",
                "assistant",
                "clé API",
                "relais",
                "article 3",
                "conversation",
            ],
        ),
        (
            Language::Vi,
            [
                "AI",
                "trợ lý",
                "khóa API",
                "chuyển tiếp",
                "điều 3",
                "cuộc trò chuyện",
            ],
        ),
    ];

    /// `text` mentions `word`, lower case or capitalised ("Relay, on your…").
    fn mentioned(text: &str, word: &str) -> bool {
        let mut chars = word.chars();
        let first = chars.next().expect("a word");
        let capitalised: String = first.to_uppercase().chain(chars).collect();
        mentions(text, word) || mentions(text, &capitalised)
    }

    fn locale_of(language: Language) -> crate::catalogue::Locale {
        crate::catalogue::Locale::from(language)
    }

    /// Task 004: v1 has no AI; the Terms lose their section 3 and renumber.
    #[test]
    fn the_terms_have_no_ai_section_and_renumber() {
        for (language, titles) in TERMS_TITLES {
            let html = render(language, PageKind::Terms);
            assert_eq!(h2s(&html), titles, "{language:?}");
            let description = PageKind::Terms.text(locale_of(language)).description;
            let text = format!("{} {description}", main_of(&html));
            let words = AI_WORDS
                .iter()
                .find(|(l, _)| *l == language)
                .expect("words")
                .1;
            for word in words {
                assert!(!mentioned(&text, word), "{language:?} {word}: {text}");
            }
            assert!(!mentioned(&text, "provider") && !mentioned(&text, "fournisseur"));
        }
    }

    /// Task 004: the privacy notice has no AI relay row, no AI provider as a
    /// controller, no API key nor conversations in the browser's storage.
    #[test]
    fn the_privacy_page_mentions_no_ai_relay_nor_api_key() {
        for (language, words) in AI_WORDS {
            let html = render(language, PageKind::Privacy);
            for word in words {
                assert!(!mentioned(main_of(&html), word), "{language:?} {word}");
            }
            // The table keeps its two rows: the queries and the hosts' logs.
            let table = PageKind::Privacy.text(locale_of(language)).sections[0]
                .table
                .as_ref()
                .expect("the privacy table");
            assert_eq!(table.rows.len(), 2, "{language:?}");
        }
    }

    /// Task 004: the acknowledgements credit translations, not an
    /// assistant's quotations.
    #[test]
    fn the_acknowledgements_do_not_mention_an_assistant() {
        for (language, word) in [
            (Language::En, "assistant"),
            (Language::Fr, "assistant"),
            (Language::Vi, "trợ lý"),
        ] {
            let html = render(language, PageKind::Acknowledgements);
            assert!(
                !main_of(&html).to_lowercase().contains(word),
                "{language:?}"
            );
        }
    }
}
