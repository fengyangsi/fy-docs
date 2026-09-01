#import "../../fy-spec/lib.typ": *

= scaffold 模块：工程骨架与模板分发 <sec-scaffold>

`scaffold` 模块独占仅有的两个写入项目*源码树*（而非生成产物）的操作：从零创建 `docs/` 目录，以及刷新项目私有的 fy-spec 模板副本。命令面——`init`、`vendor`、`vendor --check`——由 `cli` 模块声明，本章规定这些命令如何构建与校验它们的产物。

#contract[
  一个文件是所有模板副本的唯一来源：内嵌库就是本仓库自己的 `docs/fy-spec/lib.typ`，在编译期内嵌进二进制。因此二进制、本仓库的自举构建、以及每个项目落地的副本三者字节相同；`vendor --check` 也按*完全相等*比对——不做注释、空白或行尾归一化，一份被格式化过的副本即判定为漂移。
]

#contract[
  当 `docs/main.typ` 已是一个文件时，`init` 中止并报出该路径，要求使用者先移除它。它不检查其他任何既有状态，也从不修改或删除文件：一次拒绝必须让目录树保持原样。
]

#contract[
  一次成功的 `init` 恰好创建三项产物——`docs/main.typ`、`docs/fy-spec/lib.typ`、以及目录 `docs/modules/`——并使用递归建目录操作，所以 `docs/` 不存在是常态而非错误。`docs/modules/` 保持为空：那是项目安放自己章节的位置，不是工具可以代写的东西。
]

#contract[
  起始 `docs/main.typ` 就是内置模板文本，仅替换 `{{NAME}}`、`{{VERSION}}`、`{{AUTHOR}}` 三处占位符；其余字节一律原样保留，包括相对路径的 `#import "fy-spec/lib.typ"`、`lang: "en"` 声明和被注释掉的 `#include`。`init` 是唯一不需要既有 `docs/` 目录即可运行的命令。
]

#contract[
  `init` 与 `vendor` 都不要求 `typst` 二进制，也不要求可探测的项目：两者都在 typst 预检与项目探测之前就完成返回，因此在一台尚未编译出任何文档的目录中同样成功。
]

#contract[
  `vendor` 要求 `docs/` 目录已存在，缺失时失败并提示先运行 `init`。随后它无条件（重）写 `docs/fy-spec/lib.typ`，必要时创建 `docs/fy-spec/`——写入是幂等的，也绝不以"检测到漂移"为由拒绝，因为用内嵌模板覆盖*本身就是*修复动作。
]

#contract[
  `vendor --check` 不写入任何文件。文件缺失、或文件字节与内嵌模板不一致，都以非零码退出并报出该文件与可修复它的命令；字节完全一致则记录一条匹配日志并退出 `0`——项目 CI 正是凭这一条钉住模板版本。
]

#contract[
  `init` 通过构建命令共用的同一个辅助函数，把 `/docs/target/` 与 `/docs/release/` 追加进项目的 `.gitignore`：条目缺失时才追加，写入失败只记为告警而不视为错误。
]

#logic-box[
  `init(cwd: &Path) -> Result<()>` 与 `vendor(cwd: &Path, check: bool) -> Result<()>` 对输入是全域的，且不接收选项结构体：`cwd` 是 dispatch 已经解析好的规范化工作目录。起始模板的三个字段按下列顺序解析，顺序是刻意安排的——`Cargo.toml` 优先，因为它是版本号的唯一真身：
  - *name*：Cargo 包名，其次 `cwd` 的目录名，最后 `project`。
  - *version*：Cargo 包版本，其次 `0.1.0`。
  - *author*：`authors` 的首个条目，其次 `TODO`。
]

#contract[
  `init` 的版本解析阶梯没有中间一级：与 `project` 探测不同，它从不从 `main.typ` 读取 `version:` 参数，因为 `init` 时刻该文件还不存在。两条阶梯的差异是设计而非疏漏，不得合并。
]

#contract[
  清单未声明 `authors` 时，作者解析结果是字面量 `TODO`，绝不取用任何来自 fy-docs 自身的名字：起始文档属于使用者，脚手架无权在上面署名。
]

#invariant[
  `cargo fy-docs init` 之后紧接 `cargo fy-docs` 即可编译成功，无需使用者改动一个字符：脚手架产出的是一套可用工程，而不是需要手工收尾的样本。一条集成测试正是拿真实二进制跑这一对命令来锁住该不变量。
]
