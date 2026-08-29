#import "/fy-spec/lib.typ": *

= compiler 模块：Typst 构建与交付物

`compiler` 调用 Typst，并将导出的语义 HTML 组装为可离线打开的阅读页。它不解析文档语义，只提取标题、样式与正文，再注入固定的阅读器外壳。

== HTML 输出契约

```text
docs/main.typ
    │ typst compile --features html
    ▼
docs/target/
├── index.html       # 嵌入正文的离线阅读页
├── typst.css        # Typst 导出样式
├── fy-docs.css      # 主题与版式
├── fy-docs.js       # 原生阅读器交互
└── _poll.js         # 静态模式为无操作脚本
```

#invariant[
  任何一次构建都必须同时写入 HTML、阅读器 CSS 与 JavaScript。缺失 `fy-docs.js` 会使章节阅读器失效，因此它是与页面模板同级的必需静态资产。
]

== PDF 输出契约

`pdf` 与 `build --with-pdf` 使用 Typst 生成打印版，写入 `docs/release/<包名>_v<版本>_specification.pdf`。HTML 阅读页不链接或复制 PDF，避免网页构建与 PDF 构建在不同时间产生版本错配。
