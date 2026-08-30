#import "../../fy-spec/lib.typ": *

= compiler Module: Typst Compilation & Bundling <sec-compiler>

The `compiler` module invokes the `typst` CLI to produce modern HTML output and PDF 2.0 editions.

#contract[
  PDF compilation passes `--pdf-standard 2.0` to ensure strict compliance with ISO 32000-2:2020.
]

#contract[
  HTML compilation leverages `std::thread::scope` to execute parallel multi-language builds. All language targets reside side-by-side in `docs/target/` (`index_zh-CN.html`, `index_en.html`) with shared assets, and a lightweight client-side routing landing page (`index.html`).
]

#contract[
  A failed language target renders an error page at its own `index_<lang>.html`; successfully built targets keep their fresh pages, and the multi-language landing page is never overwritten by error output.
]

#invariant[
  The root `index.html` landing page generated in multi-language mode must remain under 1KB, never copying body text, and dynamically route visitors based on language priority with English fallback.
]
