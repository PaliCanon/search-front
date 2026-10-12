//! The search card's two dropdowns, this service's wiring of the generic
//! [`Select`]: the collection (`?c=`) and the mode (`?mode=`). Both sit
//! inside the card's `<form method="get">`, so a choice travels with the
//! next search — no JavaScript needed (the native `<select>` until the
//! page hydrates, `crate::ui::select`).

use leptos::prelude::*;
use leptos_i18n::td_string;

use crate::catalogue::{Strings, strings};
use crate::search::{Collection, Mode};
use crate::ui::select::{Select, SelectOption};

/// The mode dropdown: `value` is the mode `?mode=` resolved to.
#[component]
pub fn ModeSelect(
    /// The mode `?mode=` resolved to.
    value: Mode,
    /// The search form's `id` when the dropdown sits outside it.
    #[prop(optional)]
    form: Option<&'static str>,
    /// Hydrated: a choice re-runs the search (the results page).
    #[prop(default = false)]
    submit_on_change: bool,
) -> impl IntoView {
    let t = strings();
    let chosen = RwSignal::new(value.code().to_owned());
    // Rebuilt on a language switch; `chosen` keeps the choice across it.
    move || {
        let options = [Mode::Both, Mode::Words, Mode::Meaning]
            .into_iter()
            .map(|mode| SelectOption {
                value: mode.code(),
                label: mode_str(t, mode),
            })
            .collect();
        view! {
            <Select
                name="mode"
                id="mode-menu"
                aria_label=td_string!(t.locale(), mode_group_label)
                options=options
                value=chosen
                form=form
                submit_on_change=submit_on_change
            />
        }
    }
}

/// The collection dropdown: `value` is the collection `?c=` resolved to.
#[component]
pub fn CollectionSelect(
    /// The collection `?c=` resolved to.
    value: Collection,
    /// The search form's `id` when the dropdown sits outside it.
    #[prop(optional)]
    form: Option<&'static str>,
    /// Hydrated: a choice re-runs the search (the results page).
    #[prop(default = false)]
    submit_on_change: bool,
) -> impl IntoView {
    let t = strings();
    let chosen = RwSignal::new(value.code().to_owned());
    move || {
        let options = Collection::all()
            .into_iter()
            .map(|collection| SelectOption {
                value: collection.code(),
                label: collection_label(t, collection),
            })
            .collect();
        view! {
            <Select
                name="c"
                id="collection-menu"
                aria_label=td_string!(t.locale(), collection_label)
                options=options
                value=chosen
                form=form
                submit_on_change=submit_on_change
            />
        }
    }
}

/// A collection's label: "All suttas" (translated), else its nikāya's
/// abbreviation and Pāli name (never translated).
fn collection_label(t: Strings, collection: Collection) -> &'static str {
    match collection {
        Collection::All => td_string!(t.locale(), collection_all),
        Collection::Dn => "DN — Dīgha Nikāya",
        Collection::Mn => "MN — Majjhima Nikāya",
        Collection::Sn => "SN — Saṁyutta Nikāya",
        Collection::An => "AN — Aṅguttara Nikāya",
        Collection::Kn => "KN — Khuddaka Nikāya",
    }
}

/// `mode`'s label, in `t`'s current language — a plain `match` because
/// `leptos_i18n`'s key is a compile-time identifier, not a runtime value.
fn mode_str(t: Strings, mode: Mode) -> &'static str {
    match mode {
        Mode::Words => td_string!(t.locale(), mode_words),
        Mode::Meaning => td_string!(t.locale(), mode_meaning),
        Mode::Both => td_string!(t.locale(), mode_both),
    }
}

/// `mode`'s label as an owned string (the found-via tag reads it too).
pub(crate) fn mode_label(t: Strings, mode: Mode) -> String {
    mode_str(t, mode).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The mode list puts the default first, like today's site.
    #[test]
    fn the_mode_dropdown_lists_the_default_first() {
        let html = Owner::new().with(|| view! { <ModeSelect value=Mode::Words /> }.to_html());
        let both = html.find(r#"value="both""#).expect(&html);
        let words = html.find(r#"value="words""#).expect(&html);
        assert!(both < words, "{html}");
        assert!(html.contains(r#"aria-label="Search mode""#), "{html}");
    }

    #[test]
    fn the_collection_dropdown_lists_every_nikaya() {
        let html =
            Owner::new().with(|| view! { <CollectionSelect value=Collection::All /> }.to_html());
        for collection in Collection::all() {
            assert!(
                html.contains(&format!(r#"value="{}""#, collection.code())),
                "{html}"
            );
        }
        assert!(html.contains(r#"aria-label="Collection""#), "{html}");
    }
}
