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

#[test]
fn unknown_lang_filter_fails_instead_of_building_the_default() {
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

    // A typo must not exit 0 with only the default page built.
    let typo = binary()
        .args(["html", "--lang", "zz"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert_eq!(typo.status.code(), Some(1), "an unknown language must fail");
    let stderr = String::from_utf8_lossy(&typo.stderr);
    assert!(stderr.contains("`zz`"), "{stderr}");
    assert!(stderr.contains("default"), "{stderr}");

    // A real language written with the wrong case or separator still resolves.
    let main = std::fs::read_to_string(temp.path().join("docs/main.typ")).unwrap();
    std::fs::create_dir_all(temp.path().join("docs/zh-CN")).unwrap();
    std::fs::write(
        temp.path().join("docs/zh-CN/main.typ"),
        main.replace("\"fy-spec/lib.typ\"", "\"../fy-spec/lib.typ\""),
    )
    .unwrap();
    let built = binary()
        .args(["html", "--lang", "ZH_cn"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert!(temp.path().join("docs/target/index_zh-CN.html").is_file());
}

#[test]
fn successful_build_surfaces_typst_warnings() {
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
    // `v` carries no meaning in HTML export, so typst warns while still
    // succeeding: exactly the class of signal fy-docs used to discard.
    let entry = temp.path().join("docs/main.typ");
    let mut main = std::fs::read_to_string(&entry).unwrap();
    main.push_str("\n#v(1em)\n");
    std::fs::write(&entry, main).unwrap();

    let build = binary()
        .arg("html")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "a warning must not fail the build: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let stderr = String::from_utf8_lossy(&build.stderr);
    assert!(
        stderr.to_lowercase().contains("warning"),
        "typst warnings must reach stderr: {stderr}"
    );
}

#[test]
fn a_declared_language_reaches_the_page_and_its_labels() {
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
    let entry = temp.path().join("docs/main.typ");
    let main = std::fs::read_to_string(&entry).unwrap();
    std::fs::write(&entry, main.replace(r#"lang: "en""#, r#"lang: "zh-CN""#)).unwrap();

    let build = binary()
        .arg("html")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let page = std::fs::read_to_string(temp.path().join("docs/target/index.html")).unwrap();
    assert!(page.contains(r#"<html lang="zh-CN">"#), "{page}");
    assert!(page.contains(r#"title="主题""#), "{page}");
    // Declaration and export agree, so there is nothing to warn about.
    let stderr = String::from_utf8_lossy(&build.stderr);
    assert!(
        !stderr.contains("typst typesets"),
        "a matching declaration must stay quiet: {stderr}"
    );
}

#[test]
fn a_folder_and_entry_that_disagree_are_reported() {
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
    // The starter entry declares English; moving it under a zh-CN folder makes
    // the two declaration channels disagree about the same document.
    let main = std::fs::read_to_string(temp.path().join("docs/main.typ")).unwrap();
    std::fs::create_dir_all(temp.path().join("docs/zh-CN")).unwrap();
    std::fs::write(
        temp.path().join("docs/zh-CN/main.typ"),
        main.replace("\"fy-spec/lib.typ\"", "\"../fy-spec/lib.typ\""),
    )
    .unwrap();
    std::fs::remove_file(temp.path().join("docs/main.typ")).unwrap();

    let build = binary()
        .arg("html")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "a divergence must not fail the build: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let stderr = String::from_utf8_lossy(&build.stderr);
    assert!(
        stderr.contains("typst typesets `en`") && stderr.contains("reports `zh-CN`"),
        "the drift must name both tags: {stderr}"
    );
    // The folder still decides what the page calls itself.
    let page = std::fs::read_to_string(temp.path().join("docs/target/index_zh-CN.html")).unwrap();
    assert!(page.contains(r#"<html lang="zh-CN">"#), "{page}");
}
