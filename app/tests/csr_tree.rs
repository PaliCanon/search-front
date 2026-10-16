//! The csr build holds no token path (task 021): `cargo tree` of the app's
//! feature set on wasm32 lists none of the crates that carry the API client,
//! the server or the constant-time secret compare. The PaliCanon app is a
//! client that searches locally; a dependency that reached the network or
//! the token here would be a decision, not an accident.
#![cfg(feature = "ssr")]

use std::collections::BTreeSet;
use std::process::Command;

/// The crates the csr build must not hold: the API client (the token's
/// only user), the server and its runtime, the origin guard's compare.
const FORBIDDEN: [&str; 5] = [
    "palicanon-api-client",
    "axum",
    "leptos_axum",
    "tokio",
    "subtle",
];

/// The names of the normal dependencies of `search_front` built with
/// `features` and no default features, for `target` (the host if `None`).
fn tree(features: &str, target: Option<&str>) -> BTreeSet<String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let mut command = Command::new(cargo);
    command
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["tree", "--locked", "-e", "normal", "-p", "search_front"])
        .args(["--no-default-features", "--features", features])
        .args(["--prefix", "none"]);
    if let Some(target) = target {
        command.args(["--target", target]);
    }
    let out = command.output().expect("cargo tree runs");
    assert!(
        out.status.success(),
        "cargo tree {features}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_owned)
        .collect()
}

/// The forbidden crates among `names`.
fn forbidden_in(names: &BTreeSet<String>) -> Vec<&'static str> {
    FORBIDDEN
        .into_iter()
        .filter(|name| names.contains(*name))
        .collect()
}

/// The csr build, for the browser, holds none of them.
#[test]
fn the_csr_build_holds_no_token_path() {
    let names = tree("csr", Some("wasm32-unknown-unknown"));
    assert!(
        names.contains("leptos"),
        "the tree is the crate's: {names:?}"
    );
    assert_eq!(forbidden_in(&names), Vec::<&str>::new());
}

/// The guard is not vacuous: the server build (`ssr`, the host) holds every
/// one of them, so a dependency of the csr build that pulled one in would
/// be seen.
#[test]
fn the_guard_sees_the_crates_in_the_server_build() {
    let names = tree("ssr", None);
    assert_eq!(forbidden_in(&names), FORBIDDEN.to_vec());
}
