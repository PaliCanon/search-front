//! The home's own content, around the search card: the brand (lotus and
//! name), the example queries (three of a pool of nine, rotated by day),
//! the passage with the app and social links (the verse of the day, unless
//! `?passage=` names one).
//! Shown only while no `?q=` is set (`crate::page`) — but the brand's
//! link form, [`BrandLink`], heads a results page.

use std::fmt::Write as _;

use leptos::prelude::*;
use leptos_i18n::td_string;

use crate::catalogue::{Locale, strings};
use crate::links::Base;
use crate::ui::chip_links::{ChipLink, ChipLinks};
use crate::ui::icon_links::{IconLink, IconLinks};
use crate::ui::passage::{AnotherLink, Passage};

/// One mocked verse of the passage (real data waits for a daily feed).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Verse {
    /// SuttaCentral uid of the text holding it, e.g. `dhp221-234`.
    pub uid: &'static str,
    /// Its citation, e.g. "Dhammapada 223".
    pub citation: &'static str,
    /// Its translator, as credited.
    pub translator: &'static str,
    /// Its lines, top to bottom.
    pub lines: &'static [&'static str],
}

/// The mocked verses, in the order the refresh link walks them (the
/// first one is the reference's).
pub(crate) const VERSES: [Verse; 4] = [
    Verse {
        uid: "dhp221-234",
        citation: "Dhammapada 223",
        translator: "Bhikkhu Sujato",
        lines: &[
            "Defeat anger with kindness,",
            "villainy with virtue,",
            "stinginess with giving,",
            "and lies with truth.",
        ],
    },
    Verse {
        uid: "dhp1-20",
        citation: "Dhammapada 5",
        translator: "Bhikkhu Sujato",
        lines: &[
            "For never is hatred",
            "laid to rest by hate,",
            "it’s laid to rest by love:",
            "this is an ancient teaching.",
        ],
    },
    Verse {
        uid: "dhp179-196",
        citation: "Dhammapada 183",
        translator: "Bhikkhu Sujato",
        lines: &[
            "Not to do any evil;",
            "to embrace the good;",
            "to purify one’s mind:",
            "this is the instruction of the Buddhas.",
        ],
    },
    Verse {
        uid: "dhp197-208",
        citation: "Dhammapada 204",
        translator: "Bhikkhu Sujato",
        lines: &[
            "Health is the ultimate blessing;",
            "contentment, the ultimate wealth;",
            "trust is the ultimate family;",
            "extinguishment, the ultimate happiness.",
        ],
    },
];

/// The lotus and the name, centred above the card (the home).
#[component]
pub fn Brand() -> impl IntoView {
    view! {
        <h1 class="mb-6 flex items-center justify-center gap-3 font-serif text-[1.75rem] font-semibold text-heading select-none">
            <img src="/lotus.png" alt="" width="30" height="30" />
            "PaliCanon"
        </h1>
    }
}

/// The lotus and the name, a link home: a results page's header's first
/// item, at the left of the search bar (`crate::page`; today's site's
/// `brand`). Hidden on a phone, where the shell's top bar already shows
/// them.
#[component]
pub fn BrandLink(
    /// Where the fragment is mounted: the link goes to its page.
    base: Base,
) -> impl IntoView {
    view! {
        <a
            href=move || base.path()
            class="hidden shrink-0 items-center gap-2 font-serif text-lg font-semibold text-heading select-none sm:flex"
        >
            <img src="/lotus.png" alt="" width="22" height="22" />
            "PaliCanon"
        </a>
    }
}

/// How many example queries the pool holds.
pub(crate) const POOL: usize = 9;

/// How many of them the home shows at once.
const SHOWN: usize = 3;

/// The pool in `locale`, in the order the days walk it: today's site's six
/// (its order, its wording), then three more.
pub(crate) fn example_labels(locale: Locale) -> [&'static str; POOL] {
    [
        td_string!(locale, example_grief),
        td_string!(locale, example_left),
        td_string!(locale, example_speech),
        td_string!(locale, example_anger),
        td_string!(locale, example_regret),
        td_string!(locale, example_old),
        td_string!(locale, example_worry),
        td_string!(locale, example_forgive),
        td_string!(locale, example_restless),
    ]
}

