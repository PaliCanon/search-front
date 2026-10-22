//! The page footer: ui-lib's [`Footer`] with the site's links
//! (Acknowledgements · Terms · Privacy · Support, each a page of
//! `crate::pages`), as on today's site — no Legal. This deploy's release
//! badge sits here at the footer's left, from the [`ReleaseInfo`] the
//! document provides (`crate::app`); a host that provides none shows
//! `unknown`.

use leptos::prelude::*;
use palicanon_ui_lib::{EnvBadge, Footer, FooterLink};

use crate::catalogue::{Strings, strings};
use crate::pages::PageKind;
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
