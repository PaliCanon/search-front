//! Every link the fragment builds, from one place (task 021). The base is
//! never a prop of the fragment: it is what the host's route matched
//! (`use_matched()`, the empty string at the root and `/search` where the
//! PaliCanon app mounts it), so the route and the links cannot drift. The
//! brand, the example queries, the verse's refresh, the retry links and the
//! in-place submit (`crate::fragment`) all go through [`href`].

use leptos::prelude::*;
use leptos_router::hooks::use_matched;

use crate::components::home::form_encode;
use crate::search::{Collection, Mode};

/// Where the fragment is mounted: the route that matched it, read once in
/// its route and followed reactively. `Copy`, so a component takes it as a
/// required prop and no link can be built without one.
#[derive(Clone, Copy)]
pub(crate) struct Base(Signal<String>);

impl Base {
    /// The base of the route this is called in (a route's view or below);
    /// panics outside a matched route, which is a host's mistake.
    pub(crate) fn matched() -> Self {
        Base(use_matched().into())
    }

    /// The root: the footer pages' brand link goes to the site's home, wherever
    /// those pages are (the site only).
    #[cfg(feature = "site")]
    pub(crate) fn root() -> Self {
        Base(Signal::stored(String::new()))
    }

    /// A fixed base, for a component's own test.
    #[cfg(test)]
    pub(crate) fn fixed(path: &str) -> Self {
        Base(Signal::stored(path.to_owned()))
    }

    /// The base as a path: `/` at the root, `/search` in the app.
    pub(crate) fn path(self) -> String {
        href(&self.0.get(), &[])
    }

    /// A link to the fragment's page with `params` as its query.
    pub(crate) fn href(self, params: &[(&str, &str)]) -> String {
        href(&self.0.get(), params)
    }

    /// The matched base itself (the empty string at the root).
    pub(crate) fn get(self) -> String {
        self.0.get()
    }
}

/// How many times the reader asked the same search again: submitted the same
/// query, or followed a Retry link to the URL the page is already at. The
/// page's `Resource` is keyed on it, so a terminal state (`Failed`, `Cold`,
/// `RateLimited`) is never a dead end: the URL does not change, the search
/// does. Provided by the page; zero for ever where nothing bumps it.
#[derive(Clone, Copy)]
pub(crate) struct Retry(RwSignal<u32>);

impl Retry {
    /// A counter at zero.
    pub(crate) fn new() -> Self {
        Retry(RwSignal::new(0))
    }

    /// The count, tracked.
    pub(crate) fn get(self) -> u32 {
        self.0.get()
    }

    /// The count, untracked.
    #[cfg(test)]
    pub(crate) fn get_untracked(self) -> u32 {
        self.0.get_untracked()
    }

    /// Asks again.
    pub(crate) fn bump(self) {
        self.0.update(|n| *n = n.wrapping_add(1));
    }
}

/// The one href builder: `base` is the path (the empty string is the root
/// `/`; a trailing slash is dropped), `params` its form-encoded query in
/// the order given, none being the path alone.
pub(crate) fn href(base: &str, params: &[(&str, &str)]) -> String {
    let base = base.trim_end_matches('/');
    let path = if base.is_empty() { "/" } else { base };
    if params.is_empty() {
        return path.to_owned();
    }
    let query: Vec<String> = params
        .iter()
        .map(|(name, value)| format!("{name}={}", form_encode(value)))
        .collect();
    format!("{path}?{}", query.join("&"))
}

/// A search as the page's own GET: `?q=&c=&mode=&passage=`, the query
/// trimmed, what the retry links point at and what a submit navigates to.
/// A field equal to its default (a blank query, `all`, `both`, no passage) is
/// not written, so neither a link nor a submit adds what the URL did not
/// have. `passage` is the home's `?passage=`, a field no form carries.
pub(crate) fn search_href(
    base: &str,
    q: &str,
    collection: Collection,
    mode: Mode,
    passage: &str,
) -> String {
    let mut params = Vec::new();
    if !q.trim().is_empty() {
        params.push(("q", q.trim()));
    }
    if collection != Collection::default() {
        params.push(("c", collection.code()));
    }
    if mode != Mode::default() {
        params.push(("mode", mode.code()));
    }
    if !passage.is_empty() {
        params.push(("passage", passage));
    }
    href(base, &params)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The one builder: the base is the path, the query is form-encoded in
    /// the order given, and no query is the path alone.
    #[test]
    fn the_href_builder_puts_the_base_first_and_encodes_the_query() {
        assert_eq!(href("", &[("q", "how to deal")]), "/?q=how+to+deal");
        assert_eq!(
            href("/search", &[("q", "gérer & co")]),
            "/search?q=g%C3%A9rer+%26+co"
        );
        assert_eq!(href("", &[("passage", "2")]), "/?passage=2");
        assert_eq!(href("", &[]), "/");
        assert_eq!(href("/search", &[]), "/search");
        assert_eq!(
            href("/search/", &[("q", "a")]),
            "/search?q=a",
            "no double slash"
        );
    }
}
