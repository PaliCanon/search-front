//! A footer page, rendered like today's site's `Pages.tsx`: a header (a
//! back link home, the brand), the pages' nav with the current one in
//! gold, then in a 44rem column the title, the intro, Support's
//! showcases, the sections and the "Last updated" line.
//!
//! Inside the shell, above the same `SiteFooter` as the search page. A
//! phone: the shell's top bar carries the brand (the header's own is
//! hidden below `sm:`, [`BrandLink`]), the nav wraps, a wide table
//! scrolls sideways in its own box.

use leptos::prelude::*;
use leptos_i18n::td_string;
use leptos_meta::{Meta, Title};
use palicanon_ui_lib::Carousel;

use super::rich::{Span, spans};
use super::{
    AppBlock, Href, InstagramBlock, PageKind, PageText, SEARCH_ID, Section, Table, ToolBlock, pages,
};
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
        {page.tools.iter().map(tool_block).collect_view()}
        {page.app.as_ref().map(app_block)}
        {page.instagram.as_ref().map(instagram_block)}
        {page.sections.iter().map(section).collect_view()}
    }
}

/// Support's showcase of one tool (the search, learning Pāli, the map): the
/// heading, the tagline, the button, the screenshots in a carousel named
/// `tool-shots-{id}`. The search tool (`SEARCH_ID`) also carries the
/// lotus and the name, whatever its href; a button to this site stays in
/// the tab; another site's button opens a new tab and ends with ` ↗`, as
/// the App Store's.
fn tool_block(tool: &'static ToolBlock) -> impl IntoView {
    let external = matches!(tool.href, Href::Url(_));
    let button = view! {
        <a
            href=tool.href.as_str()
            target=external.then_some("_blank")
            rel=external.then_some("noopener")
            class=BTN
        >
            {if external { format!("{} ↗", tool.cta) } else { tool.cta.to_owned() }}
        </a>
    };
    view! {
        <section>
            <h2 class=H2>{tool.h}</h2>
            {(tool.id == SEARCH_ID)
                .then(|| {
                    view! {
                        <div class="mt-3 flex items-center gap-3">
                            <img src="/lotus.png" alt="" width="48" height="48" />
                            <p class="font-serif text-lg font-semibold text-heading">"PaliCanon"</p>
                        </div>
                    }
                })}
            <p class=P>{tool.tagline}</p>
            {button}
            {shots(format!("tool-shots-{}", tool.id), tool.h, tool.shots, tool.alts)}
        </section>
    }
}

/// Support's showcase of the Tipiṭaka app: icon, name, tagline, the App
/// Store button, the screenshots.
fn app_block(app: &'static AppBlock) -> impl IntoView {
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
            {shots("app-shots".to_owned(), app.h, &APP_SHOTS, app.alts)}
        </section>
    }
}

/// The width of a screenshot slide from `sm`: four in view (the column's
/// width minus three 0.75 rem gaps, over four), about 167 px in the 704 px
/// column and close to the old 180 px strip. Below `sm` see
/// [`SHOTS_PHONE_LAYOUT`].
const SHOTS_LAYOUT: &str = "sm:w-[calc((100%-2.25rem)/4)]";

/// The width of a screenshot slide below `sm`: two whole slides (the
/// column's width minus one 0.75 rem gap, over two), no peek of the next
/// (user, 2026-10-12: "non pas de bout de la suivante").
const SHOTS_PHONE_LAYOUT: &str = "w-[calc((100%-0.75rem)/2)]";

