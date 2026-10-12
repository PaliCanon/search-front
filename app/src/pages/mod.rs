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
    /// Support only: the search showcase, before the app's.
    pub search: Option<SearchBlock>,
    /// Support only: the app showcase, before the sections.
    pub app: Option<AppBlock>,
    /// Support only: the `@daily_sutta` showcase, before the sections.
    pub instagram: Option<InstagramBlock>,
    /// The page's sections, in order.
    pub sections: &'static [Section],
}

/// The search's showcase (Support).
#[derive(Debug)]
pub(crate) struct SearchBlock {
    /// Its heading, also the carousel's name.
    pub h: &'static str,
    /// One line on the search.
    pub tagline: &'static str,
    /// The button's label, linking to the search.
    pub cta: &'static str,
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

    /// Only Support carries the app and Instagram showcases.
    #[test]
    fn only_support_has_the_showcases() {
        for locale in LOCALES {
            for kind in PageKind::ALL {
                let page = kind.text(locale);
                let support = kind == PageKind::Support;
                assert_eq!(page.app.is_some(), support, "{kind:?}");
                assert_eq!(page.search.is_some(), support, "{kind:?}");
                assert_eq!(page.instagram.is_some(), support, "{kind:?}");
            }
        }
    }
}
