#import "../../fy-spec/lib.typ": *

= compiler Module: Typst Compilation & Bundling <sec-compiler>

The `compiler` module invokes the `typst` CLI to produce modern HTML output and PDF 2.0 editions.

#contract[
  PDF compilation passes `--pdf-standard 2.0` to ensure strict compliance with ISO 32000-2:2020.
]

#contract[
  When `typst` exits successfully, everything it wrote to stderr (unknown font families, directives ignored during HTML export) is forwarded to fy-docs' stderr instead of discarded, so silent font substitution and degraded rendering stay visible. The one exception is the unknown-font-family warning: the fallback chain lists candidates from several operating systems and typst repeats every unavailable family at each style site, so those collapse into one deduplicated line while all other warnings pass through verbatim.
]

#contract[
  Every build first sweeps `docs/target/`: artifacts fy-docs no longer writes (the polling-era `_poll.js` client and its `_build` marker) and `_temp_*.html` intermediates left by a killed compile, so an upgraded project never keeps serving retired files.
]

#contract[
  When the `--with-pdf` stage fails, the root `index.html` must still exist after the per-language error pages are written: in a pure i18n project `index.html` belongs to no single target, and without it the dev server has no route for `/`.
]

#contract[
  HTML compilation leverages `std::thread::scope` to execute parallel multi-language builds. All language targets reside side-by-side in `docs/target/` (`index_en.html`, `index_zh-CN.html`) with shared assets, and a lightweight client-side routing landing page (`index.html`).
]

#contract[
  A failed language target renders an error page at its own `index_<lang>.html`; successfully built targets keep their fresh pages, and the multi-language landing page is never overwritten by error output.
]

#invariant[
  The root `index.html` landing page generated in multi-language mode must remain under 1KB, never copying body text, and dynamically route visitors based on language priority with English fallback.
]
