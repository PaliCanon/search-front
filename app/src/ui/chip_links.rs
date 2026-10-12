//! A centred row of pill links: suggestions a reader can follow in one
//! click (example queries under a search card, related topics…). Plain
//! `<a href>`s, so they work before (or without) hydration.
//!
//! Modelled on today's site's example chips (`web/src/components/
//! Examples.tsx`).

use leptos::prelude::*;

/// One chip: what it says and where it goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChipLink {
    /// Shown in the pill.
    pub label: String,
    /// Where it goes.
    pub href: String,
}

/// The row, wrapped and centred, named for assistive technology.
#[component]
pub fn ChipLinks(
    /// The chips, in order.
    chips: Vec<ChipLink>,
    /// The row's accessible name.
    #[prop(into)]
    aria_label: String,
) -> impl IntoView {
    view! {
        <nav aria-label=aria_label>
            <ul class="flex flex-wrap items-center justify-center gap-2">
                {chips
                    .into_iter()
                    .map(|chip| {
                        view! {
                            <li>
                                <a
                                    href=chip.href
                                    class="block rounded-full border border-line bg-chip px-3.5 py-1.5 text-[13px] text-muted transition hover:border-gold/60 hover:text-heading"
                                >
                                    {chip.label}
                                </a>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </nav>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render() -> String {
        Owner::new().with(|| {
            let chips = vec![
                ChipLink {
                    label: "how to deal with anger".into(),
                    href: "/?q=anger".into(),
                },
                ChipLink {
                    label: "my body is getting old".into(),
                    href: "/?q=old".into(),
                },
            ];
            view! { <ChipLinks chips=chips aria_label="Examples" /> }.to_html()
        })
    }

    #[test]
    fn every_chip_is_a_real_link() {
        let html = render();
        assert!(html.contains(r#"href="/?q=anger""#), "{html}");
        assert!(html.contains(">how to deal with anger</a>"), "{html}");
        assert!(html.contains(r#"href="/?q=old""#), "{html}");
    }

    #[test]
    fn the_row_is_a_named_navigation_list() {
        let html = render();
        assert!(html.contains(r#"<nav aria-label="Examples""#), "{html}");
        assert_eq!(html.matches("<li").count(), 2, "{html}");
    }

    #[test]
    fn a_chip_is_a_pill() {
        let html = render();
        assert!(
            html.contains("rounded-full border border-line bg-chip"),
            "{html}"
        );
    }
}
