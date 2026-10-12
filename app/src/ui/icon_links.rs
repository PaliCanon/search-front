//! A small list of outbound icon links (an app store, a social account, a
//! support page): one image each, named by its label, opening in a new
//! tab. A row by default, a column when `vertical`.
//!
//! The icons are images the service serves (brand marks keep their own
//! colours there); this component adds no colour of its own.
//!
//! Modelled on today's site's `SocialLinks` (`web/src/components/
//! Footer.tsx`).

use leptos::prelude::*;

/// One link: its accessible name, where it goes, its image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconLink {
    /// Its accessible name and tooltip.
    pub label: String,
    /// Where it goes (opened in a new tab).
    pub href: String,
    /// The image's URL, served by the service.
    pub src: &'static str,
}

/// The list.
#[component]
pub fn IconLinks(
    /// The links, in order.
    links: Vec<IconLink>,
    /// A column instead of a row.
    #[prop(optional)]
    vertical: bool,
    /// The icons' size in CSS pixels.
    #[prop(default = 20)]
    size: u32,
) -> impl IntoView {
    let class = if vertical {
        "flex flex-col items-center gap-3"
    } else {
        "flex items-center gap-3"
    };
    view! {
        <ul class=class>
            {links
                .into_iter()
                .map(|link| {
                    view! {
                        <li>
                            <a
                                href=link.href
                                target="_blank"
                                rel="noopener"
                                aria-label=link.label.clone()
                                title=link.label
                                class="block rounded-md transition hover:-translate-y-0.5 hover:opacity-90"
                            >
                                <img src=link.src alt="" width=size height=size class="block" />
                            </a>
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn links() -> Vec<IconLink> {
        vec![
            IconLink {
                label: "Get the app".into(),
                href: "https://apps.example/app".into(),
                src: "/brand/app.svg",
            },
            IconLink {
                label: "Support".into(),
                href: "https://support.example".into(),
                src: "/brand/support.svg",
            },
        ]
    }

    #[test]
    fn every_link_opens_outside_named_by_its_label() {
        let html = Owner::new().with(|| view! { <IconLinks links=links() /> }.to_html());
        assert!(
            html.contains(r#"href="https://apps.example/app""#),
            "{html}"
        );
        assert!(html.contains(r#"aria-label="Get the app""#), "{html}");
        assert!(html.contains(r#"target="_blank""#), "{html}");
        assert!(html.contains(r#"rel="noopener""#), "{html}");
    }

    /// The image is decoration: the link carries the name.
    #[test]
    fn every_icon_is_a_decorative_image_of_the_given_size() {
        let html = Owner::new().with(|| view! { <IconLinks links=links() size=24 /> }.to_html());
        assert!(html.contains(r#"src="/brand/app.svg""#), "{html}");
        assert!(html.contains(r#"alt="""#), "{html}");
        assert!(html.contains(r#"width="24""#), "{html}");
    }

    /// `vertical` and the default row, each its own layout.
    #[test]
    fn vertical_stacks_the_links_in_a_column() {
        let column =
            Owner::new().with(|| view! { <IconLinks links=links() vertical=true /> }.to_html());
        let row = Owner::new().with(|| view! { <IconLinks links=links() /> }.to_html());
        assert!(column.contains("flex-col"), "{column}");
        assert!(!row.contains("flex-col"), "{row}");
    }
}
