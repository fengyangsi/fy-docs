#import "../../fy-spec/lib.typ": *

= compiler 模块：Typst 编译与资产生成 <sec-compiler>

`compiler` 模块驱动 `typst` CLI 执行 HTML 导出与 PDF 2.0 渲染，并拼接前端阅读器资产。

#contract[
  PDF 编译时传入 `--pdf-standard 2.0` 参数，严格遵循 ISO 32000-2:2020 国际规范导出 PDF 2.0 发行本。
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
