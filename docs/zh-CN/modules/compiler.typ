#import "../../fy-spec/lib.typ": *

= compiler 模块：Typst 编译与资产生成 <sec-compiler>

`compiler` 模块驱动 `typst` CLI 执行 HTML 导出与 PDF 2.0 渲染，并拼接前端阅读器资产。只有本模块会在运行期启动另一个程序。它的输入是 `project` 章给出的语言目标与路径，加上 `cli` 捕获的启动选项；`page` 章向它提供标记外壳与资产字节，而它写出的页面正是 `viewer` 章脚本运行于其中的那些页面。

#contract[
  PDF 编译时传入 `--pdf-standard 2.0` 参数，严格遵循 ISO 32000-2:2020 国际规范导出 PDF 2.0 发行本。
]

#contract[
  `typst` 成功退出时，其 stderr 转发到 fy-docs 的 stderr，使字体替换与 HTML 导出降级对用户保持可见。未知字体族告警按字体族去重为一行，其余告警原样透传；转发的诊断文本中 Windows verbatim 路径前缀被剥离。
]

#contract[
  HTML 导出根标签的 `lang` 与该语言目标解析出的内容语言不一致时，构建在 stderr 输出一条同时给出两个标签的 warning（例如 `docs/zh-CN/main.typ` 内声明 `lang: "en"`：页面自称 `zh-CN`，Typst 却按英文排版）。解析出的内容语言不被导出值覆盖。
]

#logic-box[
  `extract_root_lang(html: &str) -> Option<String>` 只解析起始 `<html ...>` 标签内的 `lang="..."`：正文里的 `lang` 属性（`<p lang="zh">`）与不带 `lang` 的根标签都返回 `None`。`language_drift(target: &LanguageTarget, exported: Option<&str>) -> Option<String>` 将两侧经 `normalize_lang` 归一化后比较，仅在同时存在且不等时返回告警文本；告警经 `term::log` 写入 stderr，不改写 `content_lang`，也不影响退出码。
]

#contract[
  构建开始时清除上一进程被强制中断遗留的 `_temp_*.html` 编译中间文件。
]

#contract[
  `--with-pdf` 失败时，写出各语言错误页后仍保证根 `index.html` 存在。纯多语言项目的 `index.html` 由路由分流页承担，不属于任何单一语言目标。
]

#contract[
  HTML 编译支持多语言并发渲染（基于 `std::thread::scope` 并发调度 Typst CLI）。所有语言页面在 `docs/target/` 下同级生成（`index_zh-CN.html`、`index_en.html`），统一共享静态资产，并在根路径生成轻量客户端路由分流页 `index.html`。
]

#contract[
  编译失败的语言目标在其自身的 `index_<lang>.html` 渲染错误页；构建成功的语言保留最新产物，多语言路由分流页绝不被错误输出覆盖。
]

#invariant[
  多语言模式下生成的根 `index.html` 仅包含跳转脚本与语言清单，绝不复制正文；体积按每语言约 60 字节线性增长，五语言时仍小于 1.4KB。
]

#contract[
  构建后的根 `index.html` 由单一位置决策：输出文件名本就是 `index.html` 的语言目标直接拥有根页面；否则单语言项目以其唯一渲染页的同一组输入，再装配一份以 `index.html` 为名的页面；否则写入客户端路由分流页。`--with-pdf` 失败路径复用同一决策，但仅在无着陆页时补写路由分流页，绝不覆盖上一次构建留下的页面。
]

#contract[
  并行编译线程中的 panic 被收集而不听任其中止进程：panic 文本即成为构建失败原因，且所有被选中的目标一律降级为错误页。
]

#contract[
  每个产物都经同一条原子路径落盘——内容先写入同级临时文件，再改名覆盖目标。任何页面、样式表或脚本都不做原地写入，于是 dev 服务器并发的 HTTP 读取绝不会看到写了一半的文件。
]

#contract[
  资产的*字节*归 `page` 章，其*文件名*归本模块：`compiler` 决定基础样式表叫 `fy-docs.css`、合并后的导出样式叫 `typst.css`、阅读器脚本叫 `fy-docs.js`、热重载客户端叫 `live.js`，并把每个文件写到 `docs/target/` 中页面的旁边。
]

#invariant[
  本模块写出的名字与 `assets/doc.html` 在 `link`、`script` 标签里引用的名字就是同一批名字。运行期没有任何机制能发现两者不一致——被改名的资产只会静默地交付一页无样式、无交互的文档——因此一侧改名即两章同时改名，且在同一个 commit 内。
]

#contract[
  多语言 HTML 编译把各次导出的 CSS 折进单一的 `typst.css`：首个非空导出即为整表之种，其 CSS 已被合并文本包含的导出不再贡献内容，确实不同的样式表则带一条标记注释追加在后。包含关系即判定相等，于是排版完全一致的多语言只保留一份规则。错误页仅在 `typst.css` 缺失时才为其播种，因为部分失败时磁盘上已是成功目标合并好的样式表，必须存活。
]

#contract[
  编译失败与成功渲染同一层外壳：错误正文由 `page` 章针对该内容语言选定的 `ui_text` 与其 HTML 转义器构成，因此错误页照样携带顶栏、主题菜单与语言切换，而不是一行裸诊断。某个目标的错误页写不出来时只记录日志，绝不中断其余目标。
]

#contract[
  凡是走完 HTML 拼装阶段的生成——包括部分语言失败的那些——都会经 `project` 章的共享辅助函数确保 `/docs/target/` 已列入项目 `.gitignore`，生成产物不得要求使用者手工编辑忽略规则。被 PDF 阶段中止的生成从不触碰该文件。
]

== 模块结构

`compiler` 模块按职责拆分为 `src/compiler/` 下的五个文件：

#figure(
  table(
    columns: (auto, auto),
    inset: 6pt,
    align: (auto, left),
    table.header([*文件*], [*职责*]),
    [`mod.rs`], [构建编排：`generate` 负责进度上报并把失败转成可见的 `FAILED` 行与非零退出信号，实际工作交给私有函数 `generate_pages`——PDF 阶段、并行 HTML 导出、资产落盘与根 `index.html` 决策都在其中；`select_targets`、目标语言标签、panic 文本格式化与忽略规则包装同样位于此文件],
    [`typst.rs`], [进程边界：0.14 版本下限预检、HTML 与 PDF 调用、stderr 转发、panic 收集],
    [`extract.rs`], [`ExtractedPage`（`title`、`styles`、`body`）与宽容的 HTML 拆解：根标签语言、正文、样式块、语言漂移检查],
    [`warnings.rs`], [typst stderr 整形：告警块切分与未知字体族折叠],
    [`output.rs`], [产物落盘：原子写入、保证存在的着陆页、错误页、临时文件清扫、多语言样式合并],
  ),
  caption: [compiler 模块的内部布局。],
)
