//! The image build's context (task 007, slice S2): what `Dockerfile` copies
//! is enough for Cargo to load the workspace, and the build needs no
//! credential.
//!
//! No docker on the laptop, so the build context is simulated: the
//! Dockerfile's own `COPY` lines (the ones without `--from`) are replayed
//! from the three sibling checkouts (`search-front/`, `ui-lib/`, `api/`,
//! the folder CI builds in) into a temp dir, then `cargo metadata --locked
//! --offline` loads the workspace there. A file the build needs but no
//! `COPY` names fails here, not on the runner.
//!
//! The property the credential-free build rests on: `Cargo.lock` holds no
//! git source. The two private dependencies are `[patch]`ed to the sibling
//! checkouts and the lock records them as path packages, so Cargo has
//! nothing to fetch from GitHub.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository root (this test's package is `app/`).
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root")
        .to_path_buf()
}

/// The Dockerfile's text.
fn dockerfile() -> String {
    fs::read_to_string(repo_root().join("Dockerfile")).expect("a Dockerfile at the repository root")
}

/// Whether a `Cargo.lock` records a package fetched from git.
fn lock_has_git_source(lock: &str) -> bool {
    lock.lines()
        .any(|line| line.trim_start().starts_with("source = \"git+"))
}

/// One instruction per entry: line continuations (`\`) joined, comments and
/// blank lines dropped. Heredoc bodies (`RUN <<EOF`) are not parsed; the
/// Dockerfile avoids them.
fn instructions(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for raw in text.lines() {
        let line = raw.trim();
        if current.is_empty() && (line.is_empty() || line.starts_with('#')) {
            continue;
        }
        if let Some(stripped) = line.strip_suffix('\\') {
            current.push_str(stripped.trim_end());
            current.push(' ');
        } else {
            current.push_str(line);
            out.push(std::mem::take(&mut current));
        }
    }
    out
}

/// `(sources, destination)` of a `COPY` that reads the build context.
fn context_copy(instruction: &str) -> Option<(Vec<String>, String)> {
    let rest = instruction.strip_prefix("COPY ")?;
    let words: Vec<&str> = rest.split_whitespace().collect();
    if words.iter().any(|w| w.starts_with("--from")) {
        return None;
    }
    let args: Vec<&str> = words.into_iter().filter(|w| !w.starts_with("--")).collect();
    let (dest, sources) = args.split_last()?;
    Some((
        sources.iter().map(ToString::to_string).collect(),
        (*dest).to_string(),
    ))
}

/// Copy a file or a directory tree.
fn copy_tree(from: &Path, to: &Path) {
    if from.is_dir() {
        fs::create_dir_all(to).expect("create a directory");
        for entry in fs::read_dir(from).expect("read a directory") {
            let entry = entry.expect("a directory entry");
            copy_tree(&entry.path(), &to.join(entry.file_name()));
        }
    } else {
        fs::create_dir_all(to.parent().expect("a parent")).expect("create a parent");
        fs::copy(from, to).unwrap_or_else(|e| panic!("copy {}: {e}", from.display()));
    }
}

/// The build context folder: the one holding `search-front/`, `ui-lib/` and
/// `api/`. This checkout stands for `search-front/` (a worktree has another
/// name), the siblings are beside it.
fn context_source(name: &str) -> PathBuf {
    let root = repo_root();
    match name {
        "search-front" => root,
        other => root.join("..").join(other),
    }
}

/// Resolve a `COPY` source (`ui-lib/src`, `search-front/app`) in the real tree.
fn resolve_source(source: &str) -> PathBuf {
    let (first, rest) = source.split_once('/').unwrap_or((source, ""));
    context_source(first).join(rest)
}

