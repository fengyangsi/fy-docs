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
cargo fy-docs vendor # Sync the embedded fy-spec template into docs/fy-spec/
```

This chapter specifies argument parsing, dispatch order, port allocation, and the exit-code path. The two commands that write into a project's source tree — `init` and `vendor` — carry their design in the `scaffold` module, and the pre-check that gates every compiling command is specified here because it is what the entry point refuses to run without.

#contract[
  `cargo fy-docs` is the primary entry point, and invoking the executable directly is equally supported. Cargo runs an external subcommand as `cargo-fy-docs fy-docs …`, so the first argument after the program name is dropped exactly when it is `fy-docs`; any other first argument is kept, which is what makes a direct call with a real subcommand parse correctly.
]

#contract[
  Every option is global, so it may appear before or after the subcommand: `--root`, `--lang`, `--open`, `--with-pdf`, `--port`, and `--no-open`. With no subcommand given, the default command is the full build.
]

#contract[
  Every compiling command pre-checks the `typst` CLI before running: a missing binary or a version older than 0.14 (the first release accepting `--pdf-standard 2.0`) aborts immediately with an actionable message. A version banner fy-docs cannot parse does not block a working install — the compile step surfaces real errors anyway.
]

#contract[
  Startup options are captured once into the shared state — whether PDFs accompany generation, and the `--lang` filter — and every generation reads the captured options. The default command and `build` capture PDFs as always on, so `--with-pdf` has no separate effect there; `html` and `dev` capture the flag as given. A filtered dev session stays filtered across rebuilds.
]

#contract[
  Dispatch order is fixed: `init` and `vendor` return before the typst precheck and before project detection, and every remaining command runs the precheck first and detection second. A missing `typst` or a missing `docs/` therefore never costs a compile attempt.
]

#contract[
  `--port` names the first candidate, defaulting to `8181`. A bound port is retried on consecutive candidates up to twenty attempts, the first success wins, and a range with no free port fails with a message naming that range. The address bound is the loopback interface only.
]

#contract[
  An error that reaches the entry point is reported as one `[fy-docs] <error>` line on stderr and the process exits non-zero. The dispatch function returns an exit code rather than terminating the process itself, so a failing command is a value a test can assert on instead of a killed test binary.
]

#contract[
  `--open` opens the artifact the command just produced — the root `index.html` for a build, the first generated PDF for `pdf` — detached from the process. `dev` opens the served URL unless `--no-open` is given, and a browser it cannot launch is reported with the URL to open by hand rather than treated as a failure.
]

#invariant[
  `cargo fy-docs` default command, `build`, `html`, and `pdf` commands are idempotent, non-blocking builds that never hang CI pipelines. They exit with code `0` only when every compilation succeeds; a compile failure exits non-zero so pipelines catch broken documents. The `dev` server instead survives failures and renders them as error pages.
]

== Module Structure

`src/main.rs` is the whole command-line surface; two support files fold into this chapter. `src/term.rs` is the terminal output module — the `[fy-docs]` line prefix, progress logging whose write errors are ignored so a closed pipe never panics the watcher thread, and Windows verbatim path prefixes stripped from every user-facing path and forwarded diagnostic. `src/lib.rs` carries the crate-level documentation and no items.

#logic-box[
  `term::log(message: &str)` writes to stderr only, and ignores the result: every progress line is diagnostic, and no command's success may depend on a readable output pipe. Structured data a machine consumes — the generated files themselves — never travels through it.
]
