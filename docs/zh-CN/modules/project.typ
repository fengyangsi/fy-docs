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
  `--lang <LANG>` 的过滤值在匹配前先归一化：大小写不敏感，且 `_` 与 `-` 等价，因此 `zh_CN`、`ZH-cn`、`zh-cn` 指向同一个语言目标。归一化后按*完整语言码*精确匹配，不做前缀回退：`--lang zh` 不会命中 `zh-CN` 目标。
]

#contract[
  根 `docs/main.typ` 注册的 `default` 目标恒被包含：任何 `--lang <LANG>` 过滤都会在其请求的语言之外一并选中 default 目标并编译。需要按语言严格隔离的项目应只保留语言目录（不放根 `main.typ`）。
]

#contract[
  归一化后仍无任何语言目标命中时，构建以非零退出码失败并列出该项目实际可用的语言。`--lang` 拼错绝不允许静默退化为"只构建 default 目标"。
]

#contract[
  语言目录判定采用正向规则：`docs/` 下任何自带 `main.typ` 的子目录即一个语言目标，仅生成目录（`target/`、`release/`）除外。共享源目录（如 `fy-spec/`、`modules/`）无需登记名单，因为它们本就不含 `main.typ`。
]

#contract[
  清单未给版本时回退读取入口 `main.typ` 的 `version:` 实参，且只认未被 `//` 注释的代码，散文或停用示例中的版本号不得被当作项目版本。
]

#contract[
  Typst root 反推所用的绝对导入扫描尊重词法边界：单双引号均可识别，`//` 之后的内容视为注释忽略，因此被注释掉的 `#import` 不会把 root 拖到错误的祖先目录。
]

#invariant[
  Typst 沙盒根目录（`root`）自动探测能覆盖全部绝对导入的最接近祖先目录；用户亦可通过 `--root <DIR>` 显式锁定。
]
