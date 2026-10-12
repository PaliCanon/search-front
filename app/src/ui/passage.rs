//! A quoted passage: serif lines behind a gold rule, then the citation in
//! gold mono and its attribution (a translator), linked to the source; an
//! optional "another" link (a refresh icon) and an optional `aside` beside
//! the quote (a column of icon links, say).
//!
//! "Another" is a plain link: the service decides what the next passage
//! is (a `?passage=` it reads, say), so it works without JavaScript.
//!
//! Modelled on today's site's passage of the day (`web/src/components/
//! DailyPassage.tsx`).

use leptos::prelude::*;

/// The "another passage" link: where it goes and its accessible name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnotherLink {
    /// Where the next passage is.
    pub href: String,
    /// Its accessible name and tooltip (it shows an icon only).
    pub label: String,
}

/// The passage.
#[component]
pub fn Passage(
    /// The quoted lines, top to bottom.
    lines: Vec<String>,
    /// Where it comes from, e.g. "Dhammapada 223".
    #[prop(into)]
    citation: String,
    /// Who translated it.
    #[prop(into)]
    attribution: String,
    /// The source's URL.
    #[prop(into)]
    href: String,
    /// The refresh link, if there is another passage to show.
    #[prop(optional)]
    another: Option<AnotherLink>,
    /// The aside: shown beside the quote, on its left.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let last = lines.len().saturating_sub(1);
    view! {
        <div class="mx-auto flex w-fit max-w-[min(34rem,100%)] items-center gap-4 px-1">
            {children.map(|c| c())} <figure class="border-l-2 border-gold/60 pl-4">
                <blockquote class="font-serif text-[16px] leading-7 break-words text-heading sm:text-[18px] sm:leading-8">
                    {lines
                        .into_iter()
                        .enumerate()
                        .map(|(i, line)| {
                            view! {
                                {line}
                                {(i < last).then(|| view! { <br /> })}
                            }
                        })
                        .collect_view()}
                </blockquote>
                <figcaption class="mt-1.5 flex items-center gap-2 text-xs text-muted sm:text-[13px]">
                    <a href=href target="_blank" rel="noopener" class="hover:text-gold">
                        <span class="font-mono text-gold">{citation}</span>
                        <span class="ml-1.5 text-subtle">{format!("· {attribution}")}</span>
                    </a>
                    {another.map(|a| view! { <Another link=a /> })}
                </figcaption>
            </figure>
        </div>
    }
}

/// The refresh icon, a link to the next passage.
#[component]
fn Another(link: AnotherLink) -> impl IntoView {
    view! {
        <a
            href=link.href
            aria-label=link.label.clone()
            title=link.label
            class="ml-auto grid size-7 place-items-center rounded-full text-subtle transition hover:bg-chip hover:text-heading"
        >
            <svg
                width="15"
                height="15"
                viewBox="0 0 16 16"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
            >
                <path d="M3 8a5 5 0 0 1 8.5-3.5L13 6" />
                <path d="M13 3v3h-3" />
                <path d="M13 8a5 5 0 0 1-8.5 3.5L3 10" />
                <path d="M3 13v-3h3" />
            </svg>
        </a>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines() -> Vec<String> {
        vec![
            "Defeat anger with kindness,".into(),
            "villainy with virtue,".into(),
        ]
    }

    fn render() -> String {
        Owner::new().with(|| {
            view! {
                <Passage
                    lines=lines()
                    citation="Dhammapada 223"
                    attribution="Bhikkhu Sujato"
                    href="https://suttacentral.net/dhp221-234/en/sujato"
                />
            }
            .to_html()
        })
    }

    #[test]
    fn the_lines_are_a_serif_quote_behind_a_gold_rule() {
        let html = render();
        assert!(html.contains("<blockquote"), "{html}");
        assert!(html.contains("font-serif"), "{html}");
        assert!(html.contains("border-l-2 border-gold"), "{html}");
        assert!(html.contains("Defeat anger with kindness,<br>"), "{html}");
        assert!(html.contains("villainy with virtue,"), "{html}");
    }

    #[test]
    fn the_citation_is_gold_mono_linked_to_the_source_with_its_translator() {
        let html = render();
        assert!(
            html.contains(r#"href="https://suttacentral.net/dhp221-234/en/sujato""#),
            "{html}"
        );
        assert!(
            html.contains(r#"<span class="font-mono text-gold">Dhammapada 223</span>"#),
            "{html}"
        );
        assert!(html.contains("Bhikkhu Sujato"), "{html}");
    }

    #[test]
    fn another_is_a_named_link_with_a_refresh_icon() {
        let another = AnotherLink {
            href: "/?passage=1".into(),
            label: "Another passage".into(),
        };
        let html = Owner::new().with(|| {
            view! { <Passage lines=lines() citation="Dhp 223" attribution="S" href="/x" another=another /> }.to_html()
        });
        assert!(html.contains(r#"href="/?passage=1""#), "{html}");
        assert!(html.contains(r#"aria-label="Another passage""#), "{html}");
    }

    /// No `another`: no refresh link at all (an empty one would be a dead
    /// control).
    #[test]
    fn without_another_there_is_no_refresh_link() {
        let html = Owner::new().with(|| {
            view! { <Passage lines=lines() citation="Dhp 223" attribution="S" href="/x" /> }
                .to_html()
        });
        assert!(!html.contains("aria-label"), "{html}");
    }

    #[test]
    fn the_aside_sits_before_the_quote() {
        let html = Owner::new().with(|| {
            view! {
                <Passage lines=lines() citation="Dhp 223" attribution="S" href="/x">
                    <ul id="aside"></ul>
                </Passage>
            }
            .to_html()
        });
        let aside = html.find(r#"id="aside""#).expect(&html);
        let quote = html.find("<blockquote").expect(&html);
        assert!(aside < quote, "{html}");
    }
}
