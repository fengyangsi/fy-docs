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

#invariant[
  Typst root sandbox (`root`) automatically defaults to the nearest ancestor satisfying all absolute imports, overridable via `--root <DIR>`.
]
