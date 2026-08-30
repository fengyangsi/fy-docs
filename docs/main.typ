#import "fy-spec/lib.typ": *

#show: project_book.with(
  title: "fy-docs 规格说明书",
  subtitle: "Typst 规格文档的本地构建、阅读与实时预览工具",
  version: "0.1.4",
  author: "fengyangsi",
  date: "2026-08-30",
)

#include "modules/project.typ"
#include "modules/cli.typ"
#include "modules/compiler.typ"
#include "modules/server.typ"
#include "modules/viewer.typ"
