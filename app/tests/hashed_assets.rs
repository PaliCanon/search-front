//! Task 027: the bundle is hash-named, so a deploy never pairs an old cached
//! glue with a new wasm. The render test is `app.rs`'s; these are the two
//! files that make the shell's hashed names resolvable at run time: the
//! workspace turns `hash-files` on, and the image ships the hash file
//! `cargo-leptos` writes, beside the binary, with the switch on.

use std::fs;
use std::path::{Path, PathBuf};

/// The repository root (this test's package is `app/`).
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root")
        .to_path_buf()
}

fn read(name: &str) -> String {
    fs::read_to_string(repo_root().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

#[test]
fn the_workspace_turns_hash_files_on() {
    let manifest = read("Cargo.toml");
    assert!(
        manifest
            .lines()
            .any(|line| line.split('#').next().unwrap_or("").replace(' ', "") == "hash-files=true"),
        "Cargo.toml's [[workspace.metadata.leptos]] has no `hash-files = true`"
    );
}

#[test]
fn the_image_ships_the_hash_file_beside_the_binary_with_the_switch_on() {
    let dockerfile = read("Dockerfile");
    let code: String = dockerfile
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    // The builder keeps hash.txt, the runtime stage copies it next to the
    // server (the server reads it next to its own executable).
    assert!(
        code.contains("app/target/release/hash.txt"),
        "the builder does not keep app/target/release/hash.txt (where cargo-leptos writes it for a single-member workspace)"
    );
    assert!(
        code.lines()
            .any(|l| l.starts_with("COPY --from=builder") && l.contains("/out/hash.txt")),
        "the runtime stage does not copy hash.txt next to the binary"
    );
    assert!(
        code.contains("LEPTOS_HASH_FILES=true"),
        "the runtime stage does not set LEPTOS_HASH_FILES=true"
    );
}
