#import "../../fy-spec/lib.typ": *

= server 模块：Axum 开发工作台与热重载 <sec-server>

`server` 模块仅在 `cargo fy-docs dev` 命令下启动，提供本地 HTTP 预览与自动文件监听热重载。

#contract[
  `server` 在 `127.0.0.1:8181`（端口冲突时自动顺延递增）启动 Axum Web 服务，并注册 `/events` SSE（Server-Sent Events）端点；当 `watcher` 捕获 `.typ` 源码变动并重构完成后，端点推送新的构建编号，打开的页面随即自动刷新。静态构建同样携带 `live.js` 客户端，连接失败时静默关闭。
]

#contract[
  文件监听以去抖窗口合并变更：一段静默期内的连续保存只触发一次构建，但等待自首次变更起设有总上限，持续写入源文件的外部进程不得让重建被无限推迟。
]
