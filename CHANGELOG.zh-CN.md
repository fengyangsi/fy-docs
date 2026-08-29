# 更新日志

[English](CHANGELOG.md) | 简体中文

本项目的所有重要变更都会记录在此文件中。

本日志格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
并且本项目遵循 [语义化版本 2.0.0](https://semver.org/lang/zh-CN/)。

## [0.1.0] - 2026-08-29

### 新增 (Added)
- `cargo fy-docs` 交互式预览服务器（基于 Axum），支持监听 `.typ` 文件变更并实时热重载。
- `cargo fy-docs init` 子命令，用于脚手架初始化 `docs/` 目录、起始 `main.typ` 入口及内置的 `fy-spec` 模板。
- `cargo fy-docs build` 子命令，用于在 `docs/target/` 目录下生成离线静态 HTML 阅读页。
- `cargo fy-docs pdf` 子命令，用于在 `docs/release/` 目录下编译打印版 ISO B5 规格书 PDF。
- 基于绝对路径 `#import` 语句自动推断并定位 Typst 编译根目录（Compile Root）。
- 支持继承并解析 Cargo Workspace 清单中的包元数据。
- 6 种文档主题配色（Light、Rust、Coal、Navy、Ayu、跟随系统偏好）。
- 可拖拽调节宽度的目录侧边栏、按章节独立翻页阅读与文档内即时搜索。
- 双语错误提示页与界面本地化支持（英文 / 简体中文自动切换）。
