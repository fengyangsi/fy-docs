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

#invariant[
  `cargo fy-docs` default command, `build`, `html`, and `pdf` commands are idempotent, non-blocking builds that exit with code `0` upon completion, never hanging CI pipelines.
]
