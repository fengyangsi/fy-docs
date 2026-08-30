#import "../../fy-spec/lib.typ": *

= project 模块：多语言目标与项目探测 <sec-project>

`project` 模块负责从当前工作目录解析项目元数据、多语言文档结构及 Typst 编译沙盒边界。

```text
docs/
├── fy-spec/lib.typ      # 共享规范模板
├── zh-CN/               # 简体中文入口 (docs/zh-CN/main.typ)
├── en/                  # 英文入口 (docs/en/main.typ)
└── target/              # 生成的离线静态页面
```

#contract[
  `Project` 自动探测单语言（`docs/main.typ`）与多语言（`docs/<lang>/main.typ`）目录结构。对于多语言项目，每个语言目录作为一个独立的 `LanguageTarget`，在编译时分别生成 `index_<lang>.html` 与版本化 PDF。
]

#contract[
  根 `docs/main.typ` 注册的 `default` 目标恒被包含：任何 `--lang <LANG>` 过滤都会在其请求的语言之外一并选中 default 目标并编译。需要按语言严格隔离的项目应只保留语言目录（不放根 `main.typ`）。
]

#invariant[
  Typst 沙盒根目录（`root`）自动探测能覆盖全部绝对导入的最接近祖先目录；用户亦可通过 `--root <DIR>` 显式锁定。
]