/// One `COPY` of the context into the stage's file system `dir`.
fn replay_copy(dir: &Path, workdir: &Path, sources: &[String], dest: &str) {
    let dest_abs = workdir.join(dest);
    let inside = dir.join(dest_abs.strip_prefix("/").expect("an absolute destination"));
    for source in sources {
        let from = resolve_source(source);
        assert!(
            from.exists(),
            "COPY {source}: not in the sibling checkouts ({})",
            from.display()
        );
        let target = if dest.ends_with('/') {
            inside.join(Path::new(source).file_name().expect("a file name"))
        } else {
            inside.clone()
        };
        copy_tree(&from, &target);
    }
}

/// Replay the Dockerfile's `COPY` lines of the `builder` stage into `dir`
/// (the stage's file system), following `WORKDIR`.
fn replay_builder(dir: &Path) {
    let mut stage = String::new();
    let mut workdir = PathBuf::from("/");
    for instruction in instructions(&dockerfile()) {
        if let Some(from) = instruction.strip_prefix("FROM ") {
            stage = from
                .split_whitespace()
                .last()
                .unwrap_or_default()
                .to_string();
            workdir = PathBuf::from("/");
        } else if let Some(wd) = instruction.strip_prefix("WORKDIR ") {
            workdir = workdir.join(wd.trim());
        } else if stage == "builder"
            && let Some((sources, dest)) = context_copy(&instruction)
        {
            replay_copy(dir, &workdir, &sources, &dest);
        }
    }
}

fn siblings_present() -> bool {
    ["ui-lib", "api"]
        .iter()
        .all(|name| context_source(name).is_dir())
}

#[test]
fn the_lock_holds_no_git_source() {
    let lock = fs::read_to_string(repo_root().join("Cargo.lock")).expect("Cargo.lock");
    assert!(
        !lock_has_git_source(&lock),
        "Cargo.lock records a git source: the image build would need a GitHub credential (task 007, R1)"
    );
}

#[test]
fn a_git_source_in_a_lock_is_seen() {
    let lock = "[[package]]\nname = \"x\"\nversion = \"1.0.0\"\nsource = \"git+ssh://git@github.com/PaliCanon/ui-lib.git?branch=main#abc\"\n";
    assert!(lock_has_git_source(lock));
    assert!(lock_has_git_source(
        "  source = \"git+https://github.com/a/b\""
    ));
    let registry = "source = \"registry+https://github.com/rust-lang/crates.io-index\"\n";
    assert!(!lock_has_git_source(registry));
}

