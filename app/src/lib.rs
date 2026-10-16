//! search-front: the PaliCanon search page, Leptos SSR + hydrate on
//! `palicanon-ui-lib`, and the same page as a fragment the PaliCanon app
//! mounts client side (`csr`).
//!
//! The site (`ssr`, `hydrate`) answers searches through the `search_passages`
//! server function ([`search::site::SiteBackend`]) behind
//! [`search::SearchSource`]: the mock (`search::mock::MockSource`,
//! server-only) or the real `/v1/search` client. The app (`csr`) mounts
//! [`fragment::SearchRoutes`] inside its own shell and plugs its own
//! [`search::backend::SearchBackend`] (a local engine, never the network);
//! its build holds no server function, no endpoint and no token.

// `AppShell`'s nested `Option<ViewFn>` / `hidden=move || …` markup builds a
// type deep enough that rustc's default query-depth limit can overflow
// computing its layout for an unrelated reason elsewhere in the crate
// (pre-existing in ui-lib; ui-lab's gallery hits the same ceiling, tasks
// 010/027). Raised defensively since this service uses `AppShell` too.
#![recursion_limit = "512"]

#[cfg(feature = "site")]
pub mod app;
#[cfg(feature = "ssr")]
pub mod boot;
mod catalogue;
mod components;
pub mod fragment;
mod highlight;
mod links;
#[cfg(feature = "ssr")]
pub mod origin;
pub(crate) mod page;
#[cfg(feature = "site")]
mod pages;
#[cfg(feature = "ssr")]
pub mod router;
pub mod search;
pub mod ui;

// `build.rs` includes this same file (`#[path]`) for the env badge's pure
// decisions; declared here too so `cargo test -p search_front` covers it.
#[allow(dead_code)]
mod git_info;

/// Browser entry point: hydrate the server-rendered page from its root
/// (`<div id="pc-app">`, written by [`app::shell`]), not from `<body>`,
/// whose first child behind Cloudflare is an injected decoy link (ui-lib
/// 063); `<html>` then carries `data-hydrated`, which
/// `scripts/rollout-check.sh` reads.
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    palicanon_ui_lib::hydrate(app::App);
}
