//! The badge is the deploy's, not the image's (task 028): nothing in the build reads git.
//! Not caught: the same capture under another name; these guard the words the old one used.
#![cfg(not(target_arch = "wasm32"))]
use std::{
    fs,
    path::{Path, PathBuf},
};
const FORBIDDEN: [&str; 7] = [
    "SEARCH_FRONT_GIT",
    "BUILD_BRANCH",
    "BUILD_SHA",
    "git_info",
    "capture_git_info",
    "rev-parse",
    "commondir",
];
fn app_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}
fn rust_files(dir: &Path, into: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("readable") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            rust_files(&path, into);
        } else if path.extension().is_some_and(|e| e == "rs") {
            into.push(path);
        }
    }
}
fn offences(path: &Path) -> Vec<String> {
    let text = fs::read_to_string(path).expect("readable");
    FORBIDDEN
        .iter()
        .filter(|w| text.contains(**w))
        .map(|w| format!("{} mentions `{w}`", path.display()))
        .collect()
}
#[test]
fn the_build_script_and_the_dockerfile_do_not_read_git() {
    let found: Vec<String> = [app_dir().join("build.rs"), app_dir().join("../Dockerfile")]
        .iter()
        .flat_map(|p| offences(p))
        .collect();
    assert_eq!(found, Vec::<String>::new(), "the image must not read git");
}
#[test]
fn no_source_file_reads_the_build_time_git() {
    let mut files = Vec::new();
    rust_files(&app_dir().join("src"), &mut files);
    files.sort();
    let found: Vec<String> = files.iter().flat_map(|f| offences(f)).collect();
    assert_eq!(
        found,
        Vec::<String>::new(),
        "no source reads the build's git"
    );
}
#[test]
fn the_git_info_module_is_gone() {
    assert!(
        !app_dir().join("src/git_info.rs").exists(),
        "git_info.rs must not exist"
    );
}
