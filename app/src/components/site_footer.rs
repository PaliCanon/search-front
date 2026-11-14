//! The page footer: ui-lib's [`Footer`] with the site's links
//! (Acknowledgements · Terms · Privacy · Support, each a page of
//! palicanon.app in the current language, `crate::moved`), as on today's
//! site — no Legal. This deploy's release
//! badge sits here at the footer's left, from the [`ReleaseInfo`] the
//! document provides (`crate::app`); a host that provides none shows
//! `unknown`.

use leptos::prelude::*;
use leptos_i18n::td_string;
use palicanon_ui_lib::{EnvBadge, Footer, FooterLink, language_signal};

use crate::catalogue::{Strings, strings};
use crate::moved::{Page, url};
use crate::release::ReleaseInfo;

/// The footer, pushed to the bottom of a short page.
#[component]
pub fn SiteFooter() -> impl IntoView {
    let t = strings();
    let info = use_context::<ReleaseInfo>().unwrap_or_else(ReleaseInfo::unknown);
    view! {
        <div class="mt-auto">
            {move || {
                let info = info.clone();
                view! {
                    <Footer
                        links=links(t)
                        credits=Vec::new()
                        leading=move || {
                            view! { <EnvBadge env=info.env.clone() sha=info.sha.clone() /> }
                        }
                    />
                }
            }}
        </div>
    }
}

/// The footer pages, in today's order, labelled in `t`'s language and
/// linked to palicanon.app in the same language. Read where `t.locale()` is
/// tracked, so a language switch redraws both.
fn links(t: Strings) -> Vec<FooterLink> {
    let locale = t.locale();
    let language = language_signal().get();
    [
        (
            Page::Acknowledgements,
            td_string!(locale, footer_acknowledgements),
        ),
        (Page::Terms, td_string!(locale, footer_terms)),
        (Page::Privacy, td_string!(locale, footer_privacy)),
        (Page::Support, td_string!(locale, footer_support)),
    ]
    .into_iter()
    .map(|(page, label)| FooterLink {
        label,
        href: url(language, page),
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The footer with `info` provided as the release context.
    fn render_for(info: ReleaseInfo) -> String {
        Owner::new().with(|| {
            provide_context(info);
            view! { <SiteFooter /> }.to_html()
        })
    }

    /// The footer on a `main` deploy.
    fn render() -> String {
        render_for(ReleaseInfo::from_vars(
            Some("main".to_owned()),
            Some("34c489c".to_owned()),
        ))
    }

    /// The footer in `language`.
    fn render_in(language: palicanon_ui_lib::Language) -> String {
        Owner::new().with(|| {
            palicanon_ui_lib::provide_language().set(language);
            view! { <SiteFooter /> }.to_html()
        })
    }

    /// The four pages of today's site, in its order, no Legal, each an
    /// absolute link to palicanon.app (the pages live there), same tab.
    #[test]
    fn the_footer_links_the_four_pages_and_no_legal() {
        let html = render();
        let mut last = 0;
        for (href, label) in [
            (
                "https://palicanon.app/acknowledgements/",
                "Acknowledgements",
            ),
            ("https://palicanon.app/terms/", "Terms"),
            ("https://palicanon.app/privacy/", "Privacy"),
            ("https://palicanon.app/support/", "Support"),
        ] {
            let at = html.find(&format!(r#"<a href="{href}""#)).expect(&html);
            assert!(at > last, "{html}");
            last = at;
            assert!(html.contains(&format!(">{label}</a>")), "{html}");
        }
        assert_eq!(html.matches("<a ").count(), 4, "{html}");
        assert!(!html.contains("target="), "same tab: {html}");
        assert!(
            !html.contains("/legal") && !html.contains("Legal"),
            "{html}"
        );
    }

    /// The labels are today's site's (`footer.nav_*`), in the language, and
    /// each link goes to the page in that language: `/fr/...`, `/vi/...`.
    #[test]
    fn the_footer_labels_follow_the_language() {
        use palicanon_ui_lib::Language;
        for (language, prefix, labels) in [
            (
                Language::Fr,
                "/fr",
                ["Crédits", "Conditions", "Confidentialité", "Soutenir"],
            ),
            (
                Language::Vi,
                "/vi",
                ["Tri ân", "Điều khoản", "Quyền riêng tư", "Ủng hộ"],
            ),
        ] {
            let html = render_in(language);
            for (page, label) in ["acknowledgements", "terms", "privacy", "support"]
                .into_iter()
                .zip(labels)
            {
                let link = format!(r#"<a href="https://palicanon.app{prefix}/{page}/""#);
                let at = html.find(&link).expect(&html);
                let tail = &html[at..];
                let text = &tail[tail.find('>').expect(&html) + 1..];
                assert!(
                    text.starts_with(&format!("{label}</a>")),
                    "{language:?}: {html}"
                );
            }
            assert_eq!(html.matches("<a ").count(), 4, "{html}");
        }
    }

    /// 044 (user, 2026-10-05): the release badge sits at the footer's left,
    /// before the links — first in the phone's column, first grid column
    /// on desktop.
    #[test]
    fn the_footer_carries_the_build_badge_at_its_left() {
        let html = render();
        let badge = html.find(r#"title="main · 34c489c""#).expect(&html);
        let nav = html.find("<nav").expect(&html);
        assert!(badge < nav, "{html}");
        let start = html[..badge].rfind("<div").expect(&html);
        let tag = &html[start..start + html[start..].find('>').expect(&html)];
        assert!(tag.contains("sm:col-start-1"), "{tag}");
    }

    /// A tag's badge is its name alone: no sha, no separator.
    #[test]
    fn a_tags_badge_has_no_sha_and_no_separator() {
        let html = render_for(ReleaseInfo::from_vars(Some("v0.1.8".to_owned()), None));
        assert!(html.contains(r#"title="v0.1.8""#), "{html}");
        assert!(html.contains(">v0.1.8<"), "{html}");
        assert!(!html.contains('·'), "{html}");
    }

    /// With no release provided (a host that did not), the badge says
    /// `unknown`, never `dev`.
    #[test]
    fn without_a_release_the_badge_says_unknown() {
        let html = Owner::new().with(|| view! { <SiteFooter /> }.to_html());
        assert!(html.contains(r#"title="unknown""#), "{html}");
        assert!(!html.contains(r#"title="dev""#), "{html}");
        assert!(!html.contains('·'), "{html}");
    }
}
