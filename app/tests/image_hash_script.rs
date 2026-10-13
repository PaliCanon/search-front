//! `scripts/image-hash.sh` (task 007, slice S5): the image tag is a hash of
//! what shapes the image, computed once in CI after the two sibling
//! checkouts. Three temp git repositories stand for `search-front/`,
//! `ui-lib/` and `api/`.
//!
//! The inputs are the tracked files of `app`, `Cargo.toml`, `Cargo.lock`,
//! `Dockerfile` and `scripts` (the Dockerfile's text and the smoke test are
//! in the hash, so a fix to either changes the tag), plus the HEAD of
//! `ui-lib` and of `api`. Nothing else: `infra/`, `.task/`, the README.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A temp directory holding the three repositories, removed on drop.
struct Workspace {
    dir: PathBuf,
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["-c", "user.name=test", "-c", "user.email=test@example.org"])
        .args(args)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn write(repo: &Path, file: &str, text: &str) {
    let path = repo.join(file);
    fs::create_dir_all(path.parent().expect("a parent")).expect("create a directory");
    fs::write(path, text).expect("write a file");
}

/// Commit everything in `repo` (the test repositories only).
fn commit(repo: &Path, message: &str) {
    git(repo, &["add", "."]);
    git(repo, &["commit", "-q", "-m", message]);
}

impl Workspace {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("image-hash-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let sf = dir.join("search-front");
        for repo in ["search-front", "ui-lib", "api"] {
            fs::create_dir_all(dir.join(repo)).expect("create a repository");
            git(&dir.join(repo), &["init", "-q", "-b", "main"]);
        }
        let real = Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts/image-hash.sh");
        let script = fs::read_to_string(&real).expect("scripts/image-hash.sh");
        write(&sf, "scripts/image-hash.sh", &script);
        write(&sf, "scripts/smoke.sh", "echo smoke\n");
        write(&sf, "app/src/lib.rs", "pub fn a() {}\n");
        write(&sf, "app/Cargo.toml", "[package]\nname = \"a\"\n");
        write(&sf, "Cargo.toml", "[workspace]\n");
        write(&sf, "Cargo.lock", "# lock\n");
        write(&sf, "Dockerfile", "FROM scratch\n");
        write(&sf, "infra/main.tf", "# infra\n");
        write(&sf, ".task/007.md", "# task\n");
        write(&sf, "README.md", "# readme\n");
        commit(&sf, "search-front");
        write(&dir.join("ui-lib"), "src/lib.rs", "// ui\n");
        commit(&dir.join("ui-lib"), "ui-lib");
        write(&dir.join("api"), "src/lib.rs", "// api\n");
        commit(&dir.join("api"), "api");
        Self { dir }
    }

    fn repo(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    /// The script's output and exit success.
    fn run(&self) -> (bool, String) {
        let output = Command::new("bash")
            .arg(self.repo("search-front").join("scripts/image-hash.sh"))
            .arg(self.repo("ui-lib"))
            .arg(self.repo("api"))
            .output()
            .expect("run the script");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        (output.status.success(), text.trim().to_string())
    }

    fn hash(&self) -> String {
        let (ok, out) = self.run();
        assert!(ok, "the script failed: {out}");
        out
    }

    /// Change one file of a repository and commit it.
    fn change(&self, repo: &str, file: &str, text: &str) {
        write(&self.repo(repo), file, text);
        commit(&self.repo(repo), "change");
    }
}

#[test]
fn the_hash_is_h_and_sixteen_hex() {
    let w = Workspace::new("shape");
    let hash = w.hash();
    let hex = hash.strip_prefix("h-").expect("an h- prefix");
    assert_eq!(hex.len(), 16, "{hash}");
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit()), "{hash}");
}

#[test]
fn the_same_inputs_give_the_same_hash() {
    let w = Workspace::new("same");
    assert_eq!(w.hash(), w.hash());
}

#[test]
fn each_input_changes_the_hash() {
    let cases: [(&str, &str, &str); 7] = [
        ("search-front", "app/src/lib.rs", "pub fn b() {}\n"),
        ("search-front", "Dockerfile", "FROM scratch\nRUN true\n"),
        ("search-front", "scripts/smoke.sh", "echo changed\n"),
        ("search-front", "Cargo.lock", "# lock 2\n"),
        (
            "search-front",
            "Cargo.toml",
            "[workspace]\nresolver = \"3\"\n",
        ),
        ("ui-lib", "src/lib.rs", "// ui 2\n"),
        ("api", "src/lib.rs", "// api 2\n"),
    ];
    for (i, (repo, file, text)) in cases.iter().enumerate() {
        let w = Workspace::new(&format!("input-{i}"));
        let before = w.hash();
        w.change(repo, file, text);
        assert_ne!(before, w.hash(), "{repo}/{file} must change the hash");
    }
}

#[test]
fn files_outside_the_inputs_do_not() {
    for (i, (file, text)) in [
        ("infra/main.tf", "# infra 2\n"),
        (".task/007.md", "# task 2\n"),
        ("README.md", "# readme 2\n"),
    ]
    .iter()
    .enumerate()
    {
        let w = Workspace::new(&format!("outside-{i}"));
        let before = w.hash();
        w.change("search-front", file, text);
        assert_eq!(before, w.hash(), "{file} is not an input");
    }
}

#[test]
fn an_untracked_file_is_not_an_input() {
    let w = Workspace::new("untracked");
    let before = w.hash();
    write(
        &w.repo("search-front"),
        "app/src/scratch.rs",
        "// not committed\n",
    );
    assert_eq!(before, w.hash());
}

#[test]
fn a_missing_sibling_fails_with_a_message() {
    let w = Workspace::new("missing");
    fs::remove_dir_all(w.repo("api").join(".git")).expect("remove .git");
    let (ok, out) = w.run();
    assert!(!ok, "a sibling that is no git checkout must fail: {out}");
    assert!(out.contains("api"), "the message names the sibling: {out}");
}

#[test]
fn missing_arguments_fail_with_a_usage_line() {
    let w = Workspace::new("usage");
    let output = Command::new("bash")
        .arg(w.repo("search-front").join("scripts/image-hash.sh"))
        .output()
        .expect("run the script");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage"));
}
