//! A dropdown for a form: one value of a fixed list, sent under `name` with
//! the form around it. Two renderings of the same choice:
//!
//! - **No JavaScript** (the server's render, and the page until it
//!   hydrates): a native `<select name>` styled as a pill with a chevron —
//!   the browser's own picker, submitted by the form as is.
//! - **Hydrated**: the same pill as a button opening ui-lib's
//!   [`Menu`] of [`MenuItem::Choice`]s; the choice is written to a hidden
//!   `<input name>`, so the form sends exactly what the native one would.
//!
//! The swap happens in an [`Effect`], which only ever runs in the browser
//! after hydration: the server and the hydrating client both render the
//! native `<select>` first, so the two never disagree.
//!
//! Modelled on today's site's filter chips (`web/src/components/
//! FilterBar.tsx`, `Menu`).

use leptos::prelude::*;
use palicanon_ui_lib::{Disclosed, Disclosure, Menu, MenuItem};

/// One choice: what the form sends, and what the reader sees.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectOption {
    /// Sent with the form.
    pub value: &'static str,
    /// Shown in the pill and the list.
    pub label: &'static str,
}

/// The dropdown. `value` holds the current choice: a caller that builds
/// the options again (a language switch) passes the same signal, and the
/// choice survives.
#[component]
pub fn Select(
    /// The form field's name.
    name: &'static str,
    /// The list's id (the button's `aria-controls`); unique on the page.
    id: &'static str,
    /// Its accessible name (the pill shows the current choice only).
    #[prop(into)]
    aria_label: String,
    /// The choices, in order.
    options: Vec<SelectOption>,
    /// The current choice's `value`.
    value: RwSignal<String>,
    /// The `id` of the form this field belongs to when it sits outside
    /// it (the `form` attribute); `None` inside the form.
    #[prop(default = None)]
    form: Option<&'static str>,
    /// Hydrated only: a choice submits the form at once (today's results
    /// page re-runs the search on a filter change). The native `<select>`
    /// never does: without JavaScript the reader presses Search.
    #[prop(default = false)]
    submit_on_change: bool,
) -> impl IntoView {
    let hydrated = RwSignal::new(false);
    // Runs in the browser only, once hydrated: the server's render and the
    // hydrating client's first one are both the native `<select>`.
    Effect::new(move |_| hydrated.set(true));
    view! {
        <span class="relative inline-flex">
            {move || {
                if hydrated.get() {
                    let field = Field {
                        name,
                        form,
                        submit_on_change,
                    };
                    enhanced(field, id, aria_label.clone(), options.clone(), value)
                } else {
                    native(name, form, aria_label.clone(), options.clone(), value)
                }
            }}
        </span>
    }
}

/// The pill's look, shared by the native `<select>` and the button: a
/// gold ring while it has keyboard focus or its menu is open.
const PILL: &str = "inline-flex cursor-pointer items-center gap-1.5 whitespace-nowrap rounded-full \
    border border-line bg-transparent py-1 pl-3 text-[13px] text-ink transition \
    hover:border-card-edge hover:text-heading \
    focus-visible:border-gold focus-visible:ring-2 focus-visible:ring-gold/70 focus-visible:outline-none \
    aria-expanded:border-gold aria-expanded:ring-2 aria-expanded:ring-gold/70";

/// No JavaScript: the browser's own `<select>`, drawn as the pill (its
/// arrow hidden, [`Chevron`] drawn over it instead).
fn native(
    name: &'static str,
    form: Option<&'static str>,
    aria_label: String,
    options: Vec<SelectOption>,
    value: RwSignal<String>,
) -> AnyView {
    let current = value.get_untracked();
    view! {
        <select
            name=name
            form=form
            aria-label=aria_label
            class=format!("{PILL} appearance-none pr-8")
            on:change=move |ev| value.set(event_target_value(&ev))
        >
            {options
                .into_iter()
                .map(|o| {
                    view! {
                        <option value=o.value selected=o.value == current>
                            {o.label}
                        </option>
                    }
                })
                .collect_view()}
        </select>
        <Chevron class="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2" />
    }
    .into_any()
}

/// The hidden field the hydrated dropdown writes: its name, the form it
/// belongs to when outside it, and whether a choice submits that form.
#[derive(Clone, Copy)]
struct Field {
    name: &'static str,
    form: Option<&'static str>,
    submit_on_change: bool,
}

/// Hydrated: the pill as a button opening ui-lib's [`Menu`]; the choice
/// goes to a hidden input of the same `name`. A `value` matching no
/// option reads as the first one, as the native `<select>` would send.
fn enhanced(
    field: Field,
    id: &'static str,
    aria_label: String,
    options: Vec<SelectOption>,
    value: RwSignal<String>,
) -> AnyView {
    let open = RwSignal::new(false);
    let controls = Disclosed::new(id, open);
    let first = options.first().copied();
    let lookup = options.clone();
    let chosen = Signal::derive(move || {
        let v = value.get();
        lookup.iter().copied().find(|o| o.value == v).or(first)
    });
    let input_ref = NodeRef::<leptos::html::Input>::new();
    let items = options
        .into_iter()
        .map(|o| choice(o, value, open, field.submit_on_change.then_some(input_ref)))
        .collect();
    view! {
        <input
            type="hidden"
            name=field.name
            form=field.form
            node_ref=input_ref
            prop:value=move || chosen.get().map(|o| o.value)
        />
        <button
            type="button"
            aria-haspopup="menu"
            aria-expanded=move || open.get().to_string()
            aria-controls=id
            aria-label=aria_label.clone()
            class=format!("{PILL} pr-2.5")
            on:click=move |_| controls.toggle()
        >
            {move || chosen.get().map(|o| o.label)}
            <Chevron class="" />
        </button>
        <Menu controls=controls aria_label=aria_label items=items layout=MENU_LAYOUT />
    }
    .into_any()
}

