#import "../../fy-spec/lib.typ": *

= cli Module: Cargo Subcommand Interface <sec-cli>

`fy-docs` is distributed as an external Cargo subcommand (`cargo-fy-docs`). Inside any project with a `docs/` directory:

```bash
cargo fy-docs        # Default: Full build of HTML & PDF 2.0
cargo fy-docs build  # Build all language documents
cargo fy-docs html   # Compile offline HTML bundle only
cargo fy-docs pdf    # Compile PDF 2.0 specifications only
cargo fy-docs dev    # Start dev server with live reload & browser preview
cargo fy-docs init   # Scaffold a self-contained docs/ directory
```

#contract[
  `cargo fy-docs` is the primary entry point. Cargo forwards any trailing arguments directly to the binary.
]

#contract[
  `init` is the only command that does not require an existing `docs/` folder: it creates `docs/`, starter `main.typ`, embedded `fy-spec/lib.typ`, and appends `/docs/target/` and `/docs/release/` to `.gitignore`.
]

#contract[
  Every compiling command pre-checks the `typst` CLI before running: a missing binary or a version older than 0.14 (the first release accepting `--pdf-standard 2.0`) aborts immediately with an actionable message.
]

#invariant[
  `cargo fy-docs` default command, `build`, `html`, and `pdf` commands are idempotent, non-blocking builds that never hang CI pipelines. They exit with code `0` only when every compilation succeeds; a compile failure exits non-zero so pipelines catch broken documents. The `dev` server instead survives failures and renders them as error pages.
]
