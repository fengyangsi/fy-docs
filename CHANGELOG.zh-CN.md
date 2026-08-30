# 更新日志

[English](CHANGELOG.md) | 简体中文

本项目的全部重要变更都会记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本遵循 [语义化版本 2.0.0](https://semver.org/lang/zh-CN/)。

## [0.1.6] - 2026-08-30

### 新增
- **多语言（i18n）原生支持**：自动探测单语言与多语言规格目录（`docs/<lang>/main.typ` 如 `docs/zh-CN/`、`docs/en/` 等），分别生成 `index_<lang>.html` 与对应语言的规格书 PDF。
- **顶栏交互式语言切换下拉框**：在多语言文档顶栏右侧新增 `🌐 语言切换下拉菜单`，点击可在多语言页面间同级顺畅跳转；单语言项目自动隐藏。
- **CLI 命令体系正交重构**：
  - `cargo fy-docs`（及 `cargo fy-docs build`）：安全全量构建（HTML + PDF 2.0），执行完毕后以退出码 `0` 退出，彻底消除 CI 挂起风险。
  - `cargo fy-docs html`：仅构建离线 HTML 网页包。
  - `cargo fy-docs dev`：交互开发工作台，启动本地 Web 服务，自动打开浏览器并监听源码热重载。
  - 新增 `--lang <LANG>` 参数以定向编译指定语言文档。
- **双语 Dogfood 规格说明书**：为 `fy-docs` 自身建立完整的中英双语规格说明书，并使用 Fletcher 绘制 5 大核心模块交互式架构 DAG 图（支持在网页端点击节点直接跳转）。
- **CI 中文字体保障**：在 GitHub Actions 流水线中预装 Google Noto 思源中文字体包，彻底根除 Linux 无头 Runner 上编译中文 PDF 出现 `.notdef` 豆腐块方格的缺陷。

### 变更
- **模板字体与语言解耦**：彻底解耦 `fy-spec` 模板字体，内置覆盖 Linux、macOS 与 Windows 的全平台安全回退栈，并支持调用方通过 `fonts` 参数自由覆盖。
- **封面元数据动态渲染**：去除强加的生态私货与写死默认值；`author`、`subtitle`、`methodology` 等字段仅在调用方显式传入时才动态渲染。

### 移除
- 彻底移除根目录下冗余的 `fy-docs/fy-spec` 目录，统一直接从 `docs/fy-spec/lib.typ` 进行编译期内嵌。

## [0.1.5] - 2026-08-30

### 新增
- 支持点击顶部工具栏标题或侧栏项目名称一键返回封面。
- 支持翻页导航（“上一页”按钮与键盘左右方向键）平滑退回封面（封面作为第 0 节参与有序翻页）。

### 变更
- 全面升级 PDF 编译规格至最新的 PDF 2.0 标准（`--pdf-standard 2.0` / ISO 32000-2:2020），显著增强标签语义、无障碍访问（Accessibility）及色彩渲染一致性。

### 修复
- 修复 Typst HTML 导出丢弃 `align`/`rect`/`line`/`grid` 容器导致封面、目录标题与居中内容空白的问题（typst/typst#5512）：引入 `centered` 辅助函数并重构封面为双分支语义结构（`.fy-cover-chip` 与 `.fy-cover-meta` `<dl>/<dt>/<dd>`）。
- 升级 `base.css` 样式表，适配 `.fy-cover` 类族的明暗主题，并补齐 `.fy-badge-done` 状态徽章样式。
- 修复封面因未分配锚点 ID 导致点击“上一页”按钮返回封面无响应的问题。

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