/// 2026-10-01 UTC, today's site's day zero (`DailyPassage.tsx` EPOCH).
const EPOCH_MS: i64 = 1_790_812_800_000;

/// One day, in milliseconds.
const DAY_MS: i64 = 86_400_000;

/// Which of `len` items is today's: whole days since [`EPOCH_MS`], wrapped,
/// never negative (a clock before the epoch still lands inside the pool).
///
/// # Panics
///
/// If `len` is zero or does not fit an `i64`.
pub(crate) fn day_index(now_ms: i64, len: usize) -> usize {
    let len = i64::try_from(len).expect("a pool length fits an i64");
    let day = (now_ms - EPOCH_MS).div_euclid(DAY_MS);
    usize::try_from(day.rem_euclid(len)).expect("a remainder in 0..len fits a usize")
}

/// The clock, in milliseconds since the Unix epoch: the server's.
#[cfg(feature = "ssr")]
pub(crate) fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| {
            i64::try_from(since.as_millis()).unwrap_or(i64::MAX)
        })
}

/// The clock, in milliseconds since the Unix epoch: the browser's.
#[cfg(not(feature = "ssr"))]
#[allow(clippy::cast_possible_truncation)] // milliseconds since 1970 fit an i64 for millennia
pub(crate) fn now_ms() -> i64 {
    js_sys::Date::now() as i64
}

/// Three consecutive example queries of the pool, each a link that runs it
/// (`/?q=…`), starting at today's index (`day_index`), wrapping at the end
/// of the pool.
#[component]
pub fn Examples(
    /// Where the fragment is mounted: the links run a query on its page.
    base: Base,
    /// The day to show (tests); today's by default.
    #[prop(optional)]
    day: Option<usize>,
) -> impl IntoView {
    let t = strings();
    let start = day.unwrap_or_else(|| day_index(now_ms(), POOL));
    move || {
        let labels = example_labels(t.locale());
        let chips = (0..SHOWN)
            .map(|offset| labels[(start + offset) % POOL])
            .map(|label| ChipLink {
                label: label.to_owned(),
                href: base.href(&[("q", label)]),
            })
            .collect();
        view! { <ChipLinks chips=chips aria_label=td_string!(t.locale(), examples_label) /> }
    }
}

/// `text` encoded as a form value (`application/x-www-form-urlencoded`,
/// what a GET form itself sends): a space is `+`, anything outside the
/// unreserved set is `%XX` per UTF-8 byte.
pub(crate) fn form_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(char::from(byte));
            }
            b' ' => out.push('+'),
            _ => {
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
    out
}

/// The verse `?passage=` names (today's, `day_index`, for anything else),
/// the app and social links beside it, a refresh link to the next verse.
#[component]
pub fn DailyPassage(
    /// Where the fragment is mounted: the refresh link goes to its page.
    base: Base,
    /// The raw `?passage=` value, if any.
    #[prop(into)]
    raw: String,
    /// The day to show when `raw` names no verse (tests); today's by default.
    #[prop(optional)]
    day: Option<usize>,
) -> impl IntoView {
    let t = strings();
    let index = raw
        .parse::<usize>()
        .ok()
        .filter(|i| *i < VERSES.len())
        .unwrap_or_else(|| day.unwrap_or_else(|| day_index(now_ms(), VERSES.len())));
    let verse = VERSES[index];
    let next = (index + 1) % VERSES.len();
    move || {
        view! {
            <Passage
                lines=verse.lines.iter().map(|l| (*l).to_owned()).collect()
                citation=verse.citation
                attribution=verse.translator
                href=format!("https://suttacentral.net/{}/en/sujato", verse.uid)
                another=AnotherLink {
                    href: base.href(&[("passage", &next.to_string())]),
                    label: td_string!(t.locale(), passage_another).to_owned(),
                }
            >
                <IconLinks links=social_links(t) vertical=true size=20 />
            </Passage>
        }
    }
}

