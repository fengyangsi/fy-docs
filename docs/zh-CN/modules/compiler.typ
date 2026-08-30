#import "../../fy-spec/lib.typ": *

= compiler 模块：Typst 编译与资产生成 <sec-compiler>

`compiler` 模块驱动 `typst` CLI 执行 HTML 导出与 PDF 2.0 渲染，并拼接前端阅读器资产。

#contract[
  PDF 编译时传入 `--pdf-standard 2.0` 参数，严格遵循 ISO 32000-2:2020 国际规范导出 PDF 2.0 发行本。
]

#contract[
  `typst` 成功退出时，它写入 stderr 的内容（未知字体族、HTML 导出期间被忽略的排版指令等告警）转发到 fy-docs 的 stderr，不得静默丢弃：产物里的字体替换与降级渲染必须对用户可见。唯一例外是「未知字体族」告警：回退链本就列出多个操作系统的候选字体，同一缺失字体会在每个样式点重复上报，故按字体族去重汇总为一行；其余告警一律原样透传。
]

#contract[
  每次构建开始前先清扫 `docs/target/`：fy-docs 已不再写出的历史产物（轮询热重载时代的 `_poll.js` 与 `_build` 标记）以及被强制中断的编译残留的 `_temp_*.html` 中间文件，避免升级后的项目永久继续服务废弃文件。
]

#contract[
  `--with-pdf` 阶段失败时，在写出各语言错误页之后仍须保证根 `index.html` 存在：纯多语言项目的 `index.html` 不属于任何单一目标，缺了它开发服务器根路径将无路由可用。
]

#contract[
  HTML 编译支持多语言并发渲染（基于 `std::thread::scope` 并发调度 Typst CLI）。所有语言页面在 `docs/target/` 下同级生成（`index_zh-CN.html`、`index_en.html`），统一共享静态资产，并在根路径生成轻量客户端路由分流页 `index.html`。
]

#contract[
  编译失败的语言目标在其自身的 `index_<lang>.html` 渲染错误页；构建成功的语言保留最新产物，多语言路由分流页绝不被错误输出覆盖。
]

#invariant[
  `compiler` 模块在多语言模式下生成的根 `index.html` 体积必须保持在 1KB 以内，绝不复制大文件正文，通过动态 JSON 字典精准匹配访问者语言偏好并以英文兜底。
]
