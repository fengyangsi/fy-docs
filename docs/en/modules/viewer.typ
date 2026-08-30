#import "../../fy-spec/lib.typ": *

= viewer Module: Single-Page Reader & Language Switching <sec-viewer>

The `viewer` module powers the responsive reader interface and multi-language controls.

#contract[
  For multilingual documentation, the top toolbar renders a `🌐 Language Switcher` dropdown to jump between `index_<lang>.html` files while preserving the active section hash anchor. For single-language projects, the switcher is automatically omitted.
]

#contract[
  Provides 5 refined dark/light themes (Light, Rust, Coal, Navy, Ayu) and system synchronization. Both toolbar title and sidebar brand navigate back to the book cover upon click.
]

#contract[
  Code blocks support one-click copy to clipboard on hover, and the search panel renders matching text snippets with keyword highlighting.
]
