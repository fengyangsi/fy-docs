#import "../../fy-spec/lib.typ": *

= compiler Module: Typst Compilation & Bundling <sec-compiler>

The `compiler` module invokes the `typst` CLI to produce modern HTML output and PDF 2.0 editions.

#contract[
  PDF compilation passes `--pdf-standard 2.0` to ensure strict compliance with ISO 32000-2:2020.
]

#contract[
  When `typst` exits successfully its stderr is forwarded to fy-docs' stderr, keeping font substitution and HTML-export degradation visible. Unknown-font-family warnings collapse into one line per family; every other warning passes through verbatim. Windows verbatim path prefixes are stripped from forwarded diagnostics.
]

#contract[
  A build removes the `_temp_*.html` compile intermediates an interrupted process left behind.
]

#contract[
  When the `--with-pdf` stage fails, the per-language error pages are written and the root `index.html` is still guaranteed to exist. In a pure i18n project `index.html` is the routing landing page and belongs to no single language target.
]

#contract[
  HTML compilation leverages `std::thread::scope` to execute parallel multi-language builds. All language targets reside side-by-side in `docs/target/` (`index_en.html`, `index_zh-CN.html`) with shared assets, and a lightweight client-side routing landing page (`index.html`).
]

#contract[
  A failed language target renders an error page at its own `index_<lang>.html`; successfully built targets keep their fresh pages, and the multi-language landing page is never overwritten by error output.
]

#invariant[
  The root `index.html` landing page generated in multi-language mode carries only the redirect script and the language list, never body text. Its size grows by roughly 60 bytes per language and stays under 1.4KB with five languages.
]
