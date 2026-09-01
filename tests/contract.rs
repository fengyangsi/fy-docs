//! Static contract tests for the `id` and `class` surface of a generated page.
//!
//! No browser and no typst process runs here. The scans read the sources that
//! write and read that surface — `assets/doc.html`, this crate's markup, the
//! fy-spec template, `assets/base.css` and `assets/viewer.js` — and assert both
//! directions the `fy-spec` and `viewer` chapters state: a class nobody writes
//! is dead stylesheet weight, a class nobody reads is a dead hook, and a
//! binding to an element nobody emits cannot exist.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const SHELL: &str = "assets/doc.html";
const SHEET: &str = "assets/base.css";
const VIEWER: &str = "assets/viewer.js";
const TEMPLATE: &str = "docs/fy-spec/lib.typ";

fn read(rel: impl AsRef<Path>) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

/// Concatenation of every Rust source, which is where the chrome markup — the
/// language switcher, the GitHub link, the error page — is written.
fn rust_markup() -> String {
    let mut files = Vec::new();
    collect_rs(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    files.sort();
    files.iter().map(read).collect()
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|err| panic!("read {dir:?}: {err}"));
    for entry in entries {
        let path = entry
            .unwrap_or_else(|err| panic!("entry under {dir:?}: {err}"))
            .path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_'
}

fn is_own_namespace(name: &str) -> bool {
    name.starts_with("fy-") || name.starts_with("doc-")
}

/// Attribute values written as `class="…"`, wherever they stand: HTML, Rust raw
/// strings, and the `innerHTML` strings inside `viewer.js` all use this form.
fn html_classes(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut after = text;
    while let Some(at) = after.find("class=\"") {
        after = &after[at + 7..];
        if let Some(value) = after.split('"').next() {
            names.extend(value.split_whitespace().map(str::to_string));
        }
    }
    names
}

fn html_ids(text: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let mut after = text;
    while let Some(at) = after.find("id=\"") {
        after = &after[at + 4..];
        if let Some(value) = after.split('"').next() {
            ids.insert(value.to_string());
        }
    }
    ids
}

/// Quoted literals of a script or template source. Comments and regular
/// expression literals are skipped, so neither can smuggle a quote into a scan.
fn literals(src: &str) -> Vec<String> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let mut last = ' ';
    while i < chars.len() {
        match chars[i] {
            '/' if chars.get(i + 1) == Some(&'/') => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '/' if chars.get(i + 1) == Some(&'*') => {
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
            }
            '/' if starts_a_regex(last) => {
                i += 1;
                let mut in_class = false;
                while i < chars.len() {
                    match chars[i] {
                        '\\' => i += 1,
                        '[' => in_class = true,
                        ']' => in_class = false,
                        '/' if !in_class => break,
                        _ => {}
                    }
                    i += 1;
                }
                last = '/';
            }
            quote @ ('\'' | '"') => {
                let begin = i + 1;
                i += 1;
                while i < chars.len() && chars[i] != quote {
                    if chars[i] == '\\' {
                        i += 1;
                    }
                    i += 1;
                }
                out.push(chars[begin..i.min(chars.len())].iter().collect());
                i += 1;
            }
            c => {
                if !c.is_whitespace() {
                    last = c;
                }
                i += 1;
            }
        }
    }
    out
}

/// A `/` following an operator or delimiter opens a literal; after a value it divides.
fn starts_a_regex(last: char) -> bool {
    matches!(
        last,
        '(' | ','
            | '='
            | ':'
            | '['
            | '!'
            | '&'
            | '|'
            | '?'
            | '{'
            | '}'
            | ';'
            | '+'
            | '-'
            | '*'
            | '%'
            | '<'
            | '>'
            | '~'
            | '^'
    )
}

/// Every `.class` token inside a selector.
fn selector_classes(selector: &str) -> Vec<String> {
    let chars: Vec<char> = selector.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let mut in_attribute = false;
    while i < chars.len() {
        match chars[i] {
            '[' => in_attribute = true,
            ']' => in_attribute = false,
            '.' if !in_attribute => {
                let begin = i + 1;
                while i + 1 < chars.len() && is_name_char(chars[i + 1]) {
                    i += 1;
                }
                if begin <= i {
                    out.push(chars[begin..=i].iter().collect());
                }
            }
            _ => {}
        }
        i += 1;
    }
    out
}

