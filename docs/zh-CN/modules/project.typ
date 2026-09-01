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
  归一化后仍无任何语言目标命中时，构建以非零退出码失败并列出该项目实际可用的语言。
]

#contract[
  语言目录判定采用正向规则：`docs/` 下任何自带 `main.typ` 的子目录即一个语言目标，仅生成目录（`target/`、`release/`）除外。
]

#contract[
  清单未给版本时，回退读取入口 `main.typ` 中 `version:` 实参，且只认未被 `//` 注释的代码；注释内的版本号不参与版本推导。
]

#contract[
  Typst root 反推所用的绝对导入扫描尊重词法边界：单双引号均可识别，`//` 之后的内容视为注释忽略，因此被注释掉的 `#import` 不会把 root 拖到错误的祖先目录。
]

#contract[
  语言目录遵循 BCP 47 语义分层：基础语言子标签决定翻译内容，地区或文字子标签*仅在其承载真实差异时*追加（`zh-CN`/`zh-TW`、`pt-BR`/`pt-PT`）。`en` 作为中性英文根语言不带地区后缀，严禁为拼写差异分叉整份文档。新增语言必须同步登记 `lang_display_name` 映射表与模板 `i18n-strings` 的基础语言键。
]

#contract[
  未登记的语言码按 BCP 47 形态规范化后展示：基础子标签小写、地区子标签全大写、文字子标签首字母大写（`pt_BR` → `pt-BR`，`zh_hant_tw` → `zh-Hant-TW`）。fy-docs 绝不臆造它无法确知的语言名称。
]

#invariant[
  Typst 沙盒根目录（`root`）自动探测能覆盖全部绝对导入的最接近祖先目录；用户亦可通过 `--root <DIR>` 显式锁定。
]
