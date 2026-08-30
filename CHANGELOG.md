# Changelog

English | [简体中文](CHANGELOG.zh-CN.md)

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
