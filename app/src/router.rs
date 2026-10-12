//! The axum router: routes, the source in context, the body limit (slice 7 of task 005).
//!
//! One `Arc<Source>` is built at boot and handed in; every route gets it in
//! the reactive context, so the page's `Resource` and the `search_passages`
//! server function both find it (`expect_context`, no global). The token
//! lives inside that source and goes no further: nothing here reads it.
//! Every route takes a body of at most [`BODY_LIMIT`] bytes (the server
//! functions, `POST /api/search_passages` among them, take form bodies of
//! four short arguments): a larger one is refused before it reaches the
//! search API.

use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use leptos::prelude::{LeptosOptions, provide_context};
use leptos_axum::{LeptosRoutes, file_and_error_handler_with_context, generate_route_list};

use crate::app::{App, shell};
use crate::search::source::Source;

/// The largest request body any route takes, in bytes. A search is a query
/// of at most 200 characters (about 2 KiB percent-encoded, the worst case
/// for a Pāli one) and three short codes; this stays well above that and
/// far below anything abusive. A body over it is an error response (the
/// server function route answers 500, not 413: Plan A4), never a call to
/// the API.
pub const BODY_LIMIT: usize = 8 * 1024;

/// The service's router: the page's routes and the server functions, the
/// static files and the error page as the fallback, `source` in every
/// route's context.
pub fn router(options: LeptosOptions, source: Arc<Source>) -> axum::Router {
    let routes = generate_route_list(App);
    let context = move || provide_context(Arc::clone(&source));
    axum::Router::new()
        .leptos_routes_with_context(&options, routes, context.clone(), {
            let options = options.clone();
            move || shell(options.clone())
        })
        .fallback(file_and_error_handler_with_context(context, shell))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .with_state(options)
}

#[cfg(test)]
mod tests;
