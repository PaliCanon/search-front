//! The footer pages — Acknowledgements, Terms, Privacy, Support — copied
//! from today's site (`web/src/locales/<lang>.json` `pages.*`, rendered
//! like its `web/src/components/Pages.tsx`). No Legal page.
//!
//! The texts are typed Rust data, one module per language
//! (`content_<lang>.rs`, generated from the old site's JSON), not this
//! service's `leptos_i18n` catalogue: that catalogue reads every JSON
//! array as a plural range, so it cannot hold a page's lists of
//! paragraphs, sections and table rows.

mod content_en;
mod content_fr;
mod content_vi;
mod rich;
mod view;

pub(crate) use view::FooterPage;

use leptos_i18n::td_string;

use crate::catalogue::Locale;

/// One footer page, in the order the footer and the pages' own nav list
/// them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PageKind {
    /// Who made the texts PaliCanon searches, and their licences.
    Acknowledgements,
    /// The terms of use.
    Terms,
    /// The privacy notice.
    Privacy,
    /// The costs, and how to help.
    Support,
}

impl PageKind {
    /// Every page, in the footer's order.
    pub(crate) const ALL: [Self; 4] = [
        Self::Acknowledgements,
        Self::Terms,
        Self::Privacy,
        Self::Support,
    ];

    /// The page's route.
    pub(crate) const fn path(self) -> &'static str {
        match self {
            Self::Acknowledgements => "/acknowledgements",
            Self::Terms => "/terms",
            Self::Privacy => "/privacy",
            Self::Support => "/support",
        }
    }

    /// The page's link label in the footer and the pages' nav.
    pub(crate) fn label(self, locale: Locale) -> &'static str {
        match self {
            Self::Acknowledgements => td_string!(locale, footer_acknowledgements),
            Self::Terms => td_string!(locale, footer_terms),
            Self::Privacy => td_string!(locale, footer_privacy),
            Self::Support => td_string!(locale, footer_support),
        }
    }

    /// The page's text in `locale`.
    pub(crate) fn text(self, locale: Locale) -> &'static PageText {
        let pages = pages(locale);
        match self {
            Self::Acknowledgements => &pages.acknowledgements,
            Self::Terms => &pages.terms,
            Self::Privacy => &pages.privacy,
            Self::Support => &pages.support,
        }
    }
}

/// Every page in `locale`.
pub(crate) fn pages(locale: Locale) -> &'static Pages {
    match locale {
        Locale::en => &content_en::PAGES,
        Locale::fr => &content_fr::PAGES,
        Locale::vi => &content_vi::PAGES,
    }
}

/// Every footer page in one language, and the "Last updated" line.
#[derive(Debug)]
pub(crate) struct Pages {
    /// Acknowledgements.
    pub acknowledgements: PageText,
    /// Terms of use.
    pub terms: PageText,
    /// Privacy.
    pub privacy: PageText,
    /// Support.
    pub support: PageText,
    /// The line closing every page, its date resolved.
    pub updated: &'static str,
}

/// One page's text. Paragraphs, list items and table cells are rich
/// text: `**bold**` and `[label](href)` (see `rich`).
#[derive(Debug)]
pub(crate) struct PageText {
    /// The `<h1>`, and the document title before " — PaliCanon".
    pub title: &'static str,
    /// The page's meta description.
    pub description: &'static str,
    /// The paragraphs under the title.
    pub intro: &'static [&'static str],
    /// Support only: the tools to discover the Canon (search, learn,
    /// map), in order, each a showcase before the app's. Empty elsewhere.
    pub tools: &'static [ToolBlock],
    /// Support only: the app showcase, before the sections.
    pub app: Option<AppBlock>,
    /// Support only: the `@daily_sutta` showcase, before the sections.
    pub instagram: Option<InstagramBlock>,
    /// The page's sections, in order.
    pub sections: &'static [Section],
}

/// Where a tool's button goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Href {
    /// A route of this site, opened in the same tab.
    Path(&'static str),
    /// Another site, opened in a new tab.
    Url(&'static str),
}

impl Href {
    /// The link's `href`.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Path(path) => path,
            Self::Url(url) => url,
        }
    }
}

