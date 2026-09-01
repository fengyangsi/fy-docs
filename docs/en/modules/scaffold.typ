#import "../../fy-spec/lib.typ": *

= scaffold Module: Project Scaffolding & Template Vendoring <sec-scaffold>

The `scaffold` module owns the only two operations that write into a project's source tree rather than into generated output: creating a `docs/` directory from nothing, and refreshing the project's private copy of the fy-spec template. The command surface — `init`, `vendor`, `vendor --check` — is declared by the `cli` module; this chapter specifies how those commands build and verify their artifacts.

#contract[
  One file is the source of every template copy: the embedded library is the repository's own `docs/fy-spec/lib.typ`, read at compile time. The binary, this repository's dogfood build, and every project's vendored copy therefore carry identical bytes, and `vendor --check` compares them for exact equality — no comment, whitespace, or line-ending normalization is applied, so a reformatted copy counts as drift.
]

#contract[
  `init` aborts when `docs/main.typ` is already a file, naming the path and instructing the user to remove it first. It examines no other pre-existing state, and it never modifies or deletes a file: a refusal leaves the tree exactly as it was.
]

#contract[
  A successful `init` creates exactly three artifacts — `docs/main.typ`, `docs/fy-spec/lib.typ`, and the directory `docs/modules/` — and creates them with the recursive directory operation, so a missing `docs/` is the normal case rather than an error. `docs/modules/` is left empty: it is where a project puts its own chapters, not something the tool can write.
]

#contract[
  The starter `docs/main.typ` is the shipped template text with only `{{NAME}}`, `{{VERSION}}`, and `{{AUTHOR}}` substituted; every other byte survives, including the relative `#import "fy-spec/lib.typ"`, the `lang: "en"` declaration, and the commented-out `#include`. `init` is the only command that runs without an existing `docs/` directory.
]

#contract[
  Neither `init` nor `vendor` requires the `typst` binary or a detectable project: both return from dispatch before the typst precheck and before project detection, so they succeed in a directory where no document can yet be compiled.
]

#contract[
  `vendor` requires an existing `docs/` directory and fails with a hint to run `init` when it is absent. It then (re)writes `docs/fy-spec/lib.typ` unconditionally, creating `docs/fy-spec/` if needed — writing is idempotent and never refuses on drift grounds, because overwriting with the embedded template *is* the repair.
]

#contract[
  `vendor --check` writes nothing. A missing file, and a file whose bytes differ from the embedded template, each exit non-zero naming the file and the command that repairs it; a byte-identical file logs that it matches and exits `0`, which is what pins the template version in a project's CI.
]

#contract[
  `init` appends `/docs/target/` and `/docs/release/` to the project's `.gitignore` through the shared helper the build commands use, adding an entry only when it is absent and treating a write failure as a logged warning rather than an error.
]

#logic-box[
  `init(cwd: &Path) -> Result<()>` and `vendor(cwd: &Path, check: bool) -> Result<()>` are total in their inputs and take no options struct: `cwd` is the canonicalized working directory dispatch already resolved. The starter's three fields resolve in this order, and the order is deliberate — `Cargo.toml` first because it is the version's only true source:
  - *name*: Cargo package name, else the directory name of `cwd`, else `project`.
  - *version*: Cargo package version, else `0.1.0`.
  - *author*: the first `authors` entry, else `TODO`.
]

#contract[
  `init`'s version ladder has no middle rung: unlike `project` detection, it never reads a `version:` argument out of a `main.typ`, because at `init` time no such file exists yet. The two ladders differ on purpose and are not to be unified.
]

#contract[
  An author resolved from a manifest that declares none becomes the literal `TODO`, never any name taken from fy-docs itself: the starter is the user's document, and a scaffolder must not sign it.
]

#invariant[
  `cargo fy-docs init` followed by `cargo fy-docs` compiles without a single user edit: the scaffolded tree is a working project, not a fixture that needs hand-finishing. An integration test runs exactly this pair against the real binary.
]
