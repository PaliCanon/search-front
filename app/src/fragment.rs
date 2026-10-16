//! The Search fragment: what a host mounts to get the search page without
//! the site's shell, footer or footer pages (task 021). The PaliCanon app
//! mounts it under its own `/search`; the site mounts it at the root.
//!
//! A host gives it [`SearchProvider`] (its [`SearchBackend`], required) and
//! mounts [`SearchRoutes`] where the page lives:
//!
//! ```ignore
//! <SearchProvider backend=my_backend>
//!     <Routes fallback=NotFound>
//!         <ParentRoute path=path!("/search") view=Outlet>
//!             <SearchRoutes />
//!         </ParentRoute>
//!     </Routes>
//! </SearchProvider>
//! ```
//!
//! Its URL is its public contract: `?q=` the query, `?c=` the collection
//! (`all`, `dn`, `mn`, `sn`, `an`, `kn`), `?mode=` (`both`, `words`,
//! `meaning`), `?passage=` the home's verse. The Map links to
//! `/search?q=<uid>`.
//!
//! What the fragment does not hold: the `AppShell` and its rail, the footer,
//! the footer pages (Acknowledgements, Terms, Privacy, Support stay the
//! site's routes), the document `<title>`. What it needs from its host,
//! each of them tested:
//!
//! - **the one language signal**: the host calls `provide_language` once;
//! - **root-served assets**: `/lotus.png` and `/brand/*.svg` answer at the
//!   host's root, whatever the fragment's base;
//! - **one [`SearchProvider`] per page**: the results page's form has a
//!   fixed `id` (`SEARCH_FORM_ID`);
//! - **the same ui-lib tag as the host's**: the host's build and this
//!   crate's must resolve one `palicanon-ui-lib` (a second copy would carry
//!   its own context types);
//! - **external links outside the window**: the read links and the verse's
//!   source are plain `https` links with `target="_blank"`; the host opens
//!   them in the system browser.
//!
//! The fragment's base is never a prop: every link is built from
//! `use_matched()` of its route, so the route and the links cannot drift.

use leptos::prelude::*;
use leptos_router::SsrMode;
use leptos_router::components::Route;
use leptos_router::path;

use crate::page::SearchPage;
use crate::search::backend::{Provided, SearchBackend};

pub(crate) mod submit;

/// Puts the host's [`SearchBackend`] in the context of everything below it:
/// the page asks it for every search. The backend is required, with no
/// default: a default in the app would be a call to an origin that is not
/// a server. Reads `backend.meaning()` once, at mount. One provider per
/// page.
#[component]
pub fn SearchProvider<B>(
    /// What answers the searches.
    backend: B,
    /// The screens that search: the fragment's routes, inside the host's.
    children: Children,
) -> impl IntoView
where
    B: SearchBackend,
{
    provide_context(Provided::new(backend));
    children()
}

/// The search page as one relative route (`path!("")`) the host mounts
/// where it wants it: at the root (the site) or inside its own parent route
/// (the app's `/search`). The page is the home while `?q=` is blank and the
/// results otherwise; rendered `SsrMode::Async` so the site's first paint
/// already holds the hits (inert under `csr`, which renders no first
/// paint). Needs a [`SearchProvider`] above it.
#[component(transparent)]
pub fn SearchRoutes() -> impl leptos_router::MatchNestedRoutes + Clone {
    view! { <Route path=path!("") view=SearchPage ssr=SsrMode::Async /> }.into_inner()
}

#[cfg(all(test, feature = "ssr"))]
mod tests;