/// The search tool's id: the brand row is its own (view.rs), the tests name
/// it by this.
pub(crate) const SEARCH_ID: &str = "search";

/// One tool to discover the Canon (Support): the search, learning Pāli,
/// the map. The same shape for each, rendered in the order given.
#[derive(Debug)]
pub(crate) struct ToolBlock {
    /// The tool's handle: the carousel's id (`tool-shots-{id}`) and the
    /// switch of the brand row, shown on the search tool only (view.rs).
    pub id: &'static str,
    /// Its heading, also the carousel's name.
    pub h: &'static str,
    /// One line on the tool.
    pub tagline: &'static str,
    /// The button's label.
    pub cta: &'static str,
    /// Where the button goes.
    pub href: Href,
    /// The screenshots under `app/public`, as served (`/support/…`).
    pub shots: &'static [&'static str],
    /// The screenshots' alt texts, in the screenshots' order.
    pub alts: &'static [&'static str],
}

/// The Tipiṭaka app's showcase (Support).
#[derive(Debug)]
pub(crate) struct AppBlock {
    /// Its heading.
    pub h: &'static str,
    /// One line on the app.
    pub tagline: &'static str,
    /// The App Store button's label.
    pub cta: &'static str,
    /// The screenshots' alt texts, in the screenshots' order.
    pub alts: &'static [&'static str],
}

/// The `@daily_sutta` showcase (Support).
#[derive(Debug)]
pub(crate) struct InstagramBlock {
    /// Its heading.
    pub h: &'static str,
    /// One line on the account.
    pub tagline: &'static str,
    /// The Instagram button's label.
    pub cta: &'static str,
}

/// One section: a heading, then (in this order, each optional) a table,
/// a list, paragraphs and a button-styled link.
#[derive(Debug)]
pub(crate) struct Section {
    /// The `<h2>`.
    pub h: &'static str,
    /// Paragraphs.
    pub p: &'static [&'static str],
    /// Bullet items.
    pub list: &'static [&'static str],
    /// A table.
    pub table: Option<Table>,
    /// A prominent outside link (Support's Ko-fi).
    pub cta: Option<Cta>,
}

/// A table: its header cells, then its rows.
#[derive(Debug)]
pub(crate) struct Table {
    /// The header cells.
    pub head: &'static [&'static str],
    /// The rows, each a list of cells.
    pub rows: &'static [&'static [&'static str]],
}

/// A button-styled outside link.
#[derive(Debug)]
pub(crate) struct Cta {
    /// What it says.
    pub label: &'static str,
    /// Where it goes.
    pub href: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCALES: [Locale; 3] = [Locale::en, Locale::fr, Locale::vi];