/// The app, Instagram and Ko-fi links (today's site's `SocialLinks`, the
/// home's campaign tag on the App Store link).
fn social_links(t: crate::catalogue::Strings) -> Vec<IconLink> {
    vec![
        IconLink {
            label: td_string!(t.locale(), social_app).to_owned(),
            href: "https://apps.apple.com/app/apple-store/id6760995639?pt=128678815&ct=PaliWebHome&mt=8".to_owned(),
            src: "/brand/app-store.svg",
        },
        IconLink {
            label: td_string!(t.locale(), social_instagram).to_owned(),
            href: "https://www.instagram.com/daily_sutta".to_owned(),
            src: "/brand/instagram.svg",
        },
        IconLink {
            label: td_string!(t.locale(), social_kofi).to_owned(),
            href: "https://ko-fi.com/tipitaka".to_owned(),
            src: "/brand/kofi.svg",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_encode_writes_spaces_as_plus_and_escapes_the_rest() {
        assert_eq!(form_encode("how to deal"), "how+to+deal");
        assert_eq!(form_encode("gérer & co"), "g%C3%A9rer+%26+co");
    }

    const EN_POOL: [&str; POOL] = [
        "how to live with grief",
        "the person I love left me",
        "how to speak truthfully and kindly",
        "how to deal with anger",
        "dealing with regret and guilt",
        "my body is getting old",
        "how to let go of worry",
        "how to forgive someone who hurt me",
        "what to do when the mind won't settle",
    ];

    fn render_examples(day: usize) -> String {
        Owner::new().with(|| view! { <Examples base=Base::fixed("") day=day /> }.to_html())
    }

    #[test]
    fn day_index_counts_whole_days_from_the_epoch_and_wraps() {
        assert_eq!(day_index(EPOCH_MS, 9), 0);
        assert_eq!(day_index(EPOCH_MS + DAY_MS, 9), 1);
        assert_eq!(day_index(EPOCH_MS + DAY_MS - 1, 9), 0);
        assert_eq!(day_index(EPOCH_MS + 9 * DAY_MS, 9), 0);
        assert_eq!(day_index(EPOCH_MS + 10 * DAY_MS, 9), 1);
        // Before the epoch: never negative, never a panic.
        assert_eq!(day_index(EPOCH_MS - 1, 9), 8);
        assert_eq!(day_index(EPOCH_MS - 10 * DAY_MS, 4), 2);
    }

    #[test]
    fn the_examples_show_three_consecutive_entries_of_the_pool_from_the_day() {
        let html = render_examples(7);
        for index in [7, 8, 0] {
            let label = EN_POOL[index];
            let link = format!(r#"href="/?q={}""#, form_encode(label));
            assert!(html.contains(&link), "{index}: {html}");
            assert!(html.contains(&format!(">{label}</a>")), "{index}: {html}");
        }
        assert_eq!(html.matches(r#"href="/?q="#).count(), 3, "{html}");
        for label in &EN_POOL[1..7] {
            assert!(!html.contains(label), "{label}: {html}");
        }
    }

    /// G5: at the app's base the examples and the verse's refresh link stay
    /// under it, and no link leaves it.
    #[test]
    fn the_examples_and_the_passage_links_carry_the_base() {
        let examples = Owner::new()
            .with(|| view! { <Examples base=Base::fixed("/search") day=7 /> }.to_html());
        assert_eq!(
            examples.matches(r#"href="/search?q="#).count(),
            3,
            "{examples}"
        );
        assert!(!examples.contains(r#"href="/?q="#), "{examples}");
        let passage = Owner::new()
            .with(|| view! { <DailyPassage base=Base::fixed("/search") raw="" day=2 /> }.to_html());
        assert!(passage.contains(r#"href="/search?passage=3""#), "{passage}");
        assert!(!passage.contains(r#"href="/?passage"#), "{passage}");
    }

    #[test]
    fn the_examples_pool_has_todays_sites_six_and_three_more() {
        assert_eq!(example_labels(Locale::en), EN_POOL);
    }

    #[test]
    fn the_verse_of_the_day_shows_without_a_passage_and_a_valid_one_beats_it() {
        let html = Owner::new()
            .with(|| view! { <DailyPassage base=Base::fixed("") raw="" day=2 /> }.to_html());
        assert!(html.contains(VERSES[2].lines[0]), "{html}");
        assert!(html.contains(r#"href="/?passage=3""#), "{html}");
        let html = Owner::new()
            .with(|| view! { <DailyPassage base=Base::fixed("") raw="1" day=2 /> }.to_html());
        assert!(html.contains(VERSES[1].lines[0]), "{html}");
        assert!(html.contains(r#"href="/?passage=2""#), "{html}");
    }
}
