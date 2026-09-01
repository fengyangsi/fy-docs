# Changelog

English | [简体中文](CHANGELOG.zh-CN.md)

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Integration tests in the published crate**: the package `include` list now carries `/tests/**`, so an extracted `.crate` keeps the suite its declared dev-dependencies exist for.
- **Documented language naming rule**: a language folder appends a region or script subtag only when it carries a real difference, `en` stays the neutral English root, and a document is never forked over spelling alone.
- **Language divergence is reported**: when a typst HTML export typesets a different language than fy-docs resolved for that target, the build prints one stderr line naming both tags and the entry path. It is diagnostic only — the resolved tag stays on the page and the exit code does not change.

### Changed
- **Content language is declared, never inferred**: every language target resolves one tag — its language directory name, else the `lang:` argument of its entry `main.typ`, else the fy-spec template default `en` — and that single tag drives `<html lang>` and the toolbar label set. The body is no longer scanned for CJK glyphs, so a Chinese document that declares nothing changes behaviour: a root `docs/main.typ` without `lang:` now gets English chrome, and `lang: "zh-CN"` in the `#show: project_book.with(...)` call is what selects Chinese. In exchange, a Japanese document is no longer announced as `zh-CN` because the two scripts share ideographs, and one document can no longer report one language to the browser and another to its PDF.
- **Unregistered language codes display in BCP 47 shape**: base subtag lowercase, region uppercase, script title-cased, so a `docs/pt_BR/` folder reads `pt-BR` in the language switcher instead of leaking the directory name. fy-docs never invents a language name it cannot know.
- **Specification chapters state current behaviour**: retired-repository notes, upgrade-history framing and design-decision narration were removed from the docs; that history belongs here.
- **Corrected landing-page size invariant**: the documented "under 1KB" was already exceeded (1189 bytes at two languages). The rule is now stated as measured, roughly 60 bytes per language, and `assets.rs` enforces the bound.
- **Internal restructure, identical behaviour**: `compiler` and `project` split into single-concern submodules (`compiler/{mod,typst,extract,warnings,output}`, `project/{mod,lang,cargo_meta,imports,template_args}`), and terminal output moved from `state.rs` into `term.rs`. Startup options are captured once and every generation — including dev-mode rebuilds — reads the captured set; the default command and `build` capture PDFs as always on instead of hardcoding it in one dispatch arm. `dispatch` returns an exit code instead of calling `process::exit` mid-function, one typst HTML export travels as a named `ExtractedPage` instead of a bare string triple, and `SKIN_FILE`/`warnings_note` become `STYLE_FILE`/`format_warnings`. No command, flag, output file name, or exit code changed; tests moved with their functions and every scratch directory is now a RAII `TempDir`.
- **Specification now mirrors the module split**: `docs/` specifies seven chapters — `cli`, `scaffold`, `project`, `compiler`, `page`, `server`, `viewer` — where `scaffold` (the `init` and `vendor` design) and `page` (`src/assets.rs` plus `assets/doc.html`) had no home of their own and their contracts sat inside `cli` and `compiler`. `main.typ` gained a file-to-chapter ownership table, states that an arrow points from a chapter to the chapter consuming its product rather than tracing the Rust `use` graph, and pins the generated shell's `id` and `class` surface as an internal invariant that `viewer.js` may bind without defending against an absent element. Written out for the first time: the fixed dispatch order, the twenty-candidate port range, the error-to-exit-code path, the `/events` baseline frame, the watch set and its `.typ`-only rule, the shared ignore-rule helper, the `typst.css` containment-as-equality merge, and the split that gives asset bytes to `page` and asset file names to `compiler`. No code changed.

### Fixed
- **Page content language**: the root `lang` attribute carried the toolbar chrome language, which exists in English and Chinese only, so every other language announced itself as English to assistive technology and `:lang()` hyphenation. A page now declares its own normalized tag while the chrome still falls back to English.

### Removed
- The `docs/target/` sweep no longer deletes `_poll.js` and `_build`. The current build never writes them; keeping them listed made the tool carry its own upgrade history, and every release could only lengthen that list. `_temp_*.html` intermediates a killed compile leaves behind are still swept.

## [0.1.10] - 2026-08-31

### Added
- **`--open` for `pdf`**: `cargo fy-docs pdf --open` opens the first generated release PDF, matching the option the README already documented for the other commands.

### Changed
- **`--lang` is normalized and strict**: `ZH_CN`, `zh-cn` and `zh_CN` now select the same target, while a filter that matches no language fails with a non-zero exit code and lists the languages the project actually provides instead of quietly building the default target only. Matching stays exact: `--lang zh` does not select `zh-CN`.
- **Typst warnings are surfaced**: a successful compile forwards typst's stderr instead of discarding it, so font substitution and directives dropped by HTML export no longer hide behind a green build. The repeated `unknown font family` reports fold into one deduplicated line, and Windows verbatim path prefixes are stripped from forwarded diagnostics.
- **Language detection is rule-based**: any `docs/` subdirectory carrying its own `main.typ` is a language target except the generated directories, so shared source folders no longer need an entry on a name denylist.
- **Debounced rebuilds are bounded**: a save burst still folds into one rebuild, but the wait is capped at 2s from the first change, so a process writing sources continuously cannot postpone the rebuild forever.

