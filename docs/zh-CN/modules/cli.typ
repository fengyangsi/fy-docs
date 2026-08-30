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
```

#contract[
  `cargo fy-docs` 是统一的命令入口。Cargo 发现 `cargo-fy-docs` 后会将 `fy-docs` 之后的参数原样转发给该程序。
]

#contract[
  `init` 是唯一不需要既有 `docs/` 目录的子命令：在当前目录创建 `docs/`，写入入口 `main.typ`、
  随二进制内嵌分发的 `fy-spec/lib.typ` 模板副本和空的 `modules/` 目录，并把 `docs/target/` 与
  `docs/release/` 补入 `.gitignore`。
]

#invariant[
  `cargo fy-docs` 默认命令以及 `build`、`html`、`pdf` 子命令均为*一次性幂等构建*，构建完成后必须以退出码 `0` 安全退出，严禁在无交互模式下挂起阻塞 CI 流水线。
]
