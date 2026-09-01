#import "../fy-spec/lib.typ": *
#import "@preview/fletcher:0.5.8": diagram, node, edge

#show: project_book.with(
  title: "fy-docs 规格说明书",
  subtitle: "多语言 Typst 规格文档编译器与实时阅读器",
  version: "0.1.10",
  author: "fengyangsi",
  date: "2026-08-31",
  lang: "zh-CN",
  methodology: "541 演进式契约驱动开发",
)

= 系统架构与模块 DAG <sec-arch>

`fy-docs` 采用清晰解耦的模块化设计：七个模块，模块之间的依赖关系严格保持为有向无环图（DAG），每个模块由且仅由一章规格描述；另有内嵌的 `fy-spec` 模板库，由文档源码自身导入。在网页端点击下方节点即可快速跳转至对应模块规格：

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
    node((0.5, 0), link(<sec-cli>)[*cli 模块*\ 命令行调度与终端输出], fill: rgb("#eff6ff"), stroke: 1.5pt + rgb("#1d4ed8")),
    node((0, 1), link(<sec-scaffold>)[*scaffold 模块*\ 工程骨架与模板分发]),
    node((1, 1), link(<sec-server>)[*server 模块*\ Axum 服务与热重载]),
    node((0, 2), link(<sec-project>)[*project 模块*\ 元数据与多语言目标探测]),
    node((1, 2), link(<sec-page>)[*page 模块*\ 页面装配与转义]),
    node((0.5, 3), link(<sec-compiler>)[*compiler 模块*\ Typst 编译与资产拼装]),
    node((0.5, 4), link(<sec-viewer>)[*viewer 模块*\ 前端阅读器与语言切换]),
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

`page` 章负责生成页面的标记外壳，`viewer` 章负责在这层外壳上运行的浏览器代码；外壳声明的 `id` 与 `class` 集合就是两章的边界。`fy-spec` 不占 DAG 节点，因为没有任何 Rust 代码在运行期查询它：二进制逐字内嵌该模板，而每份文档源码导入的是落地到项目里的那份副本。

箭头由产出方指向消费其产物的一方。因而这张图表达的是*消费分层*，不是 Rust 的 `use` 图，也不得被当作 `use` 图来读。

#invariant[
  `src/`、`assets/` 与 `docs/fy-spec/` 下的每个文件恰好归属一章，归属关系由下表给出。体量过小、不足以独占一个节点的模块*折叠*进拥有它的章节，折叠模块一律不出现在 DAG 中。仅经折叠模块才成立的关联不画：`compiler` 引用 `AppState`，而承载它的文件折叠进 `server`；`server` 的折叠 watcher 又驱动 `compiler`——这两条边都不入图，所以分层不会被读成环。任何实现文件在表中没有归属，即为规格缺口。
]

#figure(
  table(
    columns: (auto, auto),
    inset: 6pt,
    align: (auto, left),
    table.header([*章节*], [*实现文件*]),
    [`cli`], [`src/main.rs`；折叠：`src/term.rs`（终端输出）、`src/lib.rs`（crate 级文档）],
    [`scaffold`], [`src/scaffold.rs`],
    [`project`], [`src/project/mod.rs`、`src/project/lang.rs`、`src/project/cargo_meta.rs`、`src/project/imports.rs`、`src/project/template_args.rs`],
    [`compiler`], [`src/compiler/mod.rs`、`src/compiler/typst.rs`、`src/compiler/extract.rs`、`src/compiler/warnings.rs`、`src/compiler/output.rs`],
    [`page`], [`src/assets.rs`、`assets/doc.html`],
    [`server`], [`src/server.rs`；折叠：`src/watcher.rs`（源码监听）、`src/state.rs`（dev 共享状态）],
    [`viewer`], [`assets/viewer.js`、`assets/base.css`、`assets/live.js`],
    [`fy-spec`], [`docs/fy-spec/lib.typ`、`docs/fy-spec/typst.toml`、`docs/fy-spec/examples/basic.typ`],
  ),
  caption: [实现文件到规格章节的完整归属表。],
)

#include "modules/cli.typ"
#include "modules/scaffold.typ"
#include "modules/project.typ"
#include "modules/compiler.typ"
#include "modules/page.typ"
#include "modules/server.typ"
#include "modules/viewer.typ"
#include "modules/fy-spec.typ"
