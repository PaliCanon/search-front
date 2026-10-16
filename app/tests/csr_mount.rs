//! The fragment mounted by `csr::mount`, the entry the PaliCanon app calls
//! (task 021). `csr::mount` leaks its tree and the window listeners with it,
//! so this file has one test and the page to itself; `tests/csr.rs` mounts
//! and unmounts its hosts one by one.
//!
//! Run as `tests/csr.rs` is.

#![cfg(all(target_arch = "wasm32", feature = "csr"))]

mod common;

use common::{
    Fake, Host, find, location, marker_survived, one_hit, plant_marker, text, until, visit,
};
use leptos::prelude::*;
use palicanon_ui_lib::csr;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

/// Mounted by `csr::mount` at `/search`: the home renders, a query typed
/// and submitted navigates in place, the hits are shown, and the preferences
/// go to `localStorage` (no cookie is written by the fragment).
#[wasm_bindgen_test]
async fn search_runs_under_csr_mount() {
    visit("/search");
    let cookies = || {
        document()
            .unchecked_into::<web_sys::HtmlDocument>()
            .cookie()
            .unwrap_or_default()
    };
    let before = cookies();
    let fake = Fake::answering(true, one_hit("Canary Sutta"));
    let backend = fake.clone();
    csr::mount(move || view! { <Host backend=backend shown=ArcRwSignal::new(true) /> });
    until("the card", || find("#q").is_some()).await;
    assert!(text().contains("PaliCanon"), "{}", text());
    plant_marker();
    let input = find("#q")
        .expect("the field")
        .unchecked_into::<web_sys::HtmlInputElement>();
    input.set_value("metta");
    input
        .form()
        .expect("in a form")
        .request_submit()
        .expect("submitted");
    until("the results", || text().contains("Canary Sutta")).await;
    assert_eq!(location(), "/search?q=metta");
    assert!(marker_survived(), "the document was not loaded again");
    assert_eq!(fake.asked().len(), 1);
    assert_eq!(cookies(), before, "no cookie written");
}
