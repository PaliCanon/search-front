//! The page footer: ui-lib's [`Footer`] with the site's links
//! (Acknowledgements · Terms · Privacy · Support, each a page of
//! `crate::pages`), as on today's site — no Legal. This build's env
//! badge sits here at the footer's left, handed from `crate::app`'s [`BADGE`].

use leptos::prelude::*;
use palicanon_ui_lib::{EnvBadge, Footer, FooterLink};

use crate::app::BADGE;
use crate::catalogue::{Strings, strings};
use crate::pages::PageKind;

/// The footer, pushed to the bottom of a short page.
#[component]
pub fn SiteFooter() -> impl IntoView {
    let t = strings();
    view! {
        <div class="mt-auto">
            {move || {
                view! {
                    <Footer
                        links=links(t)
                        credits=Vec::new()
                        leading=|| view! { <EnvBadge env=BADGE.env sha=BADGE.sha /> }
                    />
                }
            }}
        </div>
    }
}

/// The footer pages, in today's order, labelled in `t`'s language.
fn links(t: Strings) -> Vec<FooterLink> {
    PageKind::ALL
        .into_iter()
        .map(|kind| FooterLink {
            label: kind.label(t.locale()),
            href: kind.path(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render() -> String {
        Owner::new().with(|| view! { <SiteFooter /> }.to_html())
    }

    /// Round 4 (user check): the four pages of today's site, in its
    /// order — no Legal.
    #[test]
    fn the_footer_links_the_four_pages_and_no_legal() {
        let html = render();
        let mut last = 0;
        for (href, label) in [
            ("/acknowledgements", "Acknowledgements"),
            ("/terms", "Terms"),
            ("/privacy", "Privacy"),
            ("/support", "Support"),
        ] {
            let at = html.find(&format!(r#"<a href="{href}""#)).expect(&html);
            assert!(at > last, "{html}");
            last = at;
            assert!(html.contains(&format!(">{label}</a>")), "{html}");
        }
        assert_eq!(html.matches("<a ").count(), 4, "{html}");
        assert!(
            !html.contains("/legal") && !html.contains("Legal"),
            "{html}"
        );
    }

    /// The labels are today's site's (`footer.nav_*`), in the language.
    #[test]
    fn the_footer_labels_follow_the_language() {
        let html = Owner::new().with(|| {
            palicanon_ui_lib::provide_language().set(palicanon_ui_lib::Language::Fr);
            view! { <SiteFooter /> }.to_html()
        });
        for label in ["Crédits", "Conditions", "Confidentialité", "Soutenir"] {
            assert!(html.contains(&format!(">{label}</a>")), "{html}");
        }
    }

    /// 044 (user, 2026-10-05): the build badge sits at the footer's left,
    /// before the links — first in the phone's column, first grid column
    /// on desktop.
    #[test]
    fn the_footer_carries_the_build_badge_at_its_left() {
        let html = render();
        let badge = html
            .find(&format!(r#"title="{} · {}""#, BADGE.env, BADGE.sha))
            .expect(&html);
        let nav = html.find("<nav").expect(&html);
        assert!(badge < nav, "{html}");
        let start = html[..badge].rfind("<div").expect(&html);
        let tag = &html[start..start + html[start..].find('>').expect(&html)];
        assert!(tag.contains("sm:col-start-1"), "{tag}");
    }
}