### Fixed
- The absolute-import scan honours both quote styles and `//` comments, so a commented-out `#import` can no longer steer root detection to the wrong ancestor; it also skips `docs/release/` alongside `docs/target/`.
- The `main.typ` version fallback reads uncommented code only and skips a `version:` without a quoted value instead of abandoning the search.
- Every build sweeps `docs/target/` of artifacts fy-docs no longer writes (`_poll.js`, `_build`) and of `_temp_*.html` intermediates left by a killed compile.
- `extract_all_styles` also captures `<style>` tags carrying attributes, which makes multi-language style merging a live path instead of dead code.
- A failing `--with-pdf` stage still leaves the routing landing page in place for pure i18n projects, so the dev server keeps a route for `/`.

### Removed
- Deleted the never-read `Project::entry` field and the superseded `Project::pdf_file_name()`, and gated the test-only `AppState::new` behind `#[cfg(test)]`. Internal items are now `pub(crate)`, since the crate deliberately exposes no library API.

## [0.1.9] - 2026-08-31

### Fixed
- Dev-mode live reload reacts to the first save again: the `/events` stream sends the current build id once as the subscriber baseline (`WatchStream::new`), so opening a page and saving immediately reloads it instead of requiring a second rebuild. This reverts the 0.1.8 `WatchStream::from_changes` change, which suppressed every page's initial baseline frame; the duplicate seed frame it fixed stays fixed.

## [0.1.8] - 2026-08-31

### Fixed
- A partial-failure build no longer blanks `typst.css`: error pages only seed the file when absent, so the combined styles of successfully built languages survive.
- A multi-language project with one surviving language keeps its routing landing page instead of having that page copied to `index.html`.
- A failed build no longer logs "generated docs/target" before exiting non-zero.
- The SSE stream no longer sends a duplicate seed frame; only actual rebuilds are pushed (`WatchStream::from_changes`).

## [0.1.7] - 2026-08-30

### Added
- **`vendor` Subcommand**: `cargo fy-docs vendor` (re)writes the embedded fy-spec template into `docs/fy-spec/lib.typ`; `vendor --check` verifies without writing and exits non-zero on drift, so CI can pin the template version. Requires no typst binary.
- **SSE Live Reload**: the dev server pushes build ids over a `/events` Server-Sent-Events stream, replacing the 1.5-second polling loop. The single `live.js` client ships with static builds too and stays silent when no server answers.
- **Typst Precheck**: every compiling command verifies up front that `typst` exists and is at least 0.14, failing with an actionable message instead of a raw flag error.
- **Windows CI Job** and a typst pinned to 0.15.1 in CI, so "tested against Typst 0.15" is enforced rather than implied.
- **Real-Binary Integration Tests**: fresh build, broken-source exit code, and vendor drift checks run against the actual executable, skipping gracefully where typst is absent.
- **fy-spec Single Source of Truth**: the template's `typst.toml` and `examples/basic.typ` moved into this repository's `docs/fy-spec/`; the HTML class contract between `lib.typ` and `base.css` is documented as a dogfood spec chapter.

### Changed
- **Exit Codes**: `build` and `html` now exit non-zero when compilation fails (previously exit 0 after writing an error page), so CI pipelines catch broken documents. The `dev` server still survives failures and renders error pages.
- Dev-mode rebuilds inherit the startup `--lang` / `--with-pdf` options instead of silently rebuilding every language after the first save.
- Compile failures write error pages per language (`index_<lang>.html`); successfully built targets keep their fresh pages and the multi-language landing page is never overwritten by error output.
- Output files are written atomically (sibling temp file + rename), so a dev-server reload can never serve a half-written page.
- The watcher excludes `docs/target/` and `docs/release/` by path, not only by file extension.
- UI language derives from the document's language target instead of scanning for CJK characters (Japanese documents no longer get Chinese controls).
- Minimum Typst raised from 0.13 to 0.14: the `--pdf-standard 2.0` flag fy-docs passes first shipped there.
- README corrected: PDF filenames include the language suffix, the options table lists `--with-pdf`/`--no-open`, and the version floor is stated accurately.

### Fixed
- Redirect landing page values are escaped for JavaScript string contexts (backslash, control characters) and the default target is escaped in both its script and `<noscript>` forms; an unusual language directory can no longer produce invalid JavaScript.
- Compile-thread panics degrade to per-language error pages with a log line instead of aborting the whole process.
- `_temp_*.html` intermediates are removed on every path, including extraction failures.
- `<body>` extraction tolerates attributes on the opening tag, so a typst export format drift cannot break page assembly.
- Cargo.toml parsing is unified: `init` now resolves `workspace = true` inherited versions like `build` does, and the two duplicate `.gitignore` helpers collapsed into one.

