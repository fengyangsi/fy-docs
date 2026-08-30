#import "../lib.typ": *

#show: project_book.with(
  title: "fy-spec 示例规格说明书",
  subtitle: "共享模板组件与排版验证",
  version: "0.1.0",
  author: "fengyangsi",
)

= 设计目标

#status-badge(status: "已确立", phase: "模板验证")

`fy-spec` 为 fy 生态提供统一、可版本化的规格书设计系统。

== 契约

#contract[
```rust
pub trait Specification {
    fn version(&self) -> &str;
}
```
]

#invariant[
- 文档源码只有一个当前版本。
- 模板实现只有一个权威来源。
]

#example-box[
所有 fy 项目通过同一个 `lib.typ` 生成规格书。
]
