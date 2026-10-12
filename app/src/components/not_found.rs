//! The not-found page: rendered for any unknown path in the router. A
//! centred column with the lotus and "PaliCanon" link to `/`, the eyebrow,
//! the title, the body, and a gold underlined link to `/`. Same in every
//! language.

use leptos::prelude::*;
use leptos_i18n::td_string;
use leptos_meta::{Meta, Title};

use crate::catalogue::strings;

/// The not-found page: a centred column with the brand, eyebrow, title, body,
/// and home link. Sets the page title, disallows indexing.
#[component]
pub fn NotFoundPage() -> impl IntoView {
    let t = strings();
    move || {
        let locale = t.locale();
        let head_title = td_string!(locale, notfound_head_title);
        view! {
            <Title text=head_title />
            <Meta name="robots" content="noindex" />
            <div class="mx-auto flex min-h-dvh max-w-[64rem] flex-col items-center justify-center px-6 text-center">
                <a
                    href="/"
                    class="mb-6 flex items-center gap-2 font-serif text-lg font-semibold text-heading"
                >
                    <img src="/lotus.png" alt="" width="22" height="22" />
                    "PaliCanon"
                </a>
                <p class="mb-2 font-mono text-xs tracking-widest text-subtle">
                    {td_string!(locale, notfound_eyebrow)}
                </p>
                <h1 class="font-serif text-4xl font-semibold text-heading">
                    {td_string!(locale, notfound_title)}
                </h1>
                <p class="mt-4 max-w-[42ch] text-[15px] leading-7 text-ink">
                    {td_string!(locale, notfound_body)}
                </p>
                <a
                    href="/"
                    class="mt-6 text-gold underline underline-offset-4 hover:text-gold-hover"
                >
                    {td_string!(locale, notfound_home)}
                </a>
            </div>
        }
    }
}