/// Where the hydrated menu opens: under the pill, aligned to its left
/// edge (today's site's `Menu`, `align="left"`).
const MENU_LAYOUT: &str = "absolute top-full left-0 z-20 mt-2 min-w-[14rem]";

/// One option as a menu choice: picking it sets `value` and closes; with
/// `submit` (the hidden input), it also writes the input and submits its
/// form — the value is written directly first, so the form never sends
/// the previous choice.
fn choice(
    option: SelectOption,
    value: RwSignal<String>,
    open: RwSignal<bool>,
    submit: Option<NodeRef<leptos::html::Input>>,
) -> MenuItem {
    MenuItem::Choice {
        label: option.label,
        icon: None,
        selected: Signal::derive(move || value.get() == option.value),
        on_select: Callback::new(move |()| {
            value.set(option.value.to_owned());
            open.set(false);
            if let Some(input) = submit.and_then(|r| r.get()) {
                input.set_value(option.value);
                if let Some(form) = input.form() {
                    let _ = form.request_submit();
                }
            }
        }),
    }
}

/// The small down chevron of the pill.
#[component]
fn Chevron(class: &'static str) -> impl IntoView {
    view! {
        <svg
            class=class
            width="12"
            height="12"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d="m4 6 4 4 4-4" />
        </svg>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPTIONS: [SelectOption; 3] = [
        SelectOption {
            value: "both",
            label: "Words + meaning",
        },
        SelectOption {
            value: "words",
            label: "Words",
        },
        SelectOption {
            value: "meaning",
            label: "Meaning",
        },
    ];

    fn render(value: &str) -> String {
        let value = value.to_owned();
        Owner::new().with(|| {
            let value = RwSignal::new(value);
            view! {
                <Select
                    name="mode"
                    id="mode-list"
                    aria_label="Search mode"
                    options=OPTIONS.to_vec()
                    value=value
                />
            }
            .to_html()
        })
    }

    /// No JavaScript: the server renders a native `<select>`, which the
    /// form submits as is.
    #[test]
    fn the_server_renders_a_native_select_under_the_field_name() {
        let html = render("both");
        assert!(html.contains(r#"<select name="mode""#), "{html}");
        assert!(html.contains(r#"aria-label="Search mode""#), "{html}");
        for option in OPTIONS {
            assert!(
                html.contains(&format!(r#"value="{}""#, option.value)),
                "{html}"
            );
            assert!(
                html.contains(&format!(">{}</option>", option.label)),
                "{html}"
            );
        }
    }

    /// `value` and the one-at-a-time rule together: the chosen option is
    /// selected, and it is the only one.
    #[test]
    fn exactly_the_current_value_is_selected() {
        let html = render("words");
        assert!(html.contains(r#"<option value="words" selected"#), "{html}");
        assert_eq!(html.matches(" selected").count(), 1, "{html}");
    }

    /// An unknown value selects nothing: the browser then shows (and
    /// sends) the first option, never an invented one.
    #[test]
    fn an_unknown_value_selects_no_option() {
        let html = render("bogus");
        assert_eq!(html.matches(" selected").count(), 0, "{html}");
    }

    /// The native control is a pill with a chevron drawn over it (the
    /// browser's own arrow is hidden): the hydrated button looks the same.
    #[test]
    fn the_native_select_is_drawn_as_a_pill_with_a_chevron() {
        let html = render("both");
        assert!(html.contains("appearance-none"), "{html}");
        assert!(html.contains("rounded-full"), "{html}");
        assert!(html.contains("<svg"), "{html}");
    }

    /// Round 5 (user check, 2026-10-05): the chips sit at the left, so
    /// the hydrated menu opens aligned to the left (today's site's `Menu`,
    /// `left-0`); the hidden input carries the form it belongs to.
    #[test]
    fn the_hydrated_menu_opens_aligned_to_the_left_and_names_its_form() {
        let html = Owner::new().with(|| {
            let value = RwSignal::new("both".to_owned());
            let field = Field {
                name: "mode",
                form: Some("search-form"),
                submit_on_change: true,
            };
            enhanced(
                field,
                "mode-list",
                "Search mode".to_owned(),
                OPTIONS.to_vec(),
                value,
            )
            .to_html()
        });
        let list = html.find(r#"id="mode-list""#).expect(&html);
        let start = html[..list].rfind('<').expect(&html);
        let end = list + html[list..].find('>').expect(&html);
        let tag = &html[start..=end];
        assert!(tag.contains("left-0"), "{tag}");
        assert!(!tag.contains("right-0"), "{tag}");
        assert!(
            html.contains(r#"<input type="hidden" name="mode" form="search-form""#),
            "{html}"
        );
    }

    /// Inside the form, no `form` attribute is written at all.
    #[test]
    fn inside_the_form_the_native_select_carries_no_form_attribute() {
        let html = render("both");
        assert!(
            html.contains(r#"<select name="mode" aria-label="#),
            "{html}"
        );
        assert!(!html.contains("form="), "{html}");
    }

    /// Keyboard focus (and, hydrated, the open menu: `aria-expanded`)
    /// rings the pill in gold, as in the user's expected screenshot.
    #[test]
    fn keyboard_focus_rings_the_pill_in_gold() {
        let html = render("both");
        assert!(html.contains("focus-visible:border-gold"), "{html}");
        assert!(html.contains("focus-visible:ring-2"), "{html}");
        assert!(PILL.contains("aria-expanded:border-gold"), "{PILL}");
    }
}