    /// Every string of a page, for whole-text checks.
    fn all_text(page: &PageText) -> Vec<&'static str> {
        let mut out = vec![page.title, page.description];
        out.extend(page.intro);
        for s in page.sections {
            out.push(s.h);
            out.extend(s.p);
            out.extend(s.list);
            if let Some(t) = &s.table {
                out.extend(t.head);
                out.extend(t.rows.iter().flat_map(|r| r.iter()));
            }
        }
        out
    }

    /// Round 4: no Legal page, so no text points to one, in any language,
    /// and no placeholder is left unresolved.
    #[test]
    fn no_page_points_to_legal_or_keeps_a_placeholder() {
        for locale in LOCALES {
            for kind in PageKind::ALL {
                for text in all_text(kind.text(locale)) {
                    assert!(!text.contains("/legal"), "{kind:?} {locale:?}: {text}");
                    assert!(!text.contains("{{"), "{kind:?} {locale:?}: {text}");
                }
            }
        }
    }

    /// The old site's `{{publisher}}` and `{{contact}}`, resolved.
    #[test]
    fn the_publisher_and_contact_are_resolved() {
        for locale in LOCALES {
            let intro = PageKind::Privacy.text(locale).intro.join(" ");
            assert!(intro.contains("Thibault Rivière"), "{intro}");
            assert!(intro.contains("riviere.thibault@icloud.com"), "{intro}");
        }
    }

    /// Support's three tools; a loop over an empty list would pass
    /// vacuously, so the count is asserted first.
    fn support_tools(locale: Locale) -> &'static [ToolBlock] {
        let tools = PageKind::Support.text(locale).tools;
        assert_eq!(tools.len(), 3, "{locale:?}");
        tools
    }

    /// Only Support carries the showcases: the app's, Instagram's, and
    /// the three tools (search, learn, map).
    #[test]
    fn only_support_has_the_showcases() {
        for locale in LOCALES {
            for kind in PageKind::ALL {
                let page = kind.text(locale);
                let support = kind == PageKind::Support;
                assert_eq!(page.app.is_some(), support, "{kind:?}");
                assert_eq!(page.instagram.is_some(), support, "{kind:?}");
                assert_eq!(page.tools.len(), if support { 3 } else { 0 }, "{kind:?}");
            }
        }
    }

    /// The tools in order: search (this site), learn, map (other sites).
    #[test]
    fn the_tools_come_in_order_with_their_hrefs() {
        for locale in LOCALES {
            let hrefs: Vec<Href> = PageKind::Support
                .text(locale)
                .tools
                .iter()
                .map(|tool| tool.href)
                .collect();
            assert_eq!(
                hrefs,
                [
                    Href::Path("/"),
                    Href::Url("https://learn.palicanon.app"),
                    Href::Url("https://map.palicanon.app"),
                ],
                "{locale:?}"
            );
        }
    }

    /// The ids name the carousels (`tool-shots-{id}`): the same in every
    /// language, unique, lowercase ascii.
    #[test]
    fn the_tool_ids_are_search_learn_map_in_every_language() {
        for locale in LOCALES {
            let ids: Vec<&str> = support_tools(locale).iter().map(|tool| tool.id).collect();
            assert_eq!(ids, ["search", "learn", "map"], "{locale:?}");
            let mut sorted = ids.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), ids.len(), "{locale:?} ids are unique");
            assert!(
                ids.iter()
                    .all(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_lowercase())),
                "{locale:?} ids are lowercase ascii"
            );
        }
    }

    /// The view zips the screenshots with their alt texts: a tool with
    /// fewer alts than shots (or the reverse) would drop a slide silently.
    /// Not caught: two alts swapped between slides.
    #[test]
    fn every_tool_shot_has_its_alt_in_every_language() {
        for locale in LOCALES {
            for tool in support_tools(locale) {
                assert!(!tool.shots.is_empty(), "{locale:?} {}: no slide", tool.h);
                assert_eq!(tool.shots.len(), tool.alts.len(), "{locale:?} {}", tool.h);
                assert!(tool.shots.len() >= 4, "{locale:?} {}", tool.h);
            }
        }
    }

    /// Two slides of one tool never share an alt text, in any language:
    /// a copy-pasted alt would pass the length check above.
    #[test]
    fn the_alts_of_a_tool_are_pairwise_distinct() {
        for locale in LOCALES {
            for tool in support_tools(locale) {
                let mut sorted: Vec<&str> = tool.alts.to_vec();
                sorted.sort_unstable();
                sorted.dedup();
                assert_eq!(sorted.len(), tool.alts.len(), "{locale:?} {}", tool.id);
            }
        }
    }

    /// Every tool's shots (search, learn, map) are paired with their alts
    /// by keyword on the file's stem (English): two alts swapped between
    /// slides fail here.
    #[test]
    fn every_tools_shots_are_paired_with_their_alts() {
        let tools = support_tools(Locale::en);
        assert_shots_paired(
            &tools[0],
            SEARCH_ID,
            &[
                ("search-home", "home"),
                ("search-results", "question"),
                ("search-words", "exact"),
                ("search-french", "french"),
                ("search-dark", "dark"),
            ],
        );
        assert_shots_paired(
            &tools[1],
            "learn",
            &[
                ("learn-1", "home"),
                ("learn-2", "new terms"),
                ("learn-3", "meaning"),
                ("learn-4", "answer"),
                ("learn-5", "word to find"),
            ],
        );
        assert_shots_paired(
            &tools[2],
            "map",
            &[
                ("map-parallels", "parallel"),
                ("map-meaning", "meaning"),
                ("map-cluster", "neighbour"),
                ("map-phone", "collections"),
            ],
        );
    }

    /// The tool `id`'s shots and alts, in order, match `(stem, keyword)`.
    fn assert_shots_paired(tool: &ToolBlock, id: &str, pairs: &[(&str, &str)]) {
        assert_eq!(tool.id, id);
        assert_eq!(tool.shots.len(), pairs.len(), "{id}");
        assert_eq!(tool.alts.len(), pairs.len(), "{id}");
        for ((shot, alt), (stem, keyword)) in tool.shots.iter().zip(tool.alts).zip(pairs) {
            assert!(shot.contains(stem), "{shot} is not {stem}");
            assert!(
                alt.to_lowercase().contains(keyword),
                "{shot}: alt {alt:?} lacks {keyword:?}"
            );
        }
    }

    /// A button goes to this site's root path or to an https site, and
    /// every string a tool carries is filled.
    #[test]
    fn every_tool_href_is_a_root_path_or_https_and_its_text_is_filled() {
        for locale in LOCALES {
            for tool in support_tools(locale) {
                match tool.href {
                    Href::Path(path) => assert!(path.starts_with('/'), "{path}"),
                    Href::Url(url) => assert!(url.starts_with("https://"), "{url}"),
                }
                assert_ne!(tool.href.as_str(), "");
                for text in [tool.h, tool.tagline, tool.cta]
                    .into_iter()
                    .chain(tool.alts.iter().copied())
                {
                    assert!(!text.trim().is_empty(), "{locale:?} {}", tool.h);
                    assert!(!text.contains('\u{2014}'), "no em dash: {text}");
                }
            }
        }
    }

    /// The width and height of a JPEG, read from its first start-of-frame
    /// marker; the file must also end with the end-of-image marker.
    fn jpeg_size(bytes: &[u8]) -> (u32, u32) {
        assert_eq!(bytes[..2], [0xFF, 0xD8], "not a JPEG");
        assert_eq!(
            bytes[bytes.len() - 2..],
            [0xFF, 0xD9],
            "no end-of-image marker"
        );
        let mut at = 2;
        while at + 9 < bytes.len() {
            assert_eq!(bytes[at], 0xFF, "marker expected at {at}");
            let marker = bytes[at + 1];
            let len = usize::from(u16::from_be_bytes([bytes[at + 2], bytes[at + 3]]));
            if (0xC0..=0xCF).contains(&marker) && ![0xC4, 0xC8, 0xCC].contains(&marker) {
                let h = u32::from(u16::from_be_bytes([bytes[at + 5], bytes[at + 6]]));
                let w = u32::from(u16::from_be_bytes([bytes[at + 7], bytes[at + 8]]));
                return (w, h);
            }
            at += 2 + len;
        }
        panic!("no start-of-frame marker");
    }

    /// A JPEG cut short still has its start-of-frame marker: the size
    /// helper must refuse it for the missing end-of-image marker.
    #[test]
    #[should_panic(expected = "end-of-image")]
    fn jpeg_size_refuses_a_truncated_file() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("public/support/map-phone.jpg");
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        jpeg_size(&bytes[..bytes.len() - 100]);
    }

    /// Every screenshot a tool names is served from `app/public` and is
    /// 554 x 1200, the size every slide of the carousel reserves (the
    /// view's `width`/`height` attributes). Not caught: a screenshot of
    /// the right size that shows the wrong screen.
    #[test]
    fn every_tool_shot_exists_at_554_by_1200() {
        let public = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("public");
        for locale in LOCALES {
            for tool in support_tools(locale) {
                for shot in tool.shots {
                    let path = public.join(shot.trim_start_matches('/'));
                    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
                    assert_eq!(jpeg_size(&bytes), (554, 1200), "{path:?}");
                }
            }
        }
    }
}
