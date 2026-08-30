#import "../../fy-spec/lib.typ": *

= fy-spec Library: The Embedded Typst Template <sec-fy-spec>

`fy-spec` is the Typst template library embedded in the fy-docs binary and vendored into every project at `docs/fy-spec/lib.typ`. This repository is its single source of truth: `docs/fy-spec/` carries `lib.typ`, the `typst.toml` package manifest, and `examples/basic.typ` as a compile-time smoke test. The former standalone `fy-spec` repository is retired.

#status-badge(status: "已确立", phase: "模板 v0.1.0")

#contract[
  The library stays self-contained: zero `@preview` dependencies, a single `lib.typ`, and relative-path imports only, so every vendored project compiles offline and reproducibly.
]

#contract[
  `project_book` provides the ISO B5 print shell (cover, running headers/footers, outline) and branches on `target()`: HTML exports emit semantic `fy-*` classes, print exports emit styled Typst blocks.
]

#contract[
  Callout components (`contract`, `invariant`, `example-box`, plus the domain boxes `logic-box`, `proof-box`, `math-box`, `geom-box`, `axiom-box`, `motion-box`) share one `callout` base; `status-badge` renders pending/done markers and localizes its own label.
]

#contract[
  UI strings localize through `i18n-strings` and `resolve-i18n` for English, Chinese, Japanese, German, and French, with base-language and English fallback.
]

== HTML Class Contract

The HTML viewer stylesheet (`assets/base.css`) relies *only* on the classes below — never on Typst's own experimental HTML structure:

#figure(
  table(
    columns: (auto, auto),
    inset: 6pt,
    align: (auto, left),
    table.header([*Component*], [*HTML output*]),
    [`centered`], [`<div style="text-align: center">` — replaces the `align(center, ..)` that HTML export drops],
    [`project_book` cover], [`<div class="fy-cover">` root container],
    [Cover type chip], [`<span class="fy-cover-chip">`],
    [Cover metadata list], [`<dl class="fy-cover-meta">` with `<dt>` / `<dd>`],
    [`callout` (all domain boxes)], [`<div class="fy-box fy-<kind>">`, kind in `note / contract / invariant / example / logic / proof / math / geom / axiom / motion`],
    [Callout title], [`<span class="fy-box-title">`],
    [`status-badge`], [`<span class="fy-badge fy-badge-pending">` or `fy-badge-done`],
  ),
  caption: [The complete class contract between `lib.typ` and `base.css`.],
)

#invariant[
  Any rename of an emitted `fy-*` class in `lib.typ` must be matched in `assets/base.css` within the same commit, and projects stay pinned to this template version via `cargo fy-docs vendor --check` in CI.
]
