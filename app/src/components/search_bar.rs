//! The search bar, this service's wiring of two generic forms: the
//! home's `SearchCard` — the query field, the collection and mode
//! dropdowns inside the same `<form method="get">` — and a results
//! page's `SearchRow`, the one-line bar of its header (its dropdowns
//! sit under the header and join the form by its `id`,
//! [`SEARCH_FORM_ID`]). A submit re-renders the whole page with the new
//! `?q=&c=&mode=`, exactly what the server already does for a shared
//! link: no JavaScript needed. `/` focuses the field once hydrated
//! ([`is_focus_shortcut`]); "⌘K" / "Ctrl+K" works from anywhere, even
//! while typing elsewhere on the page.

use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use leptos::web_sys;
use leptos_i18n::td_string;

use crate::catalogue::strings;
use crate::components::filters::{CollectionSelect, ModeSelect};
use crate::search::{Collection, Mode};
use palicanon_ui_lib::{SearchCard, SearchRow};

/// The results page's search form `id`: its dropdowns, outside the
/// header, name it in their `form` attribute.
pub const SEARCH_FORM_ID: &str = "search-form";

/// Whether this keydown should focus and select the search field: `/`
/// outside any other field being typed into, or `k` with ⌘/Ctrl held,
/// from anywhere (mirrors today's site, `SearchBar.tsx`). Only called
/// from [`install_focus_shortcut`]'s hydrate-only listener; an `ssr`-only
/// build (no `hydrate`, no `test`) never calls it, hence the explicit
/// allow rather than a true dead function.
#[must_use]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn is_focus_shortcut(key: &str, modifier_held: bool, already_typing: bool) -> bool {
    (key == "k" && modifier_held) || (key == "/" && !already_typing)
}

/// The search card: the query, then the collection and mode dropdowns
/// beside the round submit button.
#[component]
pub fn SearchBar(
    /// The query to show pre-filled (from `?q=`).
    #[prop(into)]
    value: String,
    /// The mode `?mode=` resolved to.
    mode: Mode,
    /// The collection `?c=` resolved to.
    collection: Collection,
) -> impl IntoView {
    let t = strings();
    let input_ref = NodeRef::<leptos::html::Input>::new();
    install_focus_shortcut(input_ref);
    view! {
        <SearchCard
            value=value
            input_ref=input_ref
            label=Signal::derive(move || td_string!(t.locale(), search_label).to_owned())
            placeholder=Signal::derive(move || {
                td_string!(t.locale(), search_placeholder).to_owned()
            })
            submit_label=Signal::derive(move || td_string!(t.locale(), search_submit).to_owned())
        >
            <CollectionSelect value=collection />
            <ModeSelect value=mode />
        </SearchCard>
    }
}

/// A results page's one-line search bar (its header); the dropdowns are
/// the page's, under the header, bound to this form by [`SEARCH_FORM_ID`].
#[component]
pub fn ResultsSearchBar(
    /// The query to show pre-filled (from `?q=`).
    #[prop(into)]
    value: String,
) -> impl IntoView {
    let t = strings();
    let input_ref = NodeRef::<leptos::html::Input>::new();
    install_focus_shortcut(input_ref);
    view! {
        <SearchRow
            value=value
            input_ref=input_ref
            form_id=SEARCH_FORM_ID
            label=Signal::derive(move || td_string!(t.locale(), search_label).to_owned())
            placeholder=Signal::derive(move || {
                td_string!(t.locale(), search_placeholder).to_owned()
            })
            submit_label=Signal::derive(move || td_string!(t.locale(), search_submit).to_owned())
        />
    }
}

/// Browser only: `/` or ⌘K/Ctrl+K focuses and selects the search field.
/// No-op during SSR (there is no window, and no listener to leak either).
fn install_focus_shortcut(input_ref: NodeRef<leptos::html::Input>) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;
        let _ = leptos::prelude::window_event_listener(leptos::ev::keydown, move |ev| {
            let typing = ev
                .target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                .is_some_and(|el| {
                    matches!(el.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT")
                });
            if is_focus_shortcut(&ev.key(), ev.meta_key() || ev.ctrl_key(), typing) {
                ev.prevent_default();
                if let Some(el) = input_ref.get() {
                    let _ = el.focus();
                    el.select();
                }
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = input_ref;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slash_focuses_unless_already_typing() {
        assert!(is_focus_shortcut("/", false, false));
        assert!(
            !is_focus_shortcut("/", false, true),
            "typing elsewhere: let it type a literal /"
        );
    }

    #[test]
    fn cmd_or_ctrl_k_focuses_even_while_typing() {
        assert!(is_focus_shortcut("k", true, true));
        assert!(
            !is_focus_shortcut("k", false, true),
            "k alone is just a letter"
        );
    }

    #[test]
    fn unrelated_keys_never_match() {
        assert!(!is_focus_shortcut("Enter", true, false));
        assert!(!is_focus_shortcut("a", false, false));
    }
}
