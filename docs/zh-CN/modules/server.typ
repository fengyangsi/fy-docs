#import "../../fy-spec/lib.typ": *

= server 模块：Axum 开发工作台与热重载 <sec-server>

`server` 模块仅在 `cargo fy-docs dev` 命令下启动，提供本地 HTTP 预览与自动文件监听热重载。

#contract[
  `server` 在 `127.0.0.1:8181`（端口冲突时自动顺延递增）启动 Axum Web 服务，并注册 `_poll.js` 轮询端点；当 `watcher` 捕获 `.typ` 源码变动并重构完成后，页面自动平滑刷新。
]
