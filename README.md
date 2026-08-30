# fy-docs

[![crates.io](https://img.shields.io/crates/v/fy-docs)](https://crates.io/crates/fy-docs)
[![docs.rs](https://docs.rs/fy-docs/badge.svg)](https://docs.rs/fy-docs)
[![CI](https://github.com/fengyangsi/fy-docs/actions/workflows/ci.yml/badge.svg)](https://github.com/fengyangsi/fy-docs/actions/workflows/ci.yml)
[![Coverage](https://coveralls.io/repos/github/fengyangsi/fy-docs/badge.svg?branch=main)](https://coveralls.io/github/fengyangsi/fy-docs)
[![License](https://img.shields.io/crates/l/fy-docs)](LICENSE-MIT)

English | [简体中文](README.zh-CN.md)

A Cargo subcommand for building and previewing Typst specification documents. Run it inside a project containing `docs/main.typ` to generate a local reading page, watch source changes, and produce a versioned PDF when needed.

## Installation

### Via Cargo (Recommended for Rust developers)

```powershell
cargo install fy-docs
```

The installed executable is `cargo-fy-docs`, which Cargo discovers as the `cargo fy-docs` command.

### Pre-built Binaries (Direct Download)

Pre-compiled binaries for **Windows (x64 / ARM64)**, **macOS (Apple Silicon / Intel)**, **Linux (x64 / ARM64, musl static)**, and **FreeBSD (x64)** are available on the [GitHub Releases](https://github.com/fengyangsi/fy-docs/releases) page. Download the archive for your platform, extract `cargo-fy-docs` into a directory on your `PATH`, and run:


```powershell
cargo-fy-docs --version
```


## Usage

```powershell
# Scaffold a docs/ directory with a starter main.typ, embedded fy-spec template, and modules/ folder.
cargo fy-docs init

# Full build: compile all language HTML pages and PDF 2.0 specifications (default command, CI-safe).
cargo fy-docs
# or explicitly:
cargo fy-docs build

# Build offline HTML documentation only.
cargo fy-docs html

# Build versioned PDF 2.0 specification(s) only.
cargo fy-docs pdf

# Interactive development: start local dev server with live reload and browser preview.
cargo fy-docs dev
```

Common options:

```powershell
cargo fy-docs --lang zh-CN    # Target a specific language
cargo fy-docs --open          # Open in browser after build
cargo fy-docs --root D:\Code  # Explicitly specify Typst root
cargo fy-docs dev --port 8181 # Customize dev server port
```

## Output

| Artifact | Location |
|---|---|
| Offline HTML reading page | `docs/target/index.html` |
| Print-edition PDF | `docs/release/<package>_v<version>_specification.pdf` |

The package name and version are read from `[package]` in `Cargo.toml`, including Cargo workspace inheritance. If the manifest has no package metadata, fy-docs falls back to the document's `version:` field and then to `0.1.0`.

## Document Layout

Single-language documentation layout:

```text
project/
├── Cargo.toml
├── src/
└── docs/
    ├── main.typ             # Single Typst entry point
    ├── fy-spec/             # Embedded template library (lib.typ, self-contained)
    ├── modules/             # Specification source by module
    ├── target/              # Generated HTML reading pages (Git ignored)
    └── release/             # Versioned specification PDFs (Git ignored)
```

Multilingual (i18n) documentation layout:

```text
project/
├── Cargo.toml
├── src/
└── docs/
    ├── fy-spec/             # Shared embedded template library (lib.typ)
    ├── zh-CN/               # Simplified Chinese specification
    │   ├── main.typ
    │   └── modules/
    ├── en/                  # English specification
    │   ├── main.typ
    │   └── modules/
    ├── target/              # Generated index.html, index_zh-CN.html, index_en.html
    └── release/             # Versioned PDFs for each language
```

`docs/fy-spec/` contains the embedded styling templates (ISO B5 layout, semantic contract boxes, badges), ensuring specification documents are fully self-contained and reproducible without external dependencies.

`docs/target/` and `docs/release/` are generated artifacts and should be ignored by Git. They are intentionally separate from Cargo's `target/release/` program artifacts.

## Requirements

`typst` must be available on `PATH` with HTML export support (Typst 0.13 or later). fy-docs is developed and tested against Typst 0.15.

## Changelog

Detailed release notes and migration guides for each version are documented in [CHANGELOG.md](CHANGELOG.md).

## License

Dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE), at your option.

