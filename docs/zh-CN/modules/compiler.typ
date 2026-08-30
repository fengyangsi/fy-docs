#import "../../fy-spec/lib.typ": *

= compiler 模块：Typst 编译与资产生成 <sec-compiler>

`compiler` 模块驱动 `typst` CLI 执行 HTML 导出与 PDF 2.0 渲染，并拼接前端阅读器资产。

#contract[
  PDF 编译时传入 `--pdf-standard 2.0` 参数，严格遵循 ISO 32000-2:2020 国际规范导出 PDF 2.0 发行本。
]

#contract[
  HTML 编译支持多语言并发/按序渲染。所有语言页面在 `docs/target/` 下同级生成（`index_zh-CN.html`、`index_en.html`），并统一共享 `fy-docs.css` 与 `fy-docs.js`，绝不重复生成多份样式表副本。
]