/// A block's screenshots in a [`Carousel`] named `label`, one slide each;
/// every image carries its size (554 × 1200) so the slides' edges do not
/// move when it loads.
fn shots(
    id: String,
    label: &'static str,
    srcs: &'static [&'static str],
    alts: &'static [&'static str],
) -> impl IntoView {
    let slides = srcs.iter().zip(alts).map(|(src, alt)| {
        view! {
            <img
                src=*src
                alt=*alt
                width="554"
                height="1200"
                loading="lazy"
                class="w-full h-auto rounded-lg"
            />
        }
    });
    view! {
        <div class="mt-4">
            <Carousel id=id label=label slide_layout=SHOTS_LAYOUT phone_layout=SHOTS_PHONE_LAYOUT>
                {slides.collect_view()}
            </Carousel>
        </div>
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
        let shots = html.find(r#"id="app-shots""#).expect(&html);
        let icon = html.find(r#"src="/support/app-icon.jpg""#).expect(&html);
        assert!(icon < shots, "the icon is outside the carousel: {html}");
        assert_eq!(
            html[shots..].matches(r#"src="/support/app-"#).count(),
            5,
            "the five screenshots are in the carousel: {html}"
        );
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

    /// The `<h2>` titles of the Support showcases, in page order, with
    /// the id of the carousel each one holds.
    const SHOWCASES: [(&str, &str); 4] = [
        (">The search</h2>", "tool-shots-search"),
        (">Learn Pāli</h2>", "tool-shots-learn"),
        (">The map</h2>", "tool-shots-map"),
        (">The Tipiṭaka app</h2>", "app-shots"),
    ];

    /// The html of the Support block whose `<h2>` is `SHOWCASES[n]`, up to
    /// the next showcase's heading (Instagram's after the app's).
    fn showcase(html: &str, n: usize) -> &str {
        let from = html.find(SHOWCASES[n].0).expect(html);
        let to = SHOWCASES
            .get(n + 1)
            .map_or(">On Instagram</h2>", |next| next.0);
        &html[from..html.find(to).expect(html)]
    }

    /// The Support page's showcases are four carousels, in order: the
    /// search, learn, the map, the app, then Instagram.
    #[test]
    fn support_has_four_carousels_search_learn_map_then_the_app() {
        let html = render(Language::En, PageKind::Support);
        assert_eq!(
            html.matches(r#"aria-roledescription="carousel""#).count(),
            4,
            "{html}"
        );
        let mut last = 0;
        for (n, (h, id)) in SHOWCASES.iter().enumerate() {
            let heading = html.find(h).expect(&html);
            let carousel = html.find(&format!(r#"id="{id}""#)).expect(&html);
            assert!(last < heading && heading < carousel, "{id}: {html}");
            let part = showcase(&html, n);
            assert!(part.contains(&format!(r#"id="{id}""#)), "{id}: {part}");
            last = carousel;
        }
        let instagram = html.find(">On Instagram</h2>").expect(&html);
        assert!(last < instagram, "Instagram after the app: {html}");
        assert!(
            !html.contains("overflow-x-auto pb-2"),
            "the strip is gone: {html}"
        );
    }

    /// The slides: 5, 5, 4 and 5, one image and one group each.
    #[test]
    fn the_four_carousels_hold_five_five_four_and_five_slides() {
        let html = render(Language::En, PageKind::Support);
        for (n, (prefix, count)) in [("search-", 5), ("learn-", 5), ("map-", 4), ("app-", 5)]
            .into_iter()
            .enumerate()
        {
            let part = showcase(&html, n);
            let src = format!(r#"src="/support/{prefix}"#);
            // The search's lotus and the app's icon are images outside the
            // carousel; the icon's name starts with the app's prefix.
            let outside = usize::from(n == 0 || n == 3);
            assert_eq!(
                part.matches(&src).count(),
                count + usize::from(n == 3),
                "{prefix}: {part}"
            );
            assert_eq!(
                part.matches("<img").count(),
                count + outside,
                "{prefix}: {part}"
            );
            assert_eq!(
                part.matches(r#"role="group""#).count(),
                count,
                "{prefix}: {part}"
            );
        }
        assert_eq!(
            html.matches(r#"src="/support/search-"#).count(),
            5,
            "{html}"
        );
        assert_eq!(html.matches(r#"src="/support/learn-"#).count(), 5, "{html}");
        assert_eq!(html.matches(r#"src="/support/map-"#).count(), 4, "{html}");
    }

    /// Every screenshot of every tool is in its own carousel, once.
    #[test]
    fn every_tool_shot_is_in_its_own_carousel() {
        let html = render(Language::En, PageKind::Support);
        let page = PageKind::Support.text(locale_of(Language::En));
        assert_eq!(page.tools.len(), 3);
        for (n, tool) in page.tools.iter().enumerate() {
            let part = showcase(&html, n);
            for shot in tool.shots {
                let src = format!(r#"src="{shot}""#);
                assert_eq!(html.matches(&src).count(), 1, "{shot}: {html}");
                assert!(part.contains(&src), "{shot} in {}: {part}", tool.id);
            }
        }
    }

    /// All the carousels show four slides from `sm`, not the library's
    /// two: 19 slides (5 + 5 + 4 + 5).
    #[test]
    fn support_slides_show_four_in_view_from_sm() {
        let html = render(Language::En, PageKind::Support);
        assert_eq!(
            html.matches("sm:w-[calc((100%-2.25rem)/4)]").count(),
            19,
            "{html}"
        );
        assert_eq!(
            html.matches("sm:w-[calc((100%-0.75rem)/2)]").count(),
            0,
            "{html}"
        );
    }

    /// On a phone all the carousels show two whole slides and no peek of
    /// the next, not the library's 85%: 19 slides.
    #[test]
    fn support_slides_show_two_whole_on_a_phone() {
        let html = render(Language::En, PageKind::Support);
        assert_eq!(
            html.matches("w-[calc((100%-0.75rem)/2)]").count()
                - html.matches("sm:w-[calc((100%-0.75rem)/2)]").count(),
            19,
            "{html}"
        );
        assert_eq!(html.matches("w-[85%]").count(), 0, "{html}");
    }

    /// Learn's and the map's buttons leave for their own site in a new
    /// tab, marked ` ↗` as the App Store and Instagram ones.
    #[test]
    fn the_learn_and_map_buttons_are_external_links() {
        let html = render(Language::En, PageKind::Support);
        for (n, url, label) in [
            (1, "https://learn.palicanon.app", "Start learning Pāli"),
            (2, "https://map.palicanon.app", "Open the map"),
        ] {
            let block = showcase(&html, n);
            let button = tag_of(block, &format!(">{label} ↗</a>"));
            assert!(button.contains(&format!(r#"href="{url}""#)), "{button}");
            assert!(button.contains(r#"target="_blank""#), "{button}");
            assert!(button.contains(r#"rel="noopener""#), "{button}");
            assert!(
                button.contains("rounded-lg border border-gold/60"),
                "{button}"
            );
            assert!(!block.contains(r#"src="/lotus.png""#), "{block}");
        }
    }

    /// The search's own button stays on this site: the root path, in the
    /// same tab, with no `rel` and no ` ↗` (that mark is for other sites),
    /// in the same button class as the other tools'.
    #[test]
    fn the_search_button_is_a_plain_in_site_link() {
        let html = render(Language::En, PageKind::Support);
        let block = showcase(&html, 0);
        assert!(!block.contains("↗"), "{block}");
        let button = tag_of(block, ">Search the Canon</a>");
        assert!(button.contains(r#"href="/""#), "{button}");
        assert!(!button.contains("target="), "{button}");
        assert!(!button.contains("rel="), "{button}");
        assert!(
            button.contains("rounded-lg border border-gold/60"),
            "{button}"
        );
    }

    /// The Search block's texts and alt texts, per language.
    #[test]
    fn the_search_block_speaks_the_three_languages() {
        for (language, h, tagline, cta, alt) in [
            (
                Language::En,
                "The search",
                "Search the whole Pāli Canon by the words you remember or by what you mean: in English, French and Vietnamese, with the passage and a link to read it.",
                "Search the Canon",
                "The dark theme",
            ),
            (
                Language::Fr,
                "La recherche",
                "Cherchez dans tout le Canon pāli par les mots dont vous vous souvenez ou par ce que vous voulez dire : en anglais, en français et en vietnamien, avec le passage et un lien pour le lire.",
                "Chercher dans le Canon",
                "Le thème sombre",
            ),
            (
                Language::Vi,
                "Tìm kiếm",
                "Tìm trong toàn bộ Kinh tạng Pāli theo những từ bạn nhớ hoặc theo ý bạn muốn nói: bằng tiếng Anh, tiếng Pháp và tiếng Việt, kèm đoạn kinh và đường dẫn để đọc.",
                "Tìm trong Kinh tạng",
                "Giao diện tối",
            ),
        ] {
            let html = render(language, PageKind::Support);
            assert!(html.contains(&format!(">{h}</h2>")), "{language:?}: {html}");
            assert!(html.contains(tagline), "{language:?}: {html}");
            assert!(
                html.contains(&format!(">{cta}</a>")),
                "{language:?}: {html}"
            );
            assert!(html.contains(&format!(r#"alt="{alt}""#)), "{language:?}");
        }
        let html = render(Language::En, PageKind::Support);
        for alt in [
            "The search's home",
            "Results for a question",
            "Results by exact words",
            "A search in French",
        ] {
            assert!(html.contains(&format!(r#"alt="{alt}""#)), "{alt}: {html}");
        }
    }

    /// `shots` zips the screenshots with their alt texts: a language
    /// with fewer alts would drop a slide silently.
    #[test]
    fn every_screenshot_has_its_alt_text_in_every_language() {
        for language in [Language::En, Language::Fr, Language::Vi] {
            let page = PageKind::Support.text(locale_of(language));
            assert_eq!(page.tools.len(), 3, "{language:?}");
            for tool in page.tools {
                assert_eq!(tool.alts.len(), tool.shots.len(), "{language:?}");
            }
            let app = page.app.as_ref().expect("an app block");
            assert_eq!(app.alts.len(), APP_SHOTS.len(), "{language:?}");
        }
    }

    /// The lotus beside the name is decoration: the name says it. It is
    /// the search's alone, not learn's nor the map's.
    #[test]
    fn the_search_blocks_lotus_has_an_empty_alt_and_is_the_only_one() {
        let html = render(Language::En, PageKind::Support);
        let block = showcase(&html, 0);
        let lotus = tag_of(block, r#"src="/lotus.png""#);
        assert!(lotus.contains(r#"alt="""#), "{lotus}");
        assert_eq!(block.matches("PaliCanon</p>").count(), 1, "{block}");
        for n in 1..SHOWCASES.len() {
            assert!(!showcase(&html, n).contains("PaliCanon</p>"), "{n}");
        }
    }

    /// A tool that is not the search but whose button stays on this site
    /// (an in-site route): the lotus and the name belong to the search
    /// tool, not to every `Href::Path`.
    static READER: ToolBlock = ToolBlock {
        id: "reader",
        h: "The reader",
        tagline: "Read a sutta.",
        cta: "Open the reader",
        href: Href::Path("/reader"),
        shots: &[],
        alts: &[],
    };

    /// The lotus row is the search's by id: an in-site tool of another id
    /// renders none, and the search tool still does.
    #[test]
    fn the_lotus_row_is_gated_on_the_search_id_not_on_an_in_site_href() {
        let html = Owner::new().with(|| tool_block(&READER).to_html());
        assert!(html.contains(r#"href="/reader""#), "{html}");
        assert!(!html.contains(r#"src="/lotus.png""#), "{html}");
        assert!(!html.contains("PaliCanon</p>"), "{html}");
        let page = PageKind::Support.text(locale_of(Language::En));
        let search = &page.tools[0];
        assert_eq!(search.id, SEARCH_ID);
        let html = Owner::new().with(|| tool_block(search).to_html());
        assert!(html.contains(r#"src="/lotus.png""#), "{html}");
        assert!(html.contains("PaliCanon</p>"), "{html}");
    }

    /// Learn's and the map's headings, taglines and buttons come from the
    /// page's language, in fr and vi as in en: the French and Vietnamese
    /// texts (alts included) are rendered, not the English ones, and the
    /// external buttons end with ` ↗` and open a new tab.
    #[test]
    fn the_tool_blocks_speak_the_three_languages() {
        let english = PageKind::Support.text(locale_of(Language::En));
        for language in [Language::Fr, Language::Vi] {
            let html = render(language, PageKind::Support);
            let page = PageKind::Support.text(locale_of(language));
            let ids: Vec<&str> = page.tools.iter().map(|tool| tool.id).collect();
            assert_eq!(ids, ["search", "learn", "map"], "{language:?}");
            for tool in &page.tools[1..] {
                let en = english
                    .tools
                    .iter()
                    .find(|t| t.id == tool.id)
                    .expect("the English tool with this id");
                assert_ne!(
                    tool.h, en.h,
                    "{language:?} {}: not the English text",
                    tool.id
                );
                assert_ne!(tool.cta, en.cta, "{language:?} {}", tool.id);
                assert_ne!(
                    tool.alts, en.alts,
                    "{language:?} {}: alts untranslated",
                    tool.id
                );
                assert!(
                    html.contains(&format!(">{}</h2>", tool.h)),
                    "{language:?} {}: {html}",
                    tool.id
                );
                assert!(
                    html.contains(tool.tagline),
                    "{language:?} {}: {html}",
                    tool.id
                );
                assert!(!html.contains(en.tagline), "{language:?} {}", tool.id);
                assert_external_button(&html, tool, language);
            }
        }
    }

    /// The tool's button is in `html`, labelled with ` ↗`, and opens a new
    /// tab with `rel="noopener"`.
    fn assert_external_button(html: &str, tool: &ToolBlock, language: Language) {
        let label = format!(">{} ↗</a>", tool.cta);
        assert!(html.contains(&label), "{language:?} {}: {html}", tool.id);
        let button = tag_of(html, &label);
        for attr in [r#"target="_blank""#, r#"rel="noopener""#] {
            assert!(button.contains(attr), "{language:?} {}: {button}", tool.id);
        }
    }

    /// Only Support renders the showcases: the Terms carry no tool
    /// carousel, no app carousel and no lotus-and-name row.
    #[test]
    fn a_page_other_than_support_renders_no_showcase() {
        for language in [Language::En, Language::Fr, Language::Vi] {
            for kind in [
                PageKind::Terms,
                PageKind::Acknowledgements,
                PageKind::Privacy,
            ] {
                let html = render(language, kind);
                assert!(!html.contains("tool-shots-"), "{kind:?} {language:?}");
                assert!(!html.contains("app-shots"), "{kind:?} {language:?}");
                assert!(!html.contains(">PaliCanon</p>"), "{kind:?} {language:?}");
                assert!(!html.contains(r#"aria-roledescription="carousel""#));
            }
        }
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
