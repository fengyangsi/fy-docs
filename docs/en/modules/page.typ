#import "../../fy-spec/lib.typ": *

= page Module: Page Assembly & Escaping <sec-page>

The `page` module assembles the reading page: it holds the markup shell, the chrome label tables, and the escaping ladder, and it turns one compiled document body into the file a browser opens. It runs at generation time. The `compiler` module owns the typst process and the names under which artifacts land; the `viewer` module owns the code that later runs inside a browser on the markup this module emits.

#contract[
  `assets/doc.html` is the shell, and rendering is a token fill over it. The token set is exactly: `TITLE`, `NAME`, `LANG`, `SIDEBAR_TOGGLE`, `THEME`, `SYSTEM_THEME`, `SEARCH`, `SEARCH_DOCUMENT`, `SEARCH_PLACEHOLDER`, `PRINT`, `TABLE_OF_CONTENTS`, `GITHUB_LINK`, `LANG_MENU`, `BODY`.
]

#contract[
  A rendered page contains no `{{` sequence. The shell may introduce a token only together with the substitution that fills it, so the two files never drift apart.
]

#contract[
  Each token is written with the escaper its context demands: `TITLE` and `NAME` are HTML-text escaped, `LANG` carries the target's content language as already normalized by the `project` module, `BODY` is the compiled document inserted verbatim, and the UI label tokens come from a fixed internal table rather than from any input. `GITHUB_LINK` and `LANG_MENU` are markup this module builds itself, already escaped at the leaf.
]

#invariant[
  `assets/doc.html` is emitted by fy-docs and is never supplied by the user. The `id` and `class` surface it declares is therefore an internal invariant, not an optional input: a page shell that is missing or renames one of them is a defect in this module, and `assets/viewer.js` is entitled to bind to every one of them without defending against its absence.
]

#contract[
  The first-paint theme is decided by an inline script in the shell, before any external stylesheet or script loads, so a page never flashes the wrong palette: a stored theme is applied directly, an unset or legacy value falls back to the system preference, and every theme class the shell can name is cleared before one is set.
]

#contract[
  The generated page is self-contained relative to its own output directory: every `href` and `src` it emits names a sibling file, and none reaches a network origin or an absolute path. `file://` and the dev server must therefore render identically, with the sole exception of the outbound repository link, which is a navigation target rather than a resource dependency.
]

== Chrome Labels

#contract[
  Toolbar chrome exists in a Chinese set and an English set, selected by one pure function of the content language: after normalization a tag beginning `zh` takes Chinese, and everything else — including the empty string and every untranslated language such as Portuguese or Japanese — takes English. The document body is never inspected, so glyphs cannot choose the interface language.
]

#logic-box[
  `ui_text(content_lang: &str) -> UiText` returns a struct of `&'static str` fields covering sidebar toggle, theme, system preference, search (label, action, placeholder), print, table of contents, GitHub, language switcher, and the three compile-failure strings. The same resolved value also becomes the page's `lang` attribute, which is why one tag cannot disagree with the labels beside it.
]

#contract[
  An untranslated language wears English chrome while its root `lang` still names it truthfully: the two are independent axes by design, and a missing translation must never relabel a document as some other language.
]

#contract[
  The `Table of contents` label exists twice on purpose, once in this module for the sidebar heading and once in the fy-spec template for the printed cover, where the Chinese value carries a typesetting space (`目 录`). The two are different surfaces with different typography and are not required to be equal.
]

== Language Switcher and Landing Page

#contract[
  The language menu is rendered only when the project declares at least two named-language targets; a single-language project ships no switcher at all. The current target's entry is the checked one, and the default target (a root `docs/main.typ`) marks the first named language active, because that is the page the switcher's own order presents.
]

#contract[
  A `GitHub` icon link is emitted only when the manifest declares a GitHub repository URL; every other host, and the absence of one, produce no markup at that slot.
]

#contract[
  The routing landing page is a client-side redirect carrying nothing but its script, its language map, and a plain link list. It resolves in this order: a language explicitly stored by the reader, then each entry of the browser's language list matched first as a full tag and then as a base subtag, then the default target. The default prefers English, then Simplified Chinese, then the first named target. A `<noscript>` refresh mirrors the same default, so the page routes without JavaScript.
]

#contract[
  The landing page's language map is keyed by the lowercased full tag and by its base subtag, both pointing at that target's HTML file name, so `zh-cn` and `zh` reach a `zh-CN` target. The landing page is generated only for a project that has at least one named-language target.
]

== Escaping Ladder

#logic-box[
  Three escapers, one per context, and the script-context escaper is defined in terms of the others:
  - text — `&`, `<`, `>`; for element content.
  - attribute — text escaping plus `"`; for quoted attribute values.
  - script string — attribute escaping is *not* enough, so this escapes HTML metacharacters first and only then the JavaScript literal characters: backslash, the surrounding quote, `\n`, `\r`, `\t`, `U+2028`, `U+2029`, and every character below `U+0020`. Two quote variants exist, for double-quoted JSON and for a single-quoted literal.
]

#contract[
  HTML metacharacters are escaped before JavaScript literal characters, and that order is load-bearing: escaping `</script>` to `&lt;/script&gt;` first is what keeps a value from terminating the script element, and doing the literal escapes afterwards keeps the result a valid literal.
]

#contract[
  The bytes of the bundled stylesheet, viewer script, and live-reload client are owned here, embedded at compile time; the file names they land under are owned by the `compiler` module. Neither list may be duplicated at the other's site.
]

#contract[
  Error output is rendered through the same shell as a successful page, so a failed build still yields a styled, localized, navigable page rather than raw text: the diagnostic is escaped as element content and placed in the body slot, and the shell's chrome, language switcher, and asset links come from the same target as always.
]
