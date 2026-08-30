# fy-docs

[![crates.io](https://img.shields.io/crates/v/fy-docs)](https://crates.io/crates/fy-docs)
[![docs.rs](https://docs.rs/fy-docs/badge.svg)](https://docs.rs/fy-docs)
[![CI](https://github.com/fengyangsi/fy-docs/actions/workflows/ci.yml/badge.svg)](https://github.com/fengyangsi/fy-docs/actions/workflows/ci.yml)
[![Coverage](https://coveralls.io/repos/github/fengyangsi/fy-docs/badge.svg?branch=main)](https://coveralls.io/github/fengyangsi/fy-docs)
[![License](https://img.shields.io/crates/l/fy-docs)](LICENSE-MIT)

English | [简体中文](README.zh-CN.md)

A Cargo subcommand for building and previewing Typst specification documents. Run it inside a project containing `docs/main.typ` to generate a local reading page, watch source changes, and produce a versioned PDF when needed.

## Installation

```powershell
cargo install fy-docs
```

The installed executable is `cargo-fy-docs`, which Cargo discovers as the `cargo fy-docs` command.

## Usage

```powershell
# Scaffold a docs/ directory with a starter main.typ, embedded fy-spec template, and modules/ folder.
cargo fy-docs init

# Interactive preview: build HTML, open the browser, and watch .typ files.
cargo fy-docs

# Build the offline HTML reading page only.
cargo fy-docs build

# Build the HTML page and a print-edition PDF together.
cargo fy-docs build --with-pdf

# Build only the print-edition PDF.
cargo fy-docs pdf
```

Common options:

```powershell
cargo fy-docs --root D:\Code\fy
cargo fy-docs --port 8181
cargo fy-docs --no-open
```

## Output

| Artifact | Location |
|---|---|
| Offline HTML reading page | `docs/target/index.html` |
| Print-edition PDF | `docs/release/<package>_v<version>_specification.pdf` |

The package name and version are read from `[package]` in `Cargo.toml`, including Cargo workspace inheritance. If the manifest has no package metadata, fy-docs falls back to the document's `version:` field and then to `0.1.0`.

## Document Layout

```text
project/
├── Cargo.toml
├── src/
├── target/                  # Program build artifacts
└── docs/
    ├── main.typ             # Typst entry point
    ├── fy-spec/             # Embedded specification template library (lib.typ, self-contained)
    ├── modules/             # Specification source, organized by module
    ├── target/              # Generated HTML, CSS, and JavaScript (Git ignored)
    └── release/             # Versioned specification PDFs (Git ignored)
```

`docs/fy-spec/` contains the embedded styling templates (ISO B5 layout, semantic contract boxes, badges), ensuring specification documents are fully self-contained and reproducible without external dependencies.

`docs/target/` and `docs/release/` are generated artifacts and should be ignored by Git. They are intentionally separate from Cargo's `target/release/` program artifacts.

## Requirements

`typst` must be available on `PATH` with HTML export support (Typst 0.13 or later). fy-docs is developed and tested against Typst 0.15.

## License

Dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE), at your option.
