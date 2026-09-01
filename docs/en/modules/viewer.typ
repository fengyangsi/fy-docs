#import "../../fy-spec/lib.typ": *

= viewer Module: Single-Page Reader & Language Switching <sec-viewer>

The `viewer` module powers the responsive reader interface and multi-language controls.

#contract[
  For multilingual documentation, the top toolbar renders a `🌐 Language Switcher` dropdown to jump between `index_<lang>.html` files while preserving the active section hash anchor. For single-language projects, the switcher is automatically omitted.
]

#contract[
  The root element's `lang` describes the *content* language: a named language target emits its normalized BCP 47 tag (`pt_BR` becomes `pt-BR`, base subtag lowercase, region uppercase, script title-cased), while a tagless default target is inferred from the body (`zh-CN` when it contains CJK ideographs, otherwise `en`). Toolbar chrome exists in English and Chinese only and `viewer.js` selects between them from the `lang` prefix, so content language and chrome language are independent axes: an untranslated language wears English chrome while its root tag still names it truthfully.
]

#contract[
  Provides 5 refined dark/light themes (Light, Rust, Coal, Navy, Ayu) and system synchronization. Both toolbar title and sidebar brand navigate back to the book cover upon click.
]

#contract[
  Code blocks support one-click copy to clipboard on hover, and the search panel renders matching text snippets with keyword highlighting.
]