/// Class names the stylesheet keys on, limited to this project's own namespaces:
/// palette classes on the root element are shared with typst and mdBook themes.
fn styled_classes(css: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for selector in rule_preludes(css) {
        names.extend(
            selector_classes(&selector)
                .into_iter()
                .filter(|n| is_own_namespace(n)),
        );
    }
    names
}

/// Selector preludes only. Declarations and at-rule preludes name no class, so
/// text immediately before a `{` that is not an at-rule is a selector list.
fn rule_preludes(css: &str) -> Vec<String> {
    let chars: Vec<char> = css.chars().collect();
    let mut preludes = Vec::new();
    let mut buffer = String::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '/' if chars.get(i + 1) == Some(&'*') => {
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 1;
            }
            '{' => {
                let prelude = buffer.trim().to_string();
                if !prelude.starts_with('@') {
                    preludes.push(prelude);
                }
                buffer.clear();
            }
            '}' | ';' => buffer.clear(),
            c => buffer.push(c),
        }
        i += 1;
    }
    preludes
}

/// One `class:` term of the typst template: a literal, and the variable it is
/// concatenated with when the term reads `"fy-" + kind`.
fn typst_terms(text: &str) -> Vec<(String, Option<String>)> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '"' {
            i += 1;
            continue;
        }
        let begin = i + 1;
        i += 1;
        while i < chars.len() && chars[i] != '"' {
            i += 1;
        }
        let literal: String = chars[begin..i.min(chars.len())].iter().collect();
        i += 1;
        let mut j = i;
        while chars.get(j) == Some(&' ') || chars.get(j) == Some(&'\t') {
            j += 1;
        }
        if chars.get(j) == Some(&'+') {
            j += 1;
            while chars.get(j) == Some(&' ') || chars.get(j) == Some(&'\t') {
                j += 1;
            }
            let mut variable = String::new();
            while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                variable.push(chars[j]);
                j += 1;
            }
            if !variable.is_empty() {
                out.push((literal, Some(variable)));
                i = j;
                continue;
            }
        }
        out.push((literal, None));
    }
    out
}

/// Every class the template emits, expanding a `"fy-" + kind` term against the
/// whole value set of that variable, so a new kind needs no edit here.
fn template_classes(template: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut after = template;
    while let Some(at) = after.find("class:") {
        let prelude = class_prelude(&after[at + 6..]);
        for (literal, variable) in typst_terms(&prelude) {
            match variable {
                Some(variable) => names.extend(
                    bound_values(template, &variable)
                        .into_iter()
                        .map(|value| format!("{literal}{value}")),
                ),
                None => {
                    names.insert(literal);
                }
            };
        }
        after = &after[at + 6..];
    }
    names
}

/// The value of one `class:` argument: a parenthesised list, or a single term.
fn class_prelude(tail: &str) -> String {
    let trimmed = tail.trim_start();
    if trimmed.starts_with('(') {
        return trimmed[..trimmed.find(')').unwrap_or(trimmed.len())].to_string();
    }
    let cut = trimmed.find([',', ')', '\n']).unwrap_or(trimmed.len());
    trimmed[..cut].to_string()
}

/// Every literal the template can bind to a variable: a named argument such as
/// `kind: "logic"`, or the branches of `let state = if pending { … }`.
fn bound_values(template: &str, variable: &str) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    for needle in [format!("{variable}:"), format!("let {variable} =")] {
        for (at, _) in template.match_indices(&needle) {
            let tail = &template[at..];
            let line = &tail[..tail.find('\n').unwrap_or(tail.len())];
            values.extend(typst_terms(line).into_iter().map(|(value, _)| value));
        }
    }
    values
}

