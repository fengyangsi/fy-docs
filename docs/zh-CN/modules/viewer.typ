#import "../../fy-spec/lib.typ": *

= viewer 模块：单页阅读器与多语言切换 <sec-viewer>

`viewer` 模块为生成的离线 HTML 提供交互能力与全套明暗主题。

#contract[
  当项目存在多个语言版本时，顶栏自动渲染 `🌐 语言切换下拉菜单`。点击语言选项将直接无缝跳转至同级对应的 `index_<lang>.html` 并自动保留当前章节 Hash 锚点；单语言项目自动隐藏该按钮。
]

#contract[
  页面根元素的 `lang` 描述**正文内容语言**：取 `project` 为该语言目标解析出的内容标签，并按 BCP 47 形态规范化（`pt_BR` → `pt-BR`，基础子标签小写、地区全大写、文字子标签首字母大写）。该标签只来自声明，绝不从正文字形推断。
]

#contract[
  顶栏 chrome 文案仅有中英两套：服务端渲染模板与 `viewer.js` 都按根 `lang` 的 `zh` 前缀各取一套，因此正文语言与 chrome 语言互不干涉——未被翻译的语言以英文 chrome 呈现，根标签仍如实标注该语言。
]

#logic-box[
  文案选择是纯函数 `ui_text(content_lang: &str) -> UiText`：输入只有内容语言，正文不参与；`normalize_lang` 后以 `zh` 开头取中文文案，其余（含空串）一律英文。页面 `lang` 属性直接取 `LanguageTarget.content_lang`，与 `ui_text` 的入参是同一个值。
]

#contract[
  支持 5 大精调明暗主题（Light、Rust、Coal、Navy、Ayu）及跟随系统；顶栏标题与侧栏项目名均支持一键返回封面。
]

#contract[
  代码块支持鼠标悬停一键复制（Copy to Clipboard），搜索面板支持关键词上下文摘要截取与高亮提示。
]