#[test]
fn the_copied_files_are_enough_for_cargo_to_load_the_workspace() {
    if !siblings_present() {
        eprintln!("skipped: ../ui-lib or ../api is not beside this checkout (CI checks them out)");
        return;
    }
    let dir =
        std::env::temp_dir().join(format!("search-front-image-context-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    replay_builder(&dir);
    let manifest = dir.join("src/search-front/Cargo.toml");
    assert!(
        manifest.is_file(),
        "the builder stage copies no search-front/Cargo.toml under /src/search-front"
    );
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    // `cargo metadata` resolves every target (windows, UEFI: `r-efi`, ...),
    // while a runner that only built and tested for its host has fetched the
    // host's crates (task 007 F2). Fetch what the lock needs for every target
    // first, as the image build's own network step does, so the `--offline`
    // load below judges the copied files and not the state of `CARGO_HOME`.
    // No git source exists (the test above), so no credential is involved.
    let fetch = Command::new(&cargo)
        .args(["fetch", "--locked", "--manifest-path"])
        .arg(&manifest)
        .env("GIT_SSH_COMMAND", "/bin/false")
        .output()
        .expect("run cargo fetch");
    assert!(
        fetch.status.success(),
        "cargo fetch on the replayed context failed:\n{}",
        String::from_utf8_lossy(&fetch.stderr)
    );
    let output = Command::new(&cargo)
        .args([
            "metadata",
            "--locked",
            "--offline",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(&manifest)
        .env("GIT_SSH_COMMAND", "/bin/false")
        .output()
        .expect("run cargo metadata");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let _ = fs::remove_dir_all(&dir);
    assert!(
        output.status.success(),
        "cargo metadata on the replayed context failed:\n{stderr}"
    );
}

/// The relative paths the build reads outside Cargo's view: the `@import` and
/// `@source` of `app/style/main.css` (Tailwind) and the `#[path]` of
/// `app/build.rs`. Cargo's metadata does not follow them; the build does.
fn relative_reads(file: &Path) -> Vec<PathBuf> {
    let text = fs::read_to_string(file).unwrap_or_else(|e| panic!("read {}: {e}", file.display()));
    let base = file.parent().expect("a directory");
    let mut reads = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        let quoted = if line.starts_with("@import")
            || line.starts_with("@source")
            || line.starts_with("#[path")
        {
            line.split('"').nth(1)
        } else {
            None
        };
        if let Some(path) = quoted.filter(|p| p.starts_with("..")) {
            reads.push(base.join(path));
        }
    }
    reads
}

#[test]
fn the_files_the_build_reads_outside_cargo_are_copied() {
    if !siblings_present() {
        eprintln!("skipped: ../ui-lib or ../api is not beside this checkout (CI checks them out)");
        return;
    }
    let dir = std::env::temp_dir().join(format!("search-front-image-reads-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    replay_builder(&dir);
    let app = dir.join("src/search-front/app");
    let mut missing = Vec::new();
    let mut seen = 0;
    for file in [app.join("style/main.css"), app.join("build.rs")] {
        for read in relative_reads(&file) {
            seen += 1;
            if !read.exists() {
                missing.push(format!(
                    "{} reads {}",
                    file.strip_prefix(&dir).unwrap().display(),
                    read.strip_prefix(&dir).unwrap_or(&read).display()
                ));
            }
        }
    }
    let _ = fs::remove_dir_all(&dir);
    assert!(
        seen >= 3,
        "expected the theme import, the source scan and the placeholders path, saw {seen}"
    );
    assert!(
        missing.is_empty(),
        "the image does not copy what the build reads: {missing:?}"
    );
}

#[test]
fn the_image_runs_in_production_mode_as_a_non_root_user() {
    let text = dockerfile();
    let all = instructions(&text);
    assert!(
        all.iter().any(|i| i.contains("LEPTOS_ENV=PROD")),
        "the image sets LEPTOS_ENV=PROD"
    );
    let user = all
        .iter()
        .rev()
        .find_map(|i| i.strip_prefix("USER "))
        .expect("a USER instruction");
    assert!(
        user.trim() != "root" && user.trim() != "0",
        "the image must not run as root, got USER {user}"
    );
    assert!(
        all.iter().any(|i| i.starts_with("CMD ")),
        "a CMD starts the server"
    );
}

#[test]
fn the_dockerfile_holds_no_credential() {
    let text = dockerfile();
    let all = instructions(&text);
    let sensitive = ["TOKEN", "SECRET", "PASSWORD", "KEY", "CREDENTIAL", "AUTH"];
    for instruction in &all {
        if let Some(rest) = instruction
            .strip_prefix("ARG ")
            .or_else(|| instruction.strip_prefix("ENV "))
        {
            let upper = rest.to_uppercase();
            let name = upper.split(['=', ' ']).next().unwrap_or_default();
            assert!(
                !sensitive.iter().any(|s| name.contains(s)),
                "an ARG/ENV named like a credential: {instruction}"
            );
        }
        assert!(
            !instruction.contains("type=secret"),
            "a BuildKit secret mount: {instruction}"
        );
        assert!(
            !instruction.contains("type=ssh"),
            "an ssh mount: {instruction}"
        );
        assert!(
            !instruction.contains("insteadOf"),
            "a git URL rewrite: {instruction}"
        );
        assert!(
            !instruction.contains("x-access-token"),
            "a token in a URL: {instruction}"
        );
        assert!(
            !instruction.contains("GIT_SSH_COMMAND"),
            "a git transport override: {instruction}"
        );
    }
}

#[test]
fn the_build_fails_when_the_lock_gains_a_git_source() {
    let text = dockerfile();
    let guarded = instructions(&text)
        .iter()
        .any(|i| i.starts_with("RUN ") && i.contains("git+") && i.contains("Cargo.lock"));
    assert!(
        guarded,
        "a RUN must fail the build when Cargo.lock holds a git source"
    );
}

#[test]
fn the_server_runs_the_smoke_test_with_its_own_throwaway_secret() {
    let all = instructions(&dockerfile());
    let smoke = all
        .iter()
        .rposition(|i| i.starts_with("RUN ") && i.contains("smoke.sh"));
    let last_run = all.iter().rposition(|i| i.starts_with("RUN "));
    assert!(smoke.is_some(), "a RUN runs scripts/smoke.sh");
    assert_eq!(smoke, last_run, "the smoke test is the last RUN");
}

/// `scripts/smoke.sh BINARY` makes its own throwaway secret, a real source
/// and a free port: the call is exactly `RUN ./smoke.sh ./search_front`, with
/// no `ORIGIN_SECRET=` prefix (the script would overwrite it anyway).
#[test]
fn the_smoke_call_has_the_form_the_script_takes() {
    let all = instructions(&dockerfile());
    let smoke = all
        .iter()
        .rfind(|i| i.starts_with("RUN ") && i.contains("smoke.sh"))
        .expect("a RUN runs smoke.sh");
    assert_eq!(
        smoke, "RUN ./smoke.sh ./search_front",
        "smoke.sh takes the binary and nothing else: no env prefix, no --port"
    );
}

/// 028: the release is not in the image. The builder takes no build argument
/// but the `cargo-leptos` version, and exports no branch or commit: the badge
/// is the deploy's (`SEARCH_FRONT_ENV` / `SEARCH_FRONT_SHA` on the revision).
#[test]
fn the_builder_takes_no_build_args() {
    let instructions = instructions(&dockerfile());
    let args: Vec<&str> = instructions
        .iter()
        .filter_map(|instruction| instruction.strip_prefix("ARG "))
        .map(|arg| arg.split('=').next().unwrap_or_default().trim())
        .collect();
    assert_eq!(
        args,
        ["CARGO_LEPTOS"],
        "the image takes no release argument"
    );
    for instruction in instructions.iter().filter(|i| i.starts_with("ENV ")) {
        assert!(
            !instruction.contains("BUILD_BRANCH") && !instruction.contains("BUILD_SHA"),
            "no ENV carries the build's branch or commit: {instruction}"
        );
    }
}

/// Task 007 F1: `scripts/smoke.sh` reads `token-pattern` from its own folder.
/// The runtime stage once copied `smoke.sh` alone, so the last RUN of the
/// image build ran without the pattern. Whatever stage runs `smoke.sh` must
/// hold `token-pattern` in the same folder: the builder stage hands both to
/// `/out`, and the runtime stage copies both into one destination.
#[test]
fn the_runtime_stage_holds_the_token_pattern_beside_smoke_sh() {
    let mut stage = String::new();
    let mut smoke_copies = 0;
    for instruction in instructions(&dockerfile()) {
        if let Some(from) = instruction.strip_prefix("FROM ") {
            stage = from
                .split_whitespace()
                .last()
                .unwrap_or_default()
                .to_string();
        }
        if stage == "builder" && instruction.contains("cp ") && instruction.contains("smoke.sh") {
            assert!(
                instruction.contains("token-pattern"),
                "the builder hands smoke.sh to /out without token-pattern: {instruction}"
            );
        }
        if stage != "builder"
            && stage != "tools"
            && instruction.starts_with("COPY --from=builder")
            && instruction.contains("smoke.sh")
        {
            smoke_copies += 1;
            assert!(
                instruction.contains("/out/token-pattern"),
                "the runtime stage copies smoke.sh without token-pattern: {instruction}"
            );
        }
    }
    assert_eq!(smoke_copies, 1, "the runtime stage copies smoke.sh once");
}
