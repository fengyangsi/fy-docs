#import "../fy-spec/lib.typ": *
#import "@preview/fletcher:0.5.8": diagram, node, edge

#show: project_book.with(
  title: "fy-docs Specification",
  subtitle: "Multilingual Typst Specification Compiler & Live Viewer",
  version: "0.1.8",
  author: "fengyangsi",
  date: "2026-08-31",
  lang: "en",
)

= System Architecture & Module DAG <sec-arch>

`fy-docs` follows a decoupled, modular architecture consisting of 5 core modules with strictly acyclic dependencies (DAG). In the HTML viewer, click on any node below to navigate directly to its section:

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
    node((0.5, 0), link(<sec-cli>)[*cli module*\ CLI Dispatch & Subcommands], fill: rgb("#eff6ff"), stroke: 1.5pt + rgb("#1d4ed8")),
    node((0, 1), link(<sec-project>)[*project module*\ Project & i18n Detection]),
    node((1, 1), link(<sec-server>)[*server module*\ Axum Dev Server & Reload]),
    node((0.5, 2), link(<sec-compiler>)[*compiler module*\ Typst Engine & Bundler]),
    node((0.5, 3), link(<sec-viewer>)[*viewer module*\ Frontend Reader & i18n]),
    edge((0.5, 0), (0, 1), "->"),
    edge((0.5, 0), (1, 1), "->"),
    edge((0.5, 0), (0.5, 2), "->"),
    edge((0, 1), (0.5, 2), "->"),
    edge((1, 1), (0.5, 2), "->"),
    edge((0.5, 2), (0.5, 3), "->"),
  )
  if target() == "html" { html.frame(d) } else { align(center, d) }
}
#v(10pt)

#include "modules/cli.typ"
#include "modules/project.typ"
#include "modules/compiler.typ"
#include "modules/server.typ"
#include "modules/viewer.typ"
#include "modules/fy-spec.typ"
