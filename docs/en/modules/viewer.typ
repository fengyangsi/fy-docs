#import "../../fy-spec/lib.typ": *

= viewer Module: Single-Page Reader & Language Switching <sec-viewer>

The `viewer` module powers the responsive reader interface and multi-language controls.

#contract[
  For multilingual documentation, the top toolbar renders a `🌐 Language Switcher` dropdown to jump between `index_<lang>.html` files. For single-language projects, the switcher is automatically omitted.
]

#contract[
  Provides 5 refined dark/light themes (Light, Rust, Coal, Navy, Ayu) and system synchronization. Both toolbar title and sidebar brand navigate back to the book cover upon click.
]
