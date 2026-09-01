#import "../../fy-spec/lib.typ": *

= viewer Module: Single-Page Reader & Language Switching <sec-viewer>

The `viewer` module powers the responsive reader interface and multi-language controls.

#contract[
  For multilingual documentation, the top toolbar renders a `🌐 Language Switcher` dropdown to jump between `index_<lang>.html` files while preserving the active section hash anchor. For single-language projects, the switcher is automatically omitted.
]

#contract[
  The root element's `lang` describes the *content* language: it carries the content tag the `project` module resolved for that target, in normalized BCP 47 shape (`pt_BR` becomes `pt-BR`, base subtag lowercase, region uppercase, script title-cased). That tag comes from declarations only and is never inferred from the glyphs in the body.
]

#contract[
  Toolbar chrome exists in English and Chinese only: both the server-rendered template and `viewer.js` pick one set from the root `lang`'s `zh` prefix, so content language and chrome language are independent axes — an untranslated language wears English chrome while its root tag still names it truthfully.
]

#logic-box[
  Label selection is a pure function, `ui_text(content_lang: &str) -> UiText`: the content language is its only input, never the body. After `normalize_lang`, a tag starting with `zh` selects the Chinese set and everything else (including the empty string) selects English. The page's `lang` attribute is `LanguageTarget.content_lang` — the same value handed to `ui_text`.
]

#contract[
  Provides 5 refined dark/light themes (Light, Rust, Coal, Navy, Ayu) and system synchronization. Both toolbar title and sidebar brand navigate back to the book cover upon click.
]

#contract[
  Code blocks support one-click copy to clipboard on hover, and the search panel renders matching text snippets with keyword highlighting.
]
