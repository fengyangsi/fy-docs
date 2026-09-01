#import "../fy-spec/lib.typ": *
#import "@preview/fletcher:0.5.8": diagram, node, edge

#show: project_book.with(
  title: "fy-docs Specification",
  subtitle: "Multilingual Typst Specification Compiler & Live Viewer",
  version: "0.1.11",
  author: "fengyangsi",
  date: "2026-08-31",
  lang: "en",
)

= System Architecture & Module DAG <sec-arch>

`fy-docs` follows a decoupled, modular architecture: seven modules with strictly acyclic dependencies (DAG), each specified by exactly one chapter, plus the embedded `fy-spec` template library that the document sources themselves import. In the HTML viewer, click on any node below to navigate directly to its section:

#v(6pt)
#context {
  let d = diagram(
    spacing: (14mm, 9mm),
    node-stroke: 1pt + rgb("#2563eb"),
    node-fill: rgb("#f8fafc"),
    node-inset: 7pt,
    node-corner-radius: 4pt,
    edge-stroke: 1pt + rgb("#64748b"),
    mark-scale: 75%,
    node((0.5, 0), link(<sec-cli>)[*cli module*\ CLI Dispatch & Terminal Output], fill: rgb("#eff6ff"), stroke: 1.5pt + rgb("#1d4ed8")),
    node((0, 1), link(<sec-scaffold>)[*scaffold module*\ Scaffolding & Template Vendoring]),
    node((1, 1), link(<sec-server>)[*server module*\ Axum Dev Server & Reload]),
    node((0, 2), link(<sec-project>)[*project module*\ Metadata & i18n Detection]),
    node((1, 2), link(<sec-page>)[*page module*\ Page Assembly & Escaping]),
    node((0.5, 3), link(<sec-compiler>)[*compiler module*\ Typst Engine & Bundler]),
    node((0.5, 4), link(<sec-viewer>)[*viewer module*\ Frontend Reader & i18n]),
    edge((0.5, 0), (0, 1), "->"),
    edge((0.5, 0), (1, 1), "->"),
    edge((0.5, 0), (0, 2), "->"),
    edge((0.5, 0), (0.5, 3), "->"),
    edge((0, 1), (0, 2), "->"),
    edge((0, 2), (1, 2), "->"),
    edge((0, 2), (0.5, 3), "->"),
    edge((1, 2), (0.5, 3), "->"),
    edge((1, 1), (0.5, 3), "->"),
    edge((0.5, 3), (0.5, 4), "->"),
  )
  if target() == "html" { html.frame(d) } else { align(center, d) }
}
#v(10pt)

The `page` chapter owns the generated page's markup shell and the `viewer` chapter owns the code that runs inside a browser on that shell; the `id` and `class` surface the shell declares is the boundary between them. `fy-spec` carries no DAG node because no Rust code consults it at run time: the binary embeds the template verbatim, and every document source imports the vendored copy.

An arrow points from a chapter to the chapter that consumes what it produces. The picture is therefore a consumption layering, not the Rust `use` graph, and it is not meant to be read as one.

#invariant[
  Every file under `src/`, `assets/`, and `docs/fy-spec/` belongs to exactly one chapter, as listed below. A module too small to carry its own node is *folded* into the chapter that owns it, and a folded module never appears in the DAG. A relation that reaches a chapter only through a folded module is not drawn: `compiler` names `AppState`, whose file is folded into `server`, while `server`'s folded watcher drives `compiler` — both edges stay out of the picture, so the layering never reads as a cycle. An implementation file with no home in this table is a specification gap.
]

#figure(
  table(
    columns: (auto, auto),
    inset: 6pt,
    align: (auto, left),
    table.header([*Chapter*], [*Implementation files*]),
    [`cli`], [`src/main.rs`; folded: `src/term.rs` (terminal output) and `src/lib.rs` (crate documentation)],
    [`scaffold`], [`src/scaffold.rs`],
    [`project`], [`src/project/mod.rs`, `src/project/lang.rs`, `src/project/cargo_meta.rs`, `src/project/imports.rs`, `src/project/template_args.rs`],
    [`compiler`], [`src/compiler/mod.rs`, `src/compiler/typst.rs`, `src/compiler/extract.rs`, `src/compiler/warnings.rs`, `src/compiler/output.rs`],
    [`page`], [`src/assets.rs`, `assets/doc.html`],
    [`server`], [`src/server.rs`; folded: `src/watcher.rs` (source watching) and `src/state.rs` (shared dev state)],
    [`viewer`], [`assets/viewer.js`, `assets/base.css`, `assets/live.js`],
    [`fy-spec`], [`docs/fy-spec/lib.typ`, `docs/fy-spec/typst.toml`, `docs/fy-spec/examples/basic.typ`],
  ),
  caption: [The complete ownership map from implementation file to specifying chapter.],
)

#include "modules/cli.typ"
#include "modules/scaffold.typ"
#include "modules/project.typ"
#include "modules/compiler.typ"
#include "modules/page.typ"
#include "modules/server.typ"
#include "modules/viewer.typ"
#include "modules/fy-spec.typ"
