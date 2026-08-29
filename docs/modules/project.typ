#import "/fy-spec/lib.typ": *

= project 模块：项目探测与目录契约

#status-badge(status: "已确立", phase: "0.1.0")

`project` 负责将当前工作目录解析为一个文档项目。输入契约是项目根目录中存在 `docs/main.typ`；不满足时，`cargo fy-docs` 必须明确失败，而不能猜测其他入口。

== 目录结构

```text
项目根目录/
├── Cargo.toml               # 唯一的包清单
├── src/                     # Rust 程序源码
├── assets/                  # 内嵌的 HTML、CSS、JavaScript
├── target/                  # Cargo 程序构建产物
└── docs/
    ├── main.typ             # Typst 文档入口
    ├── modules/             # 规格源码，按实现模块组织
    ├── target/              # fy-docs 的 HTML 阅读页生成物
    └── release/             # 版本化 PDF 生成物
```

#contract[
  `docs/target/` 与 `docs/release/` 都是生成物，必须被 Git 忽略。根目录 `target/release/` 仅用于程序构建，绝不放入规格书 PDF。
]

== 名称、版本与 Typst 根

项目名称和版本优先来自根目录 `Cargo.toml` 的 `[package]`，也支持 `name.workspace = true` 与 `version.workspace = true` 的 Cargo workspace 继承；没有 package 时，名称退回目录名，版本退回 `main.typ` 的 `version:` 参数，最后才使用 `0.1.0`。清单以 TOML 解析器读取，不依赖文本行的排列或格式。

工具扫描文档内的绝对 Typst 导入，并从项目目录向上寻找能满足全部路径的最近根目录。`--root` 是覆盖自动探测的显式接口。
