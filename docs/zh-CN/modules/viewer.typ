#import "../../fy-spec/lib.typ": *

= viewer 模块：单页阅读器与多语言切换 <sec-viewer>

`viewer` 模块为生成的离线 HTML 提供交互能力与全套明暗主题。

#contract[
  当项目存在多个语言版本时，顶栏自动渲染 `🌐 语言切换下拉菜单`。点击语言选项将直接无缝跳转至同级对应的 `index_<lang>.html`；单语言项目自动隐藏该按钮。
]

#contract[
  支持 5 大精调明暗主题（Light、Rust、Coal、Navy、Ayu）及跟随系统；顶栏标题与侧栏项目名均支持一键返回封面。
]
