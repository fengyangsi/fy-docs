# 更新日志

[English](CHANGELOG.md) | 简体中文

本项目的全部重要变更都会记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本遵循 [语义化版本 2.0.0](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### 新增
- **发布包内含集成测试**：打包 `include` 列表加入 `/tests/**`，解包后的 `.crate` 得以保留它所声明的 dev-dependencies 对应的测试套件。
- **语言命名规则成文**：语言目录仅当地区或文字子标签承载真实差异时才追加子标签，`en` 保持中性英文根语言，严禁为拼写差异分叉整份文档。

### 变更
- **未登记语言码按 BCP 47 形态展示**：基础子标签小写、地区全大写、文字子标签首字母大写，`docs/pt_BR/` 目录在语言切换器中显示为 `pt-BR` 而非照抄目录名；fy-docs 不臆造无法确知的语言名称。
- **规格章节只陈述现状**：移除文档中的仓库退役叙述、升级史措辞与设计取舍论证，这类历史归本文件。
- **修正落地页体积 invariant**：原文档声称“1KB 以内”，实测双语已达 1189 字节。现按实测陈述为每语言约 60 字节的线性增长，并由 `assets.rs` 断言锁死。

### 修复
- **页面正文语言标注**：根元素 `lang` 取自顶栏 chrome 语言，而 chrome 文案仅有中英两套，导致其余语言被读屏器与 `:lang()` 断字当作英文。现在页面标注自身规范化的语言标签，chrome 仍回退为英文。

### 移除
- `docs/target/` 清扫不再删除 `_poll.js` 与 `_build`：当前构建根本不产出这两个文件，保留名单等于让工具背负自己的升级史，且每次发版只会拉长它。被强制中断的编译所遗留的 `_temp_*.html` 中间文件仍会清理。

## [0.1.10] - 2026-08-31

### 新增
- **`pdf` 支持 `--open`**：`cargo fy-docs pdf --open` 打开首个生成的发行版 PDF，补齐 README 早已为其他子命令声明的选项。

### 变更
- **`--lang` 归一化且严格**：`ZH_CN`、`zh-cn`、`zh_CN` 现在指向同一目标；而过滤值匹配不到任何语言时以非零码失败并列出项目实际提供的语言，不再静默退化为"只构建 default 目标"。匹配保持精确：`--lang zh` 不会命中 `zh-CN`。
- **透传 typst 告警**：编译成功时转发 typst 的 stderr，字体替换与 HTML 导出丢弃的排版指令不再藏身于绿色构建之后。重复上报的 `unknown font family` 折叠为按字体族去重的一行，转发的诊断文本一并剥掉 Windows verbatim 路径前缀。
- **语言目录判定改为正向规则**：`docs/` 下自带 `main.typ` 的子目录即为语言目标，仅生成目录除外；共享源目录不必再维护硬编码名单。
- **去抖重建设有上限**：连续保存仍合并为一次构建，但等待自首次变更起最长 2 秒，持续写入源文件的进程无法再无限推迟重建。

### 修复
- 绝对导入扫描尊重单双引号与 `//` 注释，被注释掉的 `#import` 不再把 root 拖到错误的祖先目录；该扫描同时跳过 `docs/release/`。
- `main.typ` 版本回退只读未注释代码，且遇到无引号值的 `version:` 时继续查找而非直接放弃。
- 每次构建清扫 `docs/target/`：移除 fy-docs 已不再写出的历史产物（`_poll.js`、`_build`）与被强制中断的编译残留的 `_temp_*.html`。
- `extract_all_styles` 现可提取带属性的 `<style>` 标签，使多语言样式合并从死代码变为生效路径。
- `--with-pdf` 阶段失败时仍为纯多语言项目保留语言路由分流页，开发服务器根路径不再无路由可用。

### 移除
- 删除零读取的 `Project::entry` 字段与已被取代的 `Project::pdf_file_name()`，并将仅供测试使用的 `AppState::new` 收进 `#[cfg(test)]`。内部项可见性统一收窄为 `pub(crate)`——本 crate 有意不公开任何库 API。

## [0.1.9] - 2026-08-31

### 修复
- dev 模式热重载恢复「首次保存即刷新」：`/events` 流以 `WatchStream::new` 在订阅时恰好推送一次当前构建编号作为基线帧，打开页面后第一次保存即触发重载，不再需要等第二次重建。此改动回退 0.1.8 的 `WatchStream::from_changes`（它把每个页面的首个基线帧一并抑制了）；0.1.8 修掉的重复种子帧问题依然不复存在。

## [0.1.8] - 2026-08-31

### 修复
- 部分失败的构建不再清空 `typst.css`：错误页仅在文件缺失时写入空样式，成功语言的合并样式得以保留。
- 多语言项目仅剩一个成功语言时，保留语言路由分流页，不再把该语言页面复制为 `index.html`。
- 构建失败时不再先打印 "generated docs/target" 再以非零码退出。
- SSE 流不再发送重复的种子帧，仅推送真正的重建事件（`WatchStream::from_changes`）。

## [0.1.7] - 2026-08-30

### 新增
- **`vendor` 子命令**：`cargo fy-docs vendor` 将内嵌的 fy-spec 模板（重）写入 `docs/fy-spec/lib.typ`；`vendor --check` 只读校验、漂移即非零退出，供 CI 锁定模板版本。该命令无需 typst。
- **SSE 热重载**：dev 服务器通过 `/events`（Server-Sent Events）流推送构建编号，取代 1.5 秒轮询；单轨 `live.js` 客户端随静态构建一同分发，无服务器应答时静默关闭。
- **typst 预检**：所有编译类命令启动即校验 `typst` 存在且不低于 0.14，以可操作的信息报错，而非裸露的参数错误。
- **Windows CI 测试任务**，并在 CI 中将 typst 锁定到 0.15.1，让「基于 Typst 0.15 测试」成为强制事实而非口头声明。
- **真实二进制集成测试**：全新构建、损坏源码退出码、vendor 漂移校验均对真实可执行文件运行；无 typst 环境自动跳过。
- **fy-spec 唯一真身**：模板的 `typst.toml` 与 `examples/basic.typ` 迁入本仓库 `docs/fy-spec/`；`lib.typ` 与 `base.css` 之间的 HTML 类名契约以自食文档章节固化。

### 变更
- **退出码**：`build` 与 `html` 在编译失败时以非零退出码结束（此前写完错误页后仍退出 0），CI 流水线得以拦截损坏文档；`dev` 服务器依旧存活并将失败渲染为错误页。
- dev 模式重建继承启动时的 `--lang` / `--with-pdf` 选项，不再在首次保存后悄悄全量重建。
- 编译失败按语言写入错误页（`index_<lang>.html`）；成功的语言保留最新产物，多语言分流页绝不被错误输出覆盖。
- 产物原子写入（同目录临时文件 + rename），dev 服务器重载期间不可能读到半截页面。
- watcher 按路径排除 `docs/target/` 与 `docs/release/`，不再仅依赖扩展名过滤。
- UI 语言改由文档语言目标推导，弃用 CJK 字符嗅探（日文文档不再误判为中文）。
- 最低 Typst 版本由 0.13 提升至 0.14：fy-docs 传入的 `--pdf-standard 2.0` 自该版本起才受支持。
- 修正 README：PDF 文件名补语言后缀、参数表补全 `--with-pdf`/`--no-open`、版本下限如实标注。

### 修复
- 重定向分流页的取值按 JavaScript 字符串上下文转义（反斜杠、控制字符），默认跳转目标在脚本与 `<noscript>` 两处均过转义；异常语言目录不再可能产出非法 JS。
- 编译线程 panic 降级为该语言的错误页与一行日志，不再中止整个进程。
- `_temp_*.html` 中间文件在所有路径（含提取失败）下均被删除。
- `<body>` 提取容忍开始标签带属性，typst 导出格式演进不再破坏页面拼装。
- Cargo.toml 解析统一：`init` 现与 `build` 一样解析 `workspace = true` 继承版本；两处重复的 `.gitignore` helper 合一。

### 移除
- 独立仓库 `fy-spec` 退役；各项目经 `cargo fy-docs vendor` 自持模板副本，本仓库成为模板唯一真身。

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
