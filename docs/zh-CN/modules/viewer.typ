#import "../../fy-spec/lib.typ": *

= viewer 模块：单页阅读器与多语言切换 <sec-viewer>

`viewer` 模块为生成的离线 HTML 提供交互能力与全套明暗主题。

#contract[
  当项目存在多个语言版本时，顶栏自动渲染 `🌐 语言切换下拉菜单`。点击语言选项将直接无缝跳转至同级对应的 `index_<lang>.html` 并自动保留当前章节 Hash 锚点；单语言项目自动隐藏该按钮。
]

#contract[
  页面根元素的 `lang` 描述**正文语言**：具名语言目标输出其规范化 BCP 47 标签（`pt_BR` → `pt-BR`，基础子标签小写、地区全大写、文字子标签首字母大写），无标签的 default 目标按正文推断（含 CJK 表意文字取 `zh-CN`，否则 `en`）。顶栏 chrome 文案仅有中英两套，由 `viewer.js` 依 `lang` 的语言前缀择一渲染，因此正文语言与 chrome 语言互不干涉：未被翻译的语言以英文 chrome 呈现，根标签仍如实标注该语言。
]

#contract[
  支持 5 大精调明暗主题（Light、Rust、Coal、Navy、Ayu）及跟随系统；顶栏标题与侧栏项目名均支持一键返回封面。
]

#contract[
  代码块支持鼠标悬停一键复制（Copy to Clipboard），搜索面板支持关键词上下文摘要截取与高亮提示。
]
