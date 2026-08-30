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
  When no language target matches after normalization, the build fails with a non-zero exit code and lists the languages the project actually provides. A misspelled `--lang` never silently degrades to "build the default target only".
]

#contract[
  Language detection uses a positive rule: any `docs/` subdirectory carrying its own `main.typ` is a language target, except the generated directories (`target/`, `release/`). Shared source folders such as `fy-spec/` and `modules/` need no denylist because they hold no `main.typ`.
]

#contract[
  When the manifest declares no version, the fallback reads the entry `main.typ`'s `version:` argument from uncommented code only: a version inside prose or a disabled example must never become the project version.
]

#contract[
  The absolute-import scan behind root detection respects lexical boundaries: both quote styles are recognized and everything after `//` is a comment, so a commented-out `#import` cannot drag the root to the wrong ancestor.
]

#invariant[
  Typst root sandbox (`root`) automatically defaults to the nearest ancestor satisfying all absolute imports, overridable via `--root <DIR>`.
]
