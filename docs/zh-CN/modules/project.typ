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
  每个语言目标携带一个*内容语言*标记，用于 `<html lang>` 与界面文案，解析顺序为：语言目录名 → 入口 `main.typ` 的 `lang:` 实参 → `en`（与 fy-spec 模板的默认声明同值）。语言目录名总是胜出，即使入口内的 `lang:` 与它不同。
]

#contract[
  内容语言只来自声明，绝不从正文字形推断。`version:` 与 `lang:` 由同一个模板实参解析器读取：逐行扫描、丢弃 `//` 之后的内容、取该实参后首个引号内的值，并要求实参名前为标识符边界，故 `sub-lang:` 这类同名后缀不参与匹配。
]

#contract[
  `LanguageTarget` 的字段：`lang`（语言目录名，根 `main.typ` 注册的 default 目标为空串）、`content_lang`（该目标内容语言的 BCP 47 规范化标签）、`display_name`（语言切换菜单显示名）、`entry`（入口 `.typ` 路径）、`html_file_name`、`pdf_file_name`。`content_lang` 由 `detect_language_targets` 在构造目标时一次性写入，与 `version` 同源（同样读自入口源码），因此进程存续期内不随源码再编辑而变。
]

#contract[
  若 typst HTML 导出的根 `lang` 与解析出的内容语言不一致，构建在 stderr 输出一条告警（详见 `compiler` 模块）；本页与其同语言 PDF 使用的仍是此处解析出的标签，导出值不覆盖它。
]

#logic-box[
  实参扫描器签名 `parse_template_argument(text: &str, key: &str) -> Option<String>`，按 key 包装为 `main_typ_version(entry: &Path)` 与 `main_typ_lang(entry: &Path)`。前置条件：`text` 为入口文件完整源码。后置条件：返回值是源码中*首个*满足「位于未注释代码内、实参名前是标识符边界、其后同一行存在成对引号」的 `key:` 的引号内文本；任一条件不满足则继续扫描下一处同名实参，全部落空返回 `None`，绝不猜测。
]

#logic-box[
  内容语言解析器签名 `resolve_content_lang(dir_name: &str, entry: &Path) -> String`。`dir_name` 为语言目录名，default 目标传入空串。后置条件：返回值恒为非空的展示形态 BCP 47 标签；`dir_name` 非空时由其单独决定，否则取入口的 `lang:` 实参，两者皆无声明时返回 `en`。该函数全域且无副作用：入口不可读时返回默认值而非报错。
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
