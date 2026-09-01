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
  Every language target carries a *content language* tag that feeds `<html lang>` and the interface labels. It resolves in order: the language directory name, then the entry `main.typ`'s `lang:` argument, then `en` (the same default the fy-spec template declares). A language directory always wins over a differing `lang:` inside its entry.
]

#contract[
  The content language comes only from declarations and is never inferred from the glyphs in the body. `version:` and `lang:` are read by one shared template-argument parser: line by line, everything after `//` discarded, taking the first quoted value after the argument name, with an identifier boundary required before the name, so `sub-lang:` never matches.
]

#contract[
  `LanguageTarget` fields: `lang` (the language directory name; the empty string for the default target registered by a root `docs/main.typ`), `content_lang` (that target's content language as a normalized BCP 47 tag), `display_name` (the switcher label), `entry`, `html_file_name`, and `pdf_file_name`. `detect_language_targets` writes `content_lang` once while building each target — same vintage as `version`, which is also read from the entry source — so it does not move while the process lives.
]

#contract[
  A compile whose typst HTML export carries a root `lang` naming a different language warns on stderr (see the `compiler` module); the content language resolved here is what the page and its PDF sibling use and is never replaced by the export value.
]

#logic-box[
  Scanner signature: `parse_template_argument(text: &str, key: &str) -> Option<String>`, wrapped per key as `main_typ_version(entry: &Path)` and `main_typ_lang(entry: &Path)`. Precondition: `text` is the complete entry source. Postcondition: the value is the text between quotes of the *first* `key:` that sits in uncommented code, has an identifier boundary before the name, and carries a quoted value on that same line; an occurrence failing any of those keeps the scan going, and if none qualify the result is `None` — never a guess.
]

#logic-box[
  Resolver signature: `resolve_content_lang(dir_name: &str, entry: &Path) -> String`. `dir_name` is the language directory name, empty for the default target. Postcondition: the result is always a non-empty BCP 47 tag in display shape; a non-empty `dir_name` decides on its own, otherwise the value is the entry's `lang:` argument, and when neither is declared the result is `en`. It is total and side-effect free: an unreadable entry yields the default instead of an error.
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

#contract[
  `Project` carries the watch set: the `docs/` directory first, then the top-level directory named by each absolute import that resolves to a directory inside the root, deduplicated in discovery order. The derivation lives here because only detection knows which local packages a build pulls in; the watcher of the `server` chapter recursively watches exactly this list and adds nothing of its own.
]

#contract[
  `ensure_gitignore(root, entries)` is the project module's single shared ignore-rule helper, called by `scaffold` with both generated directories and by `compiler` with the HTML one. It reads the file as text, treats an entry as present when some line equals it after trimming, appends only the missing entries after restoring a missing final newline, and rewrites the file only when something was added. An existing line is never reordered, rewritten or removed, and a differently spelled equivalent pattern therefore becomes an additional line rather than a match. An unwritable `.gitignore` is logged and ignored: no command's exit code depends on it.
]

== Module Structure

The module splits by concern into five files under `src/project/`:

#figure(
  table(
    columns: (auto, auto),
    inset: 6pt,
    align: (auto, left),
    table.header([*File*], [*Concern*]),
    [`mod.rs`], [The `Project` type, `detect`, the language-target scan, `--lang` selection, and the shared path helpers (`clean_canonicalize`, `ensure_gitignore`)],
    [`lang.rs`], [`LanguageTarget` and the language toolkit: `normalize_lang`, `format_lang`, `lang_display_name`, `resolve_content_lang`. Every target is built through one constructor, so the output file naming rules live in exactly one place],
    [`cargo_meta.rs`], [`Cargo.toml` reading, including `workspace = true` inheritance],
    [`imports.rs`], [The absolute-import scan and typst root detection],
    [`template_args.rs`], [The shared template-argument parser and its `version:` / `lang:` readers],
  ),
  caption: [The project module's internal layout.],
)

#contract[
  A target's version resolves in one place, in order: the manifest version (already honoring workspace inheritance), else the entry's `version:` argument, else `0.1.0`. The same resolved version is what the target's release PDF file name embeds.
]
