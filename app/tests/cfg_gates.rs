//! The grep gates of the build modes (task 021, as ui-lib's 067): `hydrate`
//! and `csr` share the code that needs a browser through the private
//! `browser` feature, so a `cfg(feature = "hydrate")` left in `src/` would
//! silently drop that code (the `/` focus shortcut, the browser clock) from
//! a `csr` build, which compiles without a word. These tests read the
//! sources and the manifest and fail on:
//!
//! - any `feature = "hydrate"` outside `src/lib.rs` (the `hydrate()` entry,
//!   which panics on a DOM the server never rendered, stays on `hydrate`);
//! - any `feature = "csr"` outside `src/fragment.rs` and `src/fragment/`
//!   (the submit that navigates in place is the one csr-only behaviour);
//! - a manifest where `csr` reaches the site (`site`, `ssr`) or `ssr`
//!   reaches the browser, or where `hydrate` and `csr` do not both enable
//!   `browser`, or where `ssr` and `hydrate` do not both enable `site`.
//!
//! The matching ignores whitespace, so `feature="hydrate"`,
//! `feature  =  "hydrate"`, `not (feature = "hydrate")` and a `cfg_attr`
//! are all seen.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::{Path, PathBuf};

/// Every `.rs` file under `dir`, recursively.
fn rust_files(dir: &Path, into: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("the source directory is readable") {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            rust_files(&path, into);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            into.push(path);
        }
    }
}

/// `line` with every whitespace removed.
fn squeeze(line: &str) -> String {
    line.split_whitespace().collect()
}

/// Whether `line` names `feature = "<name>"`, however it is spaced.
fn names_feature(line: &str, name: &str) -> bool {
    squeeze(line).contains(&format!(r#"feature="{name}""#))
}

/// The `file:line: text` of each line under `src/` that names feature
/// `name`, leaving out the files for which `allowed` (a path relative to
/// `src/`) holds.
fn offenders(name: &str, allowed: impl Fn(&Path) -> bool) -> Vec<String> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    let mut found = Vec::new();
    for file in files {
        let relative = file.strip_prefix(&src).expect("under src/");
        if allowed(relative) {
            continue;
        }
        let text = fs::read_to_string(&file).expect("a UTF-8 source file");
        for (number, line) in text.lines().enumerate() {
            if names_feature(line, name) {
                found.push(format!(
                    "{}:{}: {}",
                    relative.display(),
                    number + 1,
                    line.trim()
                ));
            }
        }
    }
    found
}

#[test]
fn no_bare_hydrate_feature_outside_the_entry() {
    let found = offenders("hydrate", |path| path == Path::new("lib.rs"));
    assert_eq!(
        found.len(),
        0,
        "code that csr needs is gated on `browser` (or the site on `site`), not `hydrate`:\n{}",
        found.join("\n")
    );
}

#[test]
fn the_entry_is_gated_on_hydrate() {
    let lib = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs"))
        .expect("src/lib.rs is readable");
    let entry = lib
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .windows(3)
        .any(|w| {
            squeeze(w[0]) == r#"#[cfg(feature="hydrate")]"#
                && w[1].starts_with("#[wasm_bindgen")
                && w[2].starts_with("pub fn hydrate")
        });
    assert!(
        entry,
        "`pub fn hydrate` carries #[cfg(feature = \"hydrate\")]"
    );
}

#[test]
fn no_bare_csr_feature_outside_the_fragment() {
    let found = offenders("csr", |path| {
        path == Path::new("fragment.rs") || path.starts_with("fragment")
    });
    assert_eq!(
        found.len(),
        0,
        "only the fragment is gated on `csr`:\n{}",
        found.join("\n")
    );
}

/// The quoted entries of feature `name` in `[features]` of the manifest.
fn feature(name: &str) -> Vec<String> {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("Cargo.toml is readable");
    let features = manifest
        .split("\n[features]")
        .nth(1)
        .expect("a [features] table");
    let features = features.split("\n[").next().expect("the table's text");
    let start = features
        .find(&format!("\n{name} = ["))
        .unwrap_or_else(|| panic!("feature {name} is defined"));
    let rest = &features[start + 1..];
    let list = &rest[rest.find('[').expect("a list") + 1..];
    let list = &list[..list.find(']').expect("the list ends")];
    list.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .flat_map(|line| line.split('"').skip(1).step_by(2))
        .map(str::to_owned)
        .collect()
}

#[test]
fn hydrate_and_csr_share_the_browser_code() {
    for name in ["hydrate", "csr"] {
        assert!(feature(name).contains(&"browser".to_owned()), "{name}");
    }
    assert!(!feature("ssr").contains(&"browser".to_owned()));
}

#[test]
fn the_site_is_ssr_or_hydrate_never_csr() {
    for name in ["ssr", "hydrate"] {
        assert!(feature(name).contains(&"site".to_owned()), "{name}");
    }
    let csr = feature("csr");
    for forbidden in ["site", "ssr", "hydrate"] {
        assert!(
            !csr.contains(&forbidden.to_owned()),
            "csr names {forbidden}"
        );
    }
    assert!(csr.contains(&"leptos/csr".to_owned()));
    assert!(csr.contains(&"palicanon-ui-lib/csr".to_owned()));
}

#[test]
fn the_gates_see_through_spacing_and_cfg_attr() {
    for line in [
        r#"#[cfg(feature = "hydrate")]"#,
        r#"#[cfg(feature="hydrate")]"#,
        r#"#[cfg(feature  =  "hydrate")]"#,
        "#[cfg(any(feature =\t\"hydrate\", test))]",
        r#"#[cfg_attr( feature = "hydrate" , derive(Debug))]"#,
        r#"#[cfg(not(feature="hydrate"))]"#,
    ] {
        assert!(names_feature(line, "hydrate"), "{line}");
    }
    for line in [
        r#"#[cfg(feature = "browser")]"#,
        r#"#[cfg(feature = "csr")]"#,
        "// hydrate with feature flags",
    ] {
        assert!(!names_feature(line, "hydrate"), "{line}");
    }
}
