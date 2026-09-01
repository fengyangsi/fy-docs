#import "../../fy-spec/lib.typ": *

= compiler 模块：Typst 编译与资产生成 <sec-compiler>

`compiler` 模块驱动 `typst` CLI 执行 HTML 导出与 PDF 2.0 渲染，并拼接前端阅读器资产。

#contract[
  PDF 编译时传入 `--pdf-standard 2.0` 参数，严格遵循 ISO 32000-2:2020 国际规范导出 PDF 2.0 发行本。
]

#contract[
  `typst` 成功退出时，其 stderr 转发到 fy-docs 的 stderr，使字体替换与 HTML 导出降级对用户保持可见。未知字体族告警按字体族去重为一行，其余告警原样透传；转发的诊断文本中 Windows verbatim 路径前缀被剥离。
]

#contract[
  构建开始时清除上一进程被强制中断遗留的 `_temp_*.html` 编译中间文件。
]

#contract[
  `--with-pdf` 失败时，写出各语言错误页后仍保证根 `index.html` 存在。纯多语言项目的 `index.html` 由路由分流页承担，不属于任何单一语言目标。
]

#contract[
  HTML 编译支持多语言并发渲染（基于 `std::thread::scope` 并发调度 Typst CLI）。所有语言页面在 `docs/target/` 下同级生成（`index_zh-CN.html`、`index_en.html`），统一共享静态资产，并在根路径生成轻量客户端路由分流页 `index.html`。
]

#contract[
  编译失败的语言目标在其自身的 `index_<lang>.html` 渲染错误页；构建成功的语言保留最新产物，多语言路由分流页绝不被错误输出覆盖。
]

#invariant[
  多语言模式下生成的根 `index.html` 仅包含跳转脚本与语言清单，绝不复制正文；体积按每语言约 60 字节线性增长，五语言时仍小于 1.4KB。
]
