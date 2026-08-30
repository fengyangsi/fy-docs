#import "../fy-spec/lib.typ": *
#import "@preview/fletcher:0.5.8": diagram, node, edge

#show: project_book.with(
  title: "fy-docs 规格说明书",
  subtitle: "多语言 Typst 规格文档编译器与实时阅读器",
  version: "0.1.6",
  author: "fengyangsi",
  date: "2026-08-30",
  lang: "zh-CN",
  methodology: "541 演进式契约驱动开发",
)

= 系统架构与模块 DAG <sec-arch>

`fy-docs` 采用清晰解耦的模块化设计，系统由 5 个核心模块构成，模块之间的依赖关系严格保持为有向无环图（DAG）。在网页端点击下方节点即可快速跳转至对应模块规格：

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
    node((0.5, 0), link(<sec-cli>)[*cli 模块*\ 命令行调度与命令分发], fill: rgb("#eff6ff"), stroke: 1.5pt + rgb("#1d4ed8")),
    node((0, 1), link(<sec-project>)[*project 模块*\ 项目与多语言目标探测]),
    node((1, 1), link(<sec-server>)[*server 模块*\ Axum 服务与热重载]),
    node((0.5, 2), link(<sec-compiler>)[*compiler 模块*\ Typst 编译与资产拼装]),
    node((0.5, 3), link(<sec-viewer>)[*viewer 模块*\ 前端阅读器与语言切换]),
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