/// Classes `viewer.js` writes: an element it builds itself names its class in a
/// literal. A literal that is an element id belongs to `every_bound_id_is_emitted`.
fn runtime_classes(js: &str) -> BTreeSet<String> {
    let ids = bound_ids(js);
    literals(js)
        .into_iter()
        .flat_map(|value| {
            value
                .split_whitespace()
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|token| {
            token.starts_with("fy-") && token.chars().all(is_name_char) && !ids.contains(token)
        })
        .collect()
}

/// Selectors `viewer.js` matches against.
fn queried_classes(js: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for literal in literals(js) {
        names.extend(
            selector_classes(&literal)
                .into_iter()
                .filter(|name| is_own_namespace(name)),
        );
    }
    names
}

/// Ids handed to `$()` or `getElementById()`, i.e. the elements bound.
fn bound_ids(js: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for needle in ["$(", "getElementById("] {
        let mut after = js;
        while let Some(at) = after.find(needle) {
            after = &after[at + needle.len()..];
            let argument = &after[..after.find([')', '\n']).unwrap_or(after.len())];
            ids.extend(
                literals(argument)
                    .into_iter()
                    .filter(|id| !id.contains(' ')),
            );
        }
    }
    ids
}

/// Every class name written by every writer of the surface.
fn emitted_classes() -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    names.extend(html_classes(&read(SHELL)));
    names.extend(html_classes(&rust_markup()));
    names.extend(html_classes(&read(VIEWER)));
    names.extend(template_classes(&read(TEMPLATE)));
    names.extend(runtime_classes(&read(VIEWER)));
    names.retain(|name| is_own_namespace(name));
    names
}

#[test]
fn the_scans_see_the_surface() {
    // Every contract assertion passes on an empty scan, so pin each collector
    // to a name its writer is known to contribute.
    let template = read(TEMPLATE);
    let kinds = bound_values(&template, "kind");
    for expected in ["contract", "invariant", "example", "note", "motion"] {
        assert!(kinds.contains(expected), "template kinds: {kinds:?}");
    }
    assert_eq!(
        bound_values(&template, "state"),
        ["done".to_string(), "pending".to_string()]
            .into_iter()
            .collect::<BTreeSet<String>>(),
        "template badge states"
    );
    let emitted = emitted_classes();
    for expected in ["fy-shell", "fy-box", "fy-chapter", "fy-lang-item"] {
        assert!(emitted.contains(expected), "emitted classes: {emitted:?}");
    }
    assert!(styled_classes(&read(SHEET)).contains("fy-doc"));
    assert!(bound_ids(&read(VIEWER)).contains("doc-body"));
}

#[test]
fn every_bound_id_is_emitted() {
    let mut provided = html_ids(&read(SHELL));
    provided.extend(html_ids(&rust_markup()));
    let bound = bound_ids(&read(VIEWER));
    let missing: Vec<&String> = bound.iter().filter(|id| !provided.contains(*id)).collect();
    assert!(
        missing.is_empty(),
        "viewer.js binds ids no markup emits: {missing:?}"
    );
}

#[test]
fn no_rule_styles_a_class_nothing_emits() {
    let emitted = emitted_classes();
    let styled = styled_classes(&read(SHEET));
    let dead: Vec<&String> = styled
        .iter()
        .filter(|class| !emitted.contains(*class))
        .collect();
    assert!(
        dead.is_empty(),
        "base.css styles classes nothing emits: {dead:?}"
    );
}

#[test]
fn no_emitted_class_goes_unread() {
    let mut readers = styled_classes(&read(SHEET));
    readers.extend(queried_classes(&read(VIEWER)));
    let emitted = emitted_classes();
    let dead: Vec<&String> = emitted
        .iter()
        .filter(|class| !readers.contains(*class))
        .collect();
    assert!(
        dead.is_empty(),
        "markup carries classes no rule styles and no selector reads: {dead:?}"
    );
}

#[test]
fn every_template_kind_is_styled() {
    let template = read(TEMPLATE);
    let styled = styled_classes(&read(SHEET));
    let mut missing = Vec::new();
    for kind in bound_values(&template, "kind") {
        let class = format!("fy-{kind}");
        if !styled.contains(&class) {
            missing.push(class);
        }
    }
    for state in bound_values(&template, "state") {
        let class = format!("fy-badge-{state}");
        if !styled.contains(&class) {
            missing.push(class);
        }
    }
    assert!(
        missing.is_empty(),
        "the template can emit classes base.css does not style: {missing:?}"
    );
}