### Removed
- Retired the standalone `fy-spec` repository; every project vendors its template copy via `cargo fy-docs vendor`, and this repository is the template's only home.

## [0.1.6] - 2026-08-30

### Added
- **Multilingual (i18n) Support**: Automatically detect single-language and multi-language specification folders (`docs/<lang>/main.typ` like `docs/zh-CN/`, `docs/en/`).
- **Interactive Language Switcher**: Added top toolbar `🌐 Language Switcher` dropdown with smooth in-page navigation between `index_<lang>.html` pages.
- **CLI Commands Reorganized**:
  - `cargo fy-docs` (and `cargo fy-docs build`): Idempotent, non-blocking full build for HTML and PDF 2.0 (CI-safe).
  - `cargo fy-docs html`: Compile offline HTML documentation only.
  - `cargo fy-docs dev`: Interactive development server with live reload and browser auto-opening.
  - Added `--lang <LANG>` parameter to target specific language documentation.
- **Multilingual Dogfood Specifications**: Full bilingual specification books (English and Simplified Chinese) with interactive architecture DAG diagrams (powered by Fletcher).
- **CI Chinese Fonts**: Added Google Noto CJK fonts installation in GitHub Actions workflows to guarantee zero `.notdef` box artifacts on headless Linux runners.

### Changed
- **Template Decoupling**: Fully decoupled `fy-spec` template fonts with safe fallback across Linux, macOS, and Windows. Fonts can be overridden via `fonts` parameter.
- **Dynamic Cover Metadata**: Generic `title`, `lang`, `region` defaults without hardcoded ecosystem assumptions; `author`, `subtitle`, and `methodology` render dynamically only when provided.

### Removed
- Removed redundant `fy-docs/fy-spec` root directory, embedding directly from `docs/fy-spec/lib.typ`.

## [0.1.5] - 2026-08-30

### Added
- Enabled one-click return to cover by clicking toolbar title or sidebar project brand.
- Enabled sequential page navigation and arrow key paging back to cover as Chapter 0.

### Changed
- Upgraded PDF compilation target to modern PDF 2.0 (`--pdf-standard 2.0` / ISO 32000-2:2020) for enhanced tag semantics, accessibility, and color management.

### Fixed
- Fixed Typst HTML export container dropping issue (typst/typst#5512) by introducing `centered` helper and structural semantic `fy-cover` branches (`.fy-cover-chip`, `.fy-cover-meta` with `<dl>/<dt>/<dd>`).
- Updated `base.css` with dark/light theme styling for `.fy-cover` classes and completed `.fy-badge-done` styling.
- Fixed pager "Previous" link ignoring clicks on returning to cover due to missing anchor ID registration.

## [0.1.4] - 2026-08-30



### Added
- Automated multi-platform binary releases across 9 architectures (Linux GNU/musl x86_64/ARM64, macOS Apple Silicon/Intel, Windows x86_64/ARM64, FreeBSD x86_64).
- Automated build and attachment of versioned specification PDF documents (`fy-docs_v<version>_specification.pdf`) directly to GitHub Releases.


### Testing
- Expanded test coverage across all modules (server endpoints, project detection, watcher, compiler, scaffold, and CLI dispatcher), raising code line coverage to 92%+.


## [0.1.3] - 2026-08-30

### Documentation
- Documented the embedded `docs/fy-spec/` template directory structure and self-contained specification architecture in `README.md` and module specifications.

## [0.1.2] - 2026-08-30

### Added
- Comprehensive theme adaptation and interactive link styling for inline SVG diagrams exported by Typst `html.frame(...)` across all themes (Light, Rust, Coal, Navy, Ayu).

## [0.1.1] - 2026-08-30

### Changed
- Standardized `docs/` template importing to relative paths (`fy-spec/lib.typ` and `../fy-spec/lib.typ`), making doc projects fully self-contained without requiring root configuration in IDEs.
- Updated package manifest `include` list to package the local `docs/fy-spec/lib.typ`.
- De-duplicated and streamlined interaction contract items in `docs/modules/viewer.typ`.

## [0.1.0] - 2026-08-29

### Added
- `cargo fy-docs` interactive preview server with Axum, live-reloading on `.typ` changes.
- `cargo fy-docs init` subcommand to scaffold a `docs/` directory with a starter `main.typ` and bundled `fy-spec` template.
- `cargo fy-docs build` subcommand for offline static HTML generation in `docs/target/`.
- `cargo fy-docs pdf` subcommand for print-edition ISO B5 PDF compilation in `docs/release/`.
- Automatic Typst compile root detection based on absolute `#import` paths.
- Cargo workspace package metadata inheritance support.
- Six documentation themes (Light, Rust, Coal, Navy, Ayu, System preference).
- Resizable sidebar with table of contents navigation, per-chapter paging, and in-document search.
- Bilingual error page and UI localization (English / Simplified Chinese).
