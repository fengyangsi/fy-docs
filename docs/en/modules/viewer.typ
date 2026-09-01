#import "../../fy-spec/lib.typ": *

= viewer Module: Browser Reader, Theming & Live Reload <sec-viewer>

The `viewer` chapter owns three files, `assets/viewer.js`, `assets/base.css` and `assets/live.js`, and it is the only chapter whose code never runs inside the fy-docs process. The `page` chapter embeds these files as string constants, the `compiler` writes them beside every generated page, and a browser then loads them from `docs/target/`. What is specified here is therefore runtime behavior, not a Rust API.

The three files are shipped byte-for-byte: no bundler and no transpiler stand between the repository and the browser, so the source a reviewer reads is the artifact a user runs. The shell these files bind is `assets/doc.html`, specified by the `page` chapter. That shell's `id` and `class` surface is an internal invariant, so `viewer.js` is entitled to query any of it without defending against an absent element.

#contract[
  `viewer.js` regroups the document at load time: every top-level `<h2>` inside `#doc-body` opens a chapter, the nodes before the first heading form a leading cover chapter, and the `nav[role="doc-toc"]` that typst emits inside the body is moved out into the sidebar. Each chapter becomes a `section.fy-chapter` holding its nodes, and its anchor is the id of its first node — synthesized as `cover` for the leading chapter and `ch-<n>` otherwise when the node carries none.
]

#contract[
  Every `id` inside a chapter is mapped to that chapter, so an in-page anchor — a table-of-contents entry, a cross-reference, a `#` link from another language's page — always opens the chapter that contains its target. Unmapped anchors are left to the browser. The same mapping resolves the initial hash and every later `hashchange`, and the pager that offers previous, position and next is built only for a document with more than one chapter.
]

#contract[
  The sidebar follows the reader, not the last click. Only the visible chapter's headings participate, and only those that carry an `id` and have a table-of-contents entry; an entry pointing outside the chapter can never light up. A passive scroll listener on `.fy-content` — the element that actually scrolls — drives the recomputation, coalesced to one pass per animation frame and repeated on resize, by these three rules in order: the last heading at or above the top edge of the visible area, or the first one while none has reached it, or the last one once the container is scrolled to its bottom, which is the only way a trailing section too short to reach the top edge becomes reachable. Switching chapters rebuilds the participant list from the new chapter's headings.
]

#contract[
  Two granularities light up at once: the active heading's own entry carries `fy-toc-active`, and the entry of the chapter that contains it carries `fy-toc-chapter-active`, so a reader inside `3.2` still sees `3` marked as the containing chapter. When the active heading is the chapter's opening heading, the same entry carries both classes and the position class is the one that shows. An outline that stops at a single level degrades to `fy-toc-active` alone.
]

#contract[
  Left and right arrow keys move between chapters, unless the event is already handled, carries a modifier, or originates in a text field, textarea, select or content-editable node.
]

#contract[
  Two pieces of chrome state are persisted, both through accessors that swallow every storage failure so a denied `localStorage` — private browsing, a `file://` page — degrades to the default instead of throwing: the sidebar's open flag and its width. Open by default at or above a 1024 px viewport, the sidebar width is clamped between 192 px and 512 px and to 55% of the window, and is dragged through the resize handle's pointer capture.
]

#contract[
  Theme is a single class on the root element, one of `light`, `rust`, `coal`, `navy`, `ayu`. The menu's system entry stores `preference` and resolves it at paint time through `prefers-color-scheme` — dark means `navy`, anything else means `light` — and repaints when the system flips while that entry is selected. The menu marks the stored selection with `aria-checked`.
]

#contract[
  `base.css` carries, besides the palettes, per-theme overrides for the literal hex colors typst bakes into exported diagram SVGs, so an embedded DAG follows the active palette instead of staying on the template's light background.
]

#contract[
  Line length is a measure, not a share of the window. Below a 1280 px viewport the document is one centered column capped at `--fy-page-max` — 820 px, whose 32 px side padding leaves a 756 px `--fy-measure`. From 1280 px the page box grows to `--fy-page-wide` (1180 px) and from 1800 px to `--fy-page-ultra` (1440 px), while every top-level block of the document stays pinned to that same 756 px measure; a code block, a figure — the wrapper typst gives every table and diagram — and a bare exported SVG alone are released from it, so a wide screen buys width for the material that would otherwise overflow sideways instead of stretching prose. The cap is written for both shapes of the flow — the blocks typst emits under `#doc-body`, and those same blocks after the chapter grouping refiles them under `section.fy-chapter` — and it never binds a chapter shell itself, because a shell held to the measure would re-clip the very material released from it.
]

#contract[
  Search is a chapter-level index of the rendered text, case-folded with the locale-aware `toLocaleLowerCase`. One result is reported per chapter at its first match, the snippet spans 15 characters before the match and 35 after with whitespace collapsed, and the results container is a polite live region, so a replaced list is announced. Activating a result closes the panel and navigates to that chapter's anchor.
]

#contract[
  Search snippets and chapter titles are HTML-escaped before they enter the panel, and the query is regex-quoted before it is used to wrap matches in `<mark>`.
]

#contract[
  Every `.fy-doc pre` is wrapped in a copy container on load. The button reads the block's rendered text and hands it to the asynchronous clipboard, then shows a transient copied state; where the clipboard API is absent the click does nothing.
]

#contract[
  `#fy-print` invokes the browser's own print, and the toolbar title and the sidebar brand both navigate to the cover chapter.
]

#contract[
  The root element's `lang` describes the *content* language: it carries the content tag the `project` module resolved for that target, in normalized BCP 47 shape (`pt_BR` becomes `pt-BR`, base subtag lowercase, region uppercase, script title-cased). That tag comes from declarations only and is never inferred from the glyphs in the body.
]

#contract[
  Toolbar chrome exists in English and Chinese only, and the choice is made once at generation time by the `page` chapter's `ui_text`; inside the browser, `viewer.js` re-derives the same decision from the root `lang`'s `zh` prefix. Content language and chrome language are independent axes: an untranslated language wears English chrome while its root tag still names it truthfully.
]

#contract[
  Language items preserve the reading position: the chosen target's file is stored under `fydocs-lang`, and when the current URL carries a hash the click is intercepted and rewritten onto the same anchor in the other file.
]

#contract[
  `live.js` opens an `EventSource` on the relative `events` URL, records the first frame as the baseline build, and reloads on any later frame carrying a different value. A stream error closes the connection permanently and silently, which is what makes the same script harmless in a static build opened without a server. The script is loaded without `defer`, so the subscription starts before the reader finishes initializing.
]
