#import "../../fy-spec/lib.typ": *

= compiler Module: Typst Compilation & Bundling <sec-compiler>

The `compiler` module invokes the `typst` CLI to produce modern HTML output and PDF 2.0 editions.

#contract[
  PDF compilation passes `--pdf-standard 2.0` to ensure strict compliance with ISO 32000-2:2020.
]

#contract[
  HTML compilation generates all language targets under `docs/target/` (`index_zh-CN.html`, `index_en.html`) with shared static assets (`fy-docs.css`, `fy-docs.js`).
]
