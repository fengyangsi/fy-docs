#import "../fy-spec/lib.typ": *

= cli 模块：Cargo 外部子命令

`fy-docs` 以 Cargo 外部子命令发布。安装后，PATH 中的二进制名为 `cargo-fy-docs`，使用者在任何包含 `docs/main.typ` 的项目目录中调用：

```text
cargo fy-docs init
cargo fy-docs
cargo fy-docs build
cargo fy-docs build --with-pdf
cargo fy-docs pdf
```

#contract[
  `cargo fy-docs` 是唯一受支持的命令入口。Cargo 发现 `cargo-fy-docs` 后会将 `fy-docs` 之后的参数原样转发给该程序。
]

#contract[
  `init` 是唯一不需要既有 `docs/main.typ` 的子命令：在当前目录创建 `docs/`，写入入口 `main.typ`、
  随二进制内嵌分发的 `fy-spec/lib.typ` 模板副本和空的 `modules/` 目录，并把 `docs/target/` 与
  `docs/release/` 补入 `.gitignore`。生成的 `main.typ` 使用相对导入，因此项目自包含，编译根即项目目录。
  标题页字段预填规则：包名与版本取自 `Cargo.toml` 的 `[package]`（缺失时回退为目录名与 `0.1.0`），
  作者取 `authors` 数组的第一项（缺失时回退为 `TODO`）。若 `docs/main.typ` 已存在，`init` 必须拒绝执行
  并提示，不得覆盖任何既有文件。
]

默认命令进入预览模式：构建 HTML、启动本地服务、尝试打开浏览器并监听源码。`build` 没有浏览器或服务副作用，只生成离线 HTML；`build --with-pdf` 明确表示同时生成 HTML 与 PDF；`pdf` 只生成 PDF。

若系统无法打开浏览器，预览模式必须在终端显示本地 URL，供使用者手动打开。

项目包名、页面品牌和生成资源名继续使用 `fy-docs`；`cargo-fy-docs` 只是 Cargo 发现外部命令所需的可执行文件名。
