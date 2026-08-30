#import "../fy-spec/lib.typ": *
#import "@preview/fletcher:0.5.8": diagram, node, edge

#show: project_book.with(
  title: "fy-docs Specification",
  subtitle: "Multilingual Typst Specification Compiler & Live Viewer",
  version: "0.1.6",
  author: "fengyangsi",
  date: "2026-08-30",
  lang: "en",
)

= System Architecture & Module DAG <sec-arch>

`fy-docs` follows a decoupled, modular architecture consisting of 5 core modules with strictly acyclic dependencies (DAG). In the HTML viewer, click on any node below to navigate directly to its section:

#v(8pt)
#context {
  let d = diagram(
    spacing: (20mm, 14mm),
    node-stroke: 1pt + rgb("#2563eb"),
    node-fill: rgb("#f1f5f9"),
    node-inset: 8pt,
    node-corner-radius: 4pt,
    edge-stroke: 1pt + rgb("#64748b"),
    mark-scale: 80%,
    node((1, 0), link(<sec-cli>)[*cli module*\ CLI Dispatch & Subcommands], fill: rgb("#eff6ff"), stroke: 1.5pt + rgb("#1d4ed8")),
    node((0, 1), link(<sec-project>)[*project module*\ Project & i18n Detection]),
    node((1, 1), link(<sec-compiler>)[*compiler module*\ Typst Engine & Bundler]),
    node((2, 1), link(<sec-server>)[*server module*\ Axum Dev Server & Reload]),
    node((1, 2), link(<sec-viewer>)[*viewer module*\ Frontend Reader & i18n]),
    edge((1, 0), (0, 1), "->"),
    edge((1, 0), (1, 1), "->"),
    edge((1, 0), (2, 1), "->"),
    edge((0, 1), (1, 1), "->"),
    edge((1, 1), (1, 2), "->"),
    edge((2, 1), (1, 1), "->"),
  )
  if target() == "html" { html.frame(d) } else { align(center, d) }
}
#v(12pt)

#include "modules/cli.typ"
#include "modules/project.typ"
#include "modules/compiler.typ"
#include "modules/server.typ"
#include "modules/viewer.typ"
