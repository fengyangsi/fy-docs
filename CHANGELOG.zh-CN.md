# 更新日志

[English](CHANGELOG.md) | 简体中文

本项目的全部重要变更都会记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本遵循 [语义化版本 2.0.0](https://semver.org/lang/zh-CN/)。

## [0.1.4] - 2026-08-30

### 新增
- 支持 9 大平台架构预编译二进制程序全自动构建与发布（Linux GNU/musl x86_64/ARM64、macOS Apple Silicon/Intel、Windows x86_64/ARM64、FreeBSD x86_64）。
- GitHub Release 自动编译并发布版本对应的官方规格说明书 PDF 文档（`fy-docs_v<版本>_specification.pdf`）。


### 测试
- 扩充全模块测试套件（服务器端点、工程探测、文件监听、编译器、脚手架与命令行调度），单测覆盖率大幅提升至 92%+。

## [0.1.3] - 2026-08-30

### 文档
- 在 `README.md` 及模块规格说明书中完整补充记录了 `docs/fy-spec/` 内嵌模板目录结构及其自包含设计规范。

## [0.1.2] - 2026-08-30

### 新增
- 增加对 Typst `html.frame(...)` 导出的内联 SVG 图表的全面主题自适应样式，自动响应全部 5 套主题（Light, Rust, Coal, Navy, Ayu）的卡片底色、边框、连线与文字，并支持图表节点超链接与微悬浮交互。

## [0.1.1] - 2026-08-30

### 变更
- 规范化 `docs/` 模板导入为相对路径（`fy-spec/lib.typ` 与 `../fy-spec/lib.typ`），使各项目规格文档完全自包含，在 IDE（如 VSCode / Tinymist）中单文件打开时无需配置全局根路径。
- 更新 `Cargo.toml` 打包清单 `include` 列表，包含本地 `docs/fy-spec/lib.typ`。
- 去除并精炼了 `docs/modules/viewer.typ` 中重复冗余的交互契约条目。

## [0.1.0] - 2026-08-29

### 新增
- `cargo fy-docs` 基于 Axum 的交互式实时预览服务，支持 `.typ` 源码变更热重载。
- `cargo fy-docs init` 初始化脚手架子命令，快速创建包含 `main.typ` 与内嵌 `fy-spec` 模板的 `docs/` 目录。
- `cargo fy-docs build` 静态 HTML 离线生成子命令，输出至 `docs/target/`。
- `cargo fy-docs pdf` ISO B5 打印版 PDF 编译子命令，输出至 `docs/release/`。
- 基于绝对导入路径的 Typst 编译根目录自动探测。
- Cargo workspace 包元数据继承支持。
- 六款文档主题切换（Light, Rust, Coal, Navy, Ayu, 跟随系统）。
- 可调节宽度的响应式侧边栏，支持目录导航、按章分页与全文搜索。
- 中英文双语错误页与阅读器界面本地化。
