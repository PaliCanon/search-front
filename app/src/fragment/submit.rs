//! The submit of the search forms, in place: under `csr` the fragment
//! navigates instead of loading a document (the app's webview has no server
//! to load one from). The site (`ssr`, `hydrate`) keeps the browser's own
//! GET, so this wraps nothing there.

use leptos::prelude::*;
use leptos_router::params::ParamsMap;

use crate::links::{Base, Retry, search_href};
use crate::search::{Collection, Mode};

/// Wraps the page. Under `csr` its one `on:submit` catches the submit of the
/// card, of the results page's bar, and of their dropdowns (`form
/// .request_submit()` fires the same event), prevents the document load and
/// navigates to the page's own GET, built from the form's `FormData` (`q`,
/// and `c` and `mode`, which the results page's dropdowns join through their
/// `form` attribute) on the matched base by the one href builder the links
/// use. A field the form does not carry (the mode dropdown while the
/// backend has no meaning, and `?passage=`, which no form carries) keeps
/// what the current URL asked, so a shared `?mode=meaning` link is not
/// rewritten by the next query; a field equal to its default is not written. Elsewhere it
/// renders its children as they are, same HTML.
#[component]
pub(crate) fn InPlaceSubmit(
    /// Where the fragment is mounted: the navigation stays on its page.
    base: Base,
    /// Asked again when a submit is for the search the page already shows
    /// (`same_search` says so), so the same query submitted twice searches
    /// twice. A submit for another search only navigates: the new URL asks
    /// the new search, and a bump on top would first ask the old one again.
    retry: Retry,
    /// Whether a submit with these parameters (`q`, `c`, `mode`) is the
    /// search the page already shows.
    same_search: Callback<ParamsMap, bool>,
    /// The page.
    children: Children,
) -> impl IntoView {
    #[cfg(feature = "csr")]
    {
        let navigate = leptos_router::hooks::use_navigate();
        let asked = leptos_router::hooks::use_query_map();
        view! {
            <div
                class="contents"
                on:submit=move |ev| {
                    if submit_in_place(&navigate, base, asked, same_search, &ev) {
                        retry.bump();
                    }
                }
            >
                {children()}
            </div>
        }
    }
    #[cfg(not(feature = "csr"))]
    {
        let _ = (base, retry, same_search);
        children()
    }
}

/// Navigates to the page's own GET for the form that was submitted; says
/// whether it is the search the page already shows.
#[cfg(feature = "csr")]
fn submit_in_place(
    navigate: &impl Fn(&str, leptos_router::NavigateOptions),
    base: Base,
    asked: Memo<ParamsMap>,
    same_search: Callback<ParamsMap, bool>,
    ev: &leptos::ev::SubmitEvent,
) -> bool {
    use wasm_bindgen::JsCast;

    ev.prevent_default();
    let Some(form) = ev
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlFormElement>().ok())
    else {
        return false;
    };
    let Ok(data) = web_sys::FormData::new_with_form(&form) else {
        return false;
    };
    let field = |name: &str| data.get(name).as_string();
    // What the form does not carry stays what the URL asked.
    let kept = |name: &str| {
        field(name)
            .or_else(|| asked.get_untracked().get(name))
            .unwrap_or_default()
    };
    let href = form_href(
        &base.get(),
        &kept("q"),
        &kept("c"),
        &kept("mode"),
        &kept("passage"),
    );
    navigate(&href, leptos_router::NavigateOptions::default());
    let mut sent = ParamsMap::new();
    for name in ["q", "c", "mode"] {
        sent.insert(name, kept(name));
    }
    same_search.run(sent)
}

/// [`search_href`] of a submitted form's raw values (`c` and `mode` as the
/// dropdowns write them; a stray or missing value is defaulted; `passage`
/// is what the URL had, the form carries none).
#[cfg_attr(not(feature = "csr"), allow(dead_code))]
pub(crate) fn form_href(
    base: &str,
    q: &str,
    collection: &str,
    mode: &str,
    passage: &str,
) -> String {
    search_href(
        base,
        q,
        Collection::parse(collection),
        Mode::parse(mode),
        passage,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What a submitted form becomes: the query trimmed, the collection and
    /// the mode as the retry link spells them, a stray value defaulted and a
    /// field equal to its default not written.
    #[test]
    fn a_submitted_form_becomes_the_pages_own_get() {
        assert_eq!(
            form_href("/search", " metta ", "mn", "words", ""),
            "/search?q=metta&c=mn&mode=words"
        );
        assert_eq!(form_href("", "a b", "bogus", "", ""), "/?q=a+b");
        assert_eq!(form_href("/search", "  ", "", "", ""), "/search");
        assert_eq!(
            form_href("/search", "metta", "all", "both", "3"),
            "/search?q=metta&passage=3"
        );
    }
}
