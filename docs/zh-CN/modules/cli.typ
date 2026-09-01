#import "../../fy-spec/lib.typ": *

= cli 模块：Cargo 外部子命令与交互体系 <sec-cli>

`fy-docs` 以 Cargo 外部子命令发布。安装后，PATH 中的二进制名为 `cargo-fy-docs`，使用者在任何包含 `docs/` 的项目目录中调用：

```bash
cargo fy-docs        # 默认全量构建 HTML 与 PDF 2.0
cargo fy-docs build  # 全量构建所有语言文档
cargo fy-docs html   # 仅编译离线 HTML 网页包
cargo fy-docs pdf    # 仅导出版本化 PDF 2.0 规格说明书
cargo fy-docs dev    # 启动开发工作台，监听源码并热重载
cargo fy-docs init   # 初始化 docs/ 规范目录
cargo fy-docs vendor # 将内嵌 fy-spec 模板同步到 docs/fy-spec/
```

本章规定参数解析、分发顺序、端口分配与退出码路径。两个会写入项目源码目录的子命令 —— `init` 与 `vendor` —— 的设计由 `scaffold` 模块承载；门控所有编译类子命令的那道预检写在本章，因为它是入口拒绝运行之前所依据的条件。

#contract[
  `cargo fy-docs` 是主入口，直接调用可执行文件同样受支持。Cargo 调起外部子命令时的实际命令行是
  `cargo-fy-docs fy-docs …`，因此程序名之后的第一个参数当且仅当它等于 `fy-docs` 时被丢弃；其他任何
  首参都保留，这正是直接调用并携带真实子命令时仍能正确解析的原因。
]

#contract[
  所有选项都是全局选项，故可出现在子命令之前或之后：`--root`、`--lang`、`--open`、`--with-pdf`、
  `--port`、`--no-open`。未给出子命令时，默认命令即全量构建。
]

#contract[
  所有编译类子命令在运行前预检 `typst` CLI：二进制缺失或版本低于 0.14（首个支持
  `--pdf-standard 2.0` 的版本）时立即中止，并给出可操作的报错信息。fy-docs 无法解析的版本横幅不会
  阻断一次可用的安装——编译步骤本身自会暴露真实错误。
]

#contract[
  启动选项（是否随构建附带 PDF、`--lang` 过滤值）一次性捕获进共享状态，此后的每一次生成都读取捕获的选项。默认命令与 `build` 捕获时将 PDF 恒置为开启，`--with-pdf` 在这两个命令下没有独立效果；`html` 与 `dev` 按旗标原样捕获。带过滤的 dev 会话在历次重建间保持过滤。
]

#contract[
  分发顺序固定：`init` 与 `vendor` 在 typst 预检与项目探测之前就返回，其余每个命令先跑预检、再做探测。
  因此 `typst` 缺失或 `docs/` 缺失都不会额外消耗一次编译尝试。
]

#contract[
  `--port` 指定首个候选端口，缺省为 `8181`。端口被占用时沿连续候选重试，最多二十次，最先成功者生效；
  整段无空闲端口时以指明该范围的报错终止。绑定的地址仅为回环网卡。
]

#contract[
  冒泡到入口的错误以单行 `[fy-docs] <error>` 输出到 stderr，进程以非零码退出。分发函数返回退出码而不调用
  终止自身的进程，于是一次失败的命令是可被测试断言的值，而不是被杀掉的测试二进制。
]

#contract[
  `--open` 打开该命令刚刚产出的成品——构建打开根 `index.html`，`pdf` 打开第一个生成的 PDF——且与进程脱离。
  `dev` 除非给出 `--no-open` 否则打开所服务的 URL；打不开的浏览器以“请手工打开此 URL”的形式报告，而不计为失败。
]

#invariant[
  `cargo fy-docs` 默认命令以及 `build`、`html`、`pdf` 子命令均为*一次性幂等构建*，严禁在无交互模式下挂起阻塞 CI 流水线。构建完全成功方以退出码 `0` 退出；编译失败必须以非零退出码结束，确保流水线能拦截损坏的文档。`dev` 模式则始终存活，将失败渲染为错误页。
]

== 模块结构

`src/main.rs` 即全部命令行界面，两个支撑文件折叠进本章。`src/term.rs` 是终端输出模块——`[fy-docs]` 行前缀、忽略写入错误的进度日志（管道关闭时绝不会 panic 掉监听线程）、以及从每个面向用户的路径与转发的诊断信息中剥离 Windows verbatim 路径前缀。`src/lib.rs` 只承载 crate 级文档，不含任何条目。

#logic-box[
  `term::log(message: &str)` 只写 stderr 且忽略结果：每一行进度都是诊断信息，任何命令的成功都不得依赖于
  一个可读的输出管道。机器消费的结构化数据——生成的文件本身——绝不经由它传输。
]
