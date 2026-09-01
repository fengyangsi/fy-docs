#import "../../fy-spec/lib.typ": *

= compiler Module: Typst Compilation & Bundling <sec-compiler>

The `compiler` module invokes the `typst` CLI to produce modern HTML output and PDF 2.0 editions. It is the only module that launches another program at run time. Its inputs are the `project` chapter's targets and paths plus the startup options captured by `cli`; the `page` chapter supplies it the markup shell and the asset bytes, and the pages it writes are the ones the `viewer` chapter's scripts run inside.

#contract[
  PDF compilation passes `--pdf-standard 2.0` to ensure strict compliance with ISO 32000-2:2020.
]

#contract[
  When `typst` exits successfully its stderr is forwarded to fy-docs' stderr, keeping font substitution and HTML-export degradation visible. Unknown-font-family warnings collapse into one line per family; every other warning passes through verbatim. Windows verbatim path prefixes are stripped from forwarded diagnostics.
]

#contract[
  When the HTML export's root `lang` disagrees with the content language resolved for that target, the build warns on stderr naming both tags (a `docs/zh-CN/main.typ` declaring `lang: "en"` is the case in point: the page calls itself `zh-CN` while Typst typesets English). The resolved content language is never overwritten by the export value.
]

#logic-box[
  `extract_root_lang(html: &str) -> Option<String>` reads the `lang="..."` inside the opening `<html ...>` tag only: a `lang` attribute in the body (`<p lang="zh">`) and a root tag without `lang` both yield `None`. `language_drift(target: &LanguageTarget, exported: Option<&str>) -> Option<String>` compares both sides after `normalize_lang` and returns warning text only when both are present and differ. The warning goes to stderr through `term::log`; it never rewrites `content_lang` and never changes the exit code.
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

#contract[
  The root `index.html` after a build is decided in one place: a language target whose output file name already is `index.html` owns the root page; otherwise a single-language project's only rendered page is assembled a second time, from the same inputs, under `index.html`; otherwise the client-side routing page is written. The `--with-pdf`-failure path reuses the same decision but only seeds the routing page when no landing page exists, never overwriting one a previous build left.
]

#contract[
  A parallel compile thread that panics is collected rather than allowed to abort the process: the panic text becomes the build failure and every selected target degrades to an error page.
]

#contract[
  Every artifact reaches the disk through the same atomic path — content into a sibling temporary file, then a rename over the destination. No page, stylesheet, or script is written in place, so a concurrent HTTP read from the dev server cannot observe a half-written file.
]

#contract[
  The `page` chapter owns the asset *bytes* and this module owns their *file names*: `compiler` decides that the base stylesheet is `fy-docs.css`, the merged export sheet is `typst.css`, the reader script is `fy-docs.js`, and the reload client is `live.js`, and writes each one beside the pages in `docs/target/`.
]

#invariant[
  The names this module writes and the names `assets/doc.html` references in its `link` and `script` tags are the same names. Nothing at run time can detect a mismatch — a renamed asset silently ships an unstyled, non-interactive page — so a rename on one side is a rename on both, in the same commit.
]

#contract[
  Multi-language HTML compilation folds the per-export CSS into one `typst.css`: the first non-empty export seeds the sheet, an export whose CSS the combined text already contains contributes nothing, and a genuinely different sheet is appended under a marker comment. Containment decides equality, so languages that typeset identically keep exactly one copy of the rules. An error page seeds `typst.css` only when the file is absent, because in a partial failure the merged sheet of the successful targets is already on disk and must survive.
]

#contract[
  A compile failure is rendered through the same shell as a success: the error body is built from the `page` chapter's `ui_text` set for that target's content language and its HTML escaper, so an error page carries the toolbar, the theme menu, and the language switcher instead of a bare diagnostic. A failure to write one target's error page is logged and never aborts the remaining targets.
]

#contract[
  Any generation that completes its HTML assembly — including one where some language targets failed — ensures `/docs/target/` is listed in the project's `.gitignore` through the shared `project` helper, so generated output never asks the user to edit ignore rules by hand. A generation aborted by the PDF stage never touches that file.
]

== Module Structure

The module splits by concern into five files under `src/compiler/`:

#figure(
  table(
    columns: (auto, auto),
    inset: 6pt,
    align: (auto, left),
    table.header([*File*], [*Concern*]),
    [`mod.rs`], [Build orchestration: `generate` reports progress and turns a failure into the visible `FAILED` line and the non-zero signal, delegating to the private `generate_pages`, which runs the PDF stage, the parallel HTML export, the asset writing and the root `index.html` decision; `select_targets`, the target label, the panic-payload formatter and the ignore-rule wrapper live here too],
    [`typst.rs`], [The process boundary: the precheck with its 0.14 floor, the HTML and PDF invocations, stderr forwarding, and panic collection],
    [`extract.rs`], [`ExtractedPage` (`title`, `styles`, `body`) and the tolerant HTML decomposition: root tag language, body, style blocks, and the language-drift check],
    [`warnings.rs`], [typst stderr shaping: warning-block splitting and unknown-font-family folding],
    [`output.rs`], [Artifact writing: atomic writes, the guaranteed landing page, error pages, temporary sweeping, and multi-language style merging],
  ),
  caption: [The compiler module's internal layout.],
)
