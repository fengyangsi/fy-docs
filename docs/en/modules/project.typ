#import "../../fy-spec/lib.typ": *

= project Module: Metadata & i18n Target Detection <sec-project>

The `project` module scans the directory to resolve package metadata, multilingual document targets, and Typst root boundaries.

```text
docs/
├── fy-spec/lib.typ      # Shared specification template
├── zh-CN/               # Simplified Chinese target
├── en/                  # English target
└── target/              # Generated offline HTML bundle
```

#contract[
  `Project` automatically detects single-language (`docs/main.typ`) and multilingual (`docs/<lang>/main.typ`) directories. Each language target compiles to its own `index_<lang>.html` and versioned PDF.
]

#contract[
  A `--lang <LANG>` filter value is normalized before matching: case-insensitive, with `_` and `-` interchangeable, so `zh_CN`, `ZH-cn`, and `zh-cn` all select the same language target. After normalization the *full* language tag must match exactly with no prefix fallback: `--lang zh` does not select a `zh-CN` target.
]

#contract[
  A root `docs/main.typ` registers the always-included `default` target: any `--lang <LANG>` filter selects it alongside the requested language, so it compiles even when filtered. Projects wanting per-language isolation must keep language directories only (no root `main.typ`).
]

#contract[
  When no language target matches after normalization, the build fails with a non-zero exit code and lists the languages the project actually provides.
]

#contract[
  Language detection uses a positive rule: any `docs/` subdirectory carrying its own `main.typ` is a language target, except the generated directories (`target/`, `release/`).
]

#contract[
  When the manifest declares no version, the fallback reads the entry `main.typ`'s `version:` argument from uncommented code only; a version inside a comment never becomes the project version.
]

#contract[
  The absolute-import scan behind root detection respects lexical boundaries: both quote styles are recognized and everything after `//` is a comment, so a commented-out `#import` cannot drag the root to the wrong ancestor.
]

#contract[
  Language folders follow BCP 47 layering: the base language subtag decides the translation, and a region or script subtag is appended *only when it carries a real difference* (`zh-CN`/`zh-TW`, `pt-BR`/`pt-PT`). `en` stays the neutral English root with no region, and a document must never be forked over spelling alone. Adding a language means registering it in both the `lang_display_name` table and the template's `i18n-strings` base keys.
]

#contract[
  An unregistered language tag is displayed in normalized BCP 47 shape: base subtag lowercase, region subtag uppercase, script subtag title-cased (`pt_BR` becomes `pt-BR`, `zh_hant_tw` becomes `zh-Hant-TW`). fy-docs never invents a language name it cannot know.
]

#invariant[
  Typst root sandbox (`root`) automatically defaults to the nearest ancestor satisfying all absolute imports, overridable via `--root <DIR>`.
]
