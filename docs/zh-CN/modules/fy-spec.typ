#import "../../fy-spec/lib.typ": *

= fy-spec 模板库：内嵌 Typst 排版系统 <sec-fy-spec>

`fy-spec` 是内嵌于 fy-docs 二进制、并以 `docs/fy-spec/lib.typ` 形式随项目分发的 Typst 模板库。本仓库是其唯一真身：`docs/fy-spec/` 内含 `lib.typ`、包清单 `typst.toml` 与冒烟示例 `examples/basic.typ`。

#status-badge(status: "已确立", phase: "模板 v0.1.0")

#contract[
  模板保持完全自包含：零 `@preview` 依赖、单一 `lib.typ`、仅相对路径导入，任何分发后的项目均可离线、可复现地编译。
]

#contract[
  `project_book` 提供 ISO B5 印刷版式（封面、页眉页脚、目录），并按 `target()` 分支：HTML 导出发出语义化 `fy-*` 类名，印刷导出生成 Typst 排版块。
]

#contract[
  提示框组件（`contract`、`invariant`、`example-box` 及领域框 `logic-box`、`proof-box`、`math-box`、`geom-box`、`axiom-box`、`motion-box`）共享同一 `callout` 基座；`status-badge` 渲染进行中/已确立徽章并自动本地化标签。
]

#contract[
  界面文案经 `i18n-strings` 与 `resolve-i18n` 本地化，覆盖英、中、日、德、法五种语言，并按基础语言、英语逐级兜底。
]

== HTML 类名契约

HTML 阅读器样式表（`assets/base.css`）*只*依赖下表列出的类名，绝不依赖 Typst 实验性的 HTML 导出结构：

#figure(
  table(
    columns: (auto, auto),
    inset: 6pt,
    align: (auto, left),
    table.header([*组件*], [*HTML 输出*]),
    [`centered`], [`<div style="text-align: center">` —— 替代 HTML 导出会丢弃的 `align(center, ..)`],
    [`project_book` 封面], [`<div class="fy-cover">` 根容器],
    [封面类型徽章], [`<span class="fy-cover-chip">`],
    [封面元数据列表], [`<dl class="fy-cover-meta">`（配合 `<dt>` 与 `<dd>`）],
    [`callout`（含全部领域框）], [`<div class="fy-box fy-<kind>">`，kind 取 `note / contract / invariant / example / logic / proof / math / geom / axiom / motion`],
    [提示框标题], [`<span class="fy-box-title">`],
    [`status-badge`], [`<span class="fy-badge fy-badge-pending">` 或 `fy-badge-done`],
  ),
  caption: [`lib.typ` 与 `base.css` 之间的完整类名契约。],
)

#contract[
  每种提示框在 HTML 中都保持可辨别：`base.css` 为 `lib.typ` 能产出的每一类都备有样式，于是 `note`、`logic`、`proof`、`math`、`geom`、`axiom`、`motion` 不会塌缩成普通的 `contract` 或 `invariant` 观感。某一类没有样式是缺陷而非风格选择：类名正是样式表据以着色的键，缺少样式只会让该框与普通框长得一模一样，本库定义的这套分类在屏幕上就此消失。
]

#invariant[
  类名面是双向契约：本库产出的每个 `fy-*` 类都必须在 `assets/base.css` 中有规则，`base.css` 也不得为没有任何产出方的 `fy-*` 类书写样式。任一侧改名都必须在同一提交内同步另一侧；各项目通过 CI 中的 `cargo fy-docs vendor --check` 锁定本模板版本。
]
