//! search-front: the PaliCanon search page, Leptos SSR + hydrate on
//! `palicanon-ui-lib`. Search answers behind [`search::SearchSource`]: the
//! mock (`search::mock::MockSource`, server-only) today, the real `/v1/search`
//! client (task 005) without the view code changing.

// `AppShell`'s nested `Option<ViewFn>` / `hidden=move || …` markup builds a
// type deep enough that rustc's default query-depth limit can overflow
// computing its layout for an unrelated reason elsewhere in the crate
// (pre-existing in ui-lib; ui-lab's gallery hits the same ceiling, tasks
// 010/027). Raised defensively since this service uses `AppShell` too.
#![recursion_limit = "512"]

pub mod app;
#[cfg(feature = "ssr")]
pub mod boot;
mod catalogue;
mod components;
mod highlight;
pub(crate) mod page;
mod pages;
#[cfg(feature = "ssr")]
pub mod router;
pub mod search;
pub mod ui;

// `build.rs` includes this same file (`#[path]`) for the env badge's pure
// decisions; declared here too so `cargo test -p search_front` covers it.
#[allow(dead_code)]
mod git_info;

/// Browser entry point: hydrate the server-rendered page.
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
