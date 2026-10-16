//! The page's own pieces: the search card and its dropdowns, the home's
//! content, the results and their states, the footer, and the not-found page.

pub mod filters;
pub mod home;
#[cfg(feature = "site")]
pub mod not_found;
pub mod result_list;
pub mod search_bar;
#[cfg(feature = "site")]
pub mod site_footer;
pub mod states;
