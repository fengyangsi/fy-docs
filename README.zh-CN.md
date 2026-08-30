# fy-docs

[![crates.io](https://img.shields.io/crates/v/fy-docs)](https://crates.io/crates/fy-docs)
[![docs.rs](https://docs.rs/fy-docs/badge.svg)](https://docs.rs/fy-docs)
[![CI](https://github.com/fengyangsi/fy-docs/actions/workflows/ci.yml/badge.svg)](https://github.com/fengyangsi/fy-docs/actions/workflows/ci.yml)
[![Coverage](https://coveralls.io/repos/github/fengyangsi/fy-docs/badge.svg?branch=main)](https://coveralls.io/github/fengyangsi/fy-docs)
[![License](https://img.shields.io/crates/l/fy-docs)](LICENSE-MIT)

[English](README.md) | 简体中文

用于构建和预览 Typst 规格文档的 Cargo 子命令。在包含 `docs/main.typ` 的项目目录中执行，即可生成本地阅读页、监听文档变更，并按需生成带版本号的 PDF。

## 安装

### 通过 Cargo 安装（推荐 Rust 开发者）

```powershell
cargo install fy-docs
```

安装后得到的可执行文件名为 `cargo-fy-docs`，Cargo 会将其发现为 `cargo fy-docs` 命令。

### 下载预编译二进制程序（开箱即用）

在 [GitHub Releases](https://github.com/fengyangsi/fy-docs/releases) 发布页面中，为 **Windows (x64 / ARM64)**、**macOS (Apple Silicon / Intel)**、**Linux (x64 / ARM64, musl 静态单文件)** 及 **FreeBSD (x64)** 提供了预编译的二进制包。下载对应平台的压缩包并解压至系统 `PATH` 路径中即可直接使用：


```powershell
cargo-fy-docs --version
```


## 用法

```powershell
# 初始化 docs/ 目录：自动生成 main.typ 入口、内嵌 fy-spec 模板库与 modules/ 目录。
cargo fy-docs init

# 交互预览：构建 HTML、打开浏览器并监听 .typ 文件。
cargo fy-docs

# 只构建离线 HTML 阅读页。
cargo fy-docs build

# 同时构建 HTML 阅读页与打印版 PDF。
cargo fy-docs build --with-pdf

# 只构建打印版 PDF。
cargo fy-docs pdf
```

常用参数：

```powershell
cargo fy-docs --root D:\Code\fy
cargo fy-docs --port 8181
cargo fy-docs --no-open
```

## 输出位置

| 产物 | 位置 |
|---|---|
| 离线 HTML 阅读页 | `docs/target/index.html` |
| 打印版 PDF | `docs/release/<package>_v<version>_specification.pdf` |

包名和版本优先读取 `Cargo.toml` 的 `[package]`，并支持 Cargo workspace 的继承字段。若清单不包含 package 元数据，则回退到文档中的 `version:` 字段，最后回退为 `0.1.0`。

## 文档目录结构

```text
项目根目录/
├── Cargo.toml
├── src/
├── target/                  # 程序构建产物
└── docs/
    ├── main.typ             # Typst 文档入口
    ├── fy-spec/             # 内嵌的规格模板库（lib.typ，完全自包含）
    ├── modules/             # 按模块组织的规格源码
    ├── target/              # 生成物：HTML、CSS、JavaScript（Git 忽略）
    └── release/             # 生成物：版本化的规格书 PDF（Git 忽略）
```

`docs/fy-spec/` 存放内嵌的完整排版规范模板（ISO B5 版式、语义契约框、状态徽章等），确保每个项目的规格文档完全自包含，在任何环境下均可独立编译与渲染，不依赖外部或系统级路径。

`docs/target/` 与 `docs/release/` 都是生成物，应当由 Git 忽略；它们与 Cargo 的 `target/release/` 程序构建产物保持分离。

## 环境要求

系统 `PATH` 中需要可用的 `typst`，并支持 HTML 导出（Typst 0.13 或更高版本）。fy-docs 基于 Typst 0.15 开发与测试。

## 更新日志

各版本的详细更新说明与变更记录请参阅 [CHANGELOG.zh-CN.md](CHANGELOG.zh-CN.md)。

## 许可证

双许可：[MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE)，使用者任选其一。

