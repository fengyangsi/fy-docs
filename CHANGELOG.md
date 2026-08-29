# Changelog

English | [简体中文](CHANGELOG.zh-CN.md)

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
