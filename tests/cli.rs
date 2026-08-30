//! Integration tests running the real `cargo-fy-docs` binary end to end.
//!
//! Tests that compile documents need `typst` on PATH; they skip (with a
//! note) on machines without it so the suite stays green everywhere.

use std::process::Command;

fn typst_available() -> bool {
    Command::new("typst")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cargo-fy-docs"))
}

#[test]
fn build_succeeds_on_a_fresh_project() {
    if !typst_available() {
        eprintln!("skipping: typst is not on PATH");
        return;
    }
    let temp = tempfile::tempdir().unwrap();

    let init = binary()
        .arg("init")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(init.status.success(), "init failed: {init:?}");

    let build = binary()
        .arg("build")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );

    let index = temp.path().join("docs/target/index.html");
    let page = std::fs::read_to_string(&index).unwrap();
    assert!(
        !page.contains("fy-error"),
        "fresh build must not be an error page"
    );
}

#[test]
fn build_exits_nonzero_and_writes_an_error_page_on_broken_source() {
    if !typst_available() {
        eprintln!("skipping: typst is not on PATH");
        return;
    }
    let temp = tempfile::tempdir().unwrap();

    binary()
        .arg("init")
        .current_dir(temp.path())
        .output()
        .unwrap();
    std::fs::write(
        temp.path().join("docs/main.typ"),
        "#import \"fy-spec/lib.typ\": *\n\n#this-does-not-exist\n",
    )
    .unwrap();

    let build = binary()
        .arg("build")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert_eq!(
        build.status.code(),
        Some(1),
        "broken docs must fail the build"
    );
    let stderr = String::from_utf8_lossy(&build.stderr);
    assert!(
        stderr.contains("FAILED"),
        "stderr should say FAILED: {stderr}"
    );

    let page = std::fs::read_to_string(temp.path().join("docs/target/index.html")).unwrap();
    assert!(page.contains("fy-error"), "the error page must render");
}

#[test]
fn vendor_check_flags_a_drifted_template() {
    let temp = tempfile::tempdir().unwrap();

    // No docs/ at all: refuses with init hint.
    let status = binary()
        .arg("vendor")
        .arg("--check")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert_eq!(status.status.code(), Some(1));

    // Fresh init is in sync; a manual edit is flagged as drift.
    binary()
        .arg("init")
        .current_dir(temp.path())
        .output()
        .unwrap();
    let ok = binary()
        .arg("vendor")
        .arg("--check")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(ok.status.success());

    std::fs::write(temp.path().join("docs/fy-spec/lib.typ"), "drifted").unwrap();
    let drifted = binary()
        .arg("vendor")
        .arg("--check")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert_eq!(drifted.status.code(), Some(1));
}
