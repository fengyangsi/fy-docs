mod assets;
mod compiler;
mod project;
mod scaffold;
mod server;
mod state;
mod term;
mod watcher;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use crate::state::AppState;

#[derive(Parser)]
#[command(
    name = "cargo-fy-docs",
    bin_name = "cargo fy-docs",
    version,
    about = "Build, preview, and generate Typst specification documents (HTML & PDF 2.0) with i18n support"
)]
struct Cli {
    /// Typst compile root; auto-detected from imports when omitted
    #[arg(long, global = true)]
    root: Option<PathBuf>,

    /// Target specific language (e.g., "zh-CN", "en", "zh-TW")
    #[arg(long, global = true)]
    lang: Option<String>,

    /// Open in default web browser after build
    #[arg(long, global = true)]
    open: bool,

    /// Also compile a print-edition PDF into docs/release/ (html and dev
    /// subcommands; build always compiles PDFs)
    #[arg(long, global = true)]
    with_pdf: bool,

    /// Server port for dev mode (increments on collision)
    #[arg(long, global = true, default_value_t = 8181)]
    port: u16,

    /// Do not open the browser in dev mode
    #[arg(long, global = true)]
    no_open: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Scaffold a new docs/ directory with a starter template
    Init,
    /// Sync the embedded fy-spec template into docs/fy-spec/lib.typ
    Vendor {
        /// Verify the vendored template without writing; non-zero on drift
        #[arg(long)]
        check: bool,
    },
    /// Build everything (HTML pages and PDF 2.0 specifications) - default command
    Build,
    /// Build only the offline HTML documentation
    Html,
    /// Compile only the print-edition PDF 2.0 specification(s) into docs/release/
    Pdf,
    /// Start interactive development server with live reload and browser preview
    Dev,
    /// Deprecated alias for dev
    #[command(hide = true)]
    Serve,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse_from(cargo_external_args(std::env::args_os()));
    let dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            crate::term::log(&format!(
                "[fy-docs] could not read the current directory: {err}"
            ));
            return ExitCode::FAILURE;
        }
    };
    let cwd = match crate::project::clean_canonicalize(&dir) {
        Ok(cwd) => cwd,
        Err(err) => {
            crate::term::log(&format!("[fy-docs] {err:#}"));
            return ExitCode::FAILURE;
        }
    };
    match dispatch(cli, &cwd).await {
        Ok(code) => code,
        Err(err) => {
            crate::term::log(&format!("[fy-docs] {err:#}"));
            ExitCode::FAILURE
        }
    }
}

async fn dispatch(cli: Cli, cwd: &Path) -> Result<ExitCode> {
    // Init does not require an existing docs/ directory.
    if matches!(cli.command, Some(Command::Init)) {
        return scaffold::init(cwd).map(|()| ExitCode::SUCCESS);
    }

    // Vendor only copies the embedded template; it needs no typst binary.
    if let Some(Command::Vendor { check }) = cli.command {
        return scaffold::vendor(cwd, check).map(|()| ExitCode::SUCCESS);
    }

    // Every remaining command compiles, so fail fast on a missing or old typst.
    compiler::precheck()?;

    let project = project::detect(cwd, cli.root.as_deref())?;
    // Capture the CLI options once so every generation — including each
    // dev-mode rebuild — repeats them. The default command and `build` always
    // compile PDFs; `html` and `dev` follow the flag.
    let with_pdf = match cli.command {
        None | Some(Command::Build) => true,
        _ => cli.with_pdf,
    };
    let state = Arc::new(AppState::with_generate(
        project,
        state::GenerateOptions {
            with_pdf,
            lang_filter: cli.lang.clone(),
        },
    ));

    match cli.command {
        // Init and Vendor returned before project detection and precheck.
        Some(Command::Init | Command::Vendor { .. }) => unreachable!(),
        None | Some(Command::Build | Command::Html) => {
            if compiler::generate(&state).is_err() {
                return Ok(ExitCode::FAILURE);
            }
            crate::term::log(&format!(
                "[fy-docs] generated {}",
                crate::term::display_path(&state.project.target_dir)
            ));
            if cli.open {
                let index_path = state.project.target_dir.join(compiler::INDEX_FILE);
                let _ = open::that_detached(index_path);
            }
        }
        Some(Command::Pdf) => {
            // PDF 2.0 specifications only
            let paths = compiler::compile_pdf(&state.project, cli.lang.as_deref())?;
            for path in &paths {
                crate::term::log(&format!(
                    "[fy-docs] PDF written to {}",
                    crate::term::display_path(path)
                ));
            }
            if cli.open
                && let Some(first) = paths.first()
            {
                let _ = open::that_detached(first);
            }
        }
        Some(Command::Dev) | Some(Command::Serve) => {
            // Dev mode: live server + watcher. A failing first build still
            // starts the server, whose error pages the browser then shows.
            let _ = compiler::generate(&state);
            watcher::spawn(state.clone())?;

            let app = server::router(&state);
            let listener = bind(cli.port).await?;
            let url = format!("http://{}", listener.local_addr()?);
            crate::term::log(&format!("[fy-docs] serving at {url}"));
            if !cli.no_open {
                if let Err(err) = open::that_detached(&url) {
                    crate::term::log(&format!(
                        "[fy-docs] could not open a browser ({err}); open {url} manually"
                    ));
                }
            }
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                    crate::term::log("[fy-docs] shutting down");
                })
                .await?;
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// Cargo invokes external commands as `cargo-<name> <name> ...`; remove the
/// forwarded subcommand name so Clap receives the tool's own arguments.
fn cargo_external_args<I>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args.into_iter();
    let mut normalized = vec![
        args.next()
            .unwrap_or_else(|| OsString::from("cargo-fy-docs")),
    ];
    if let Some(first) = args.next()
        && first != OsStr::new("fy-docs")
    {
        normalized.push(first);
    }
    normalized.extend(args);
    normalized
}

async fn bind(port: u16) -> Result<tokio::net::TcpListener> {
    let last = port.saturating_add(19);
    for candidate in port..=last {
        match tokio::net::TcpListener::bind(("127.0.0.1", candidate)).await {
            Ok(listener) => return Ok(listener),
            Err(err) if err.kind() == std::io::ErrorKind::AddrInUse => continue,
            Err(err) => return Err(err.into()),
        }
    }
    anyhow::bail!("no free port in {port}..={last}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_cargo_forwarded_subcommand_name() {
        let args = ["cargo-fy-docs", "fy-docs", "dev"]
            .into_iter()
            .map(OsString::from);
        let normalized = cargo_external_args(args);
        assert_eq!(
            normalized,
            vec![OsString::from("cargo-fy-docs"), OsString::from("dev")]
        );
    }

    #[test]
    fn keeps_direct_executable_arguments() {
        let args = ["cargo-fy-docs", "html", "--root", "."]
            .into_iter()
            .map(OsString::from);
        let normalized = cargo_external_args(args);
        assert_eq!(
            normalized,
            vec![
                OsString::from("cargo-fy-docs"),
                OsString::from("html"),
                OsString::from("--root"),
                OsString::from(".")
            ]
        );
    }

    #[test]
    fn cli_parses_all_subcommands_and_flags() {
        let args = ["cargo-fy-docs", "build", "--lang", "zh-CN", "--open"]
            .into_iter()
            .map(OsString::from);
        let cli = Cli::parse_from(cargo_external_args(args));
        assert!(matches!(cli.command, Some(Command::Build)));
        assert_eq!(cli.lang.as_deref(), Some("zh-CN"));
        assert!(cli.open);
    }

    #[tokio::test]
    async fn bind_allocates_available_listener() {
        let listener = bind(8181).await.expect("bind should find a free port");
        assert!(listener.local_addr().is_ok());
    }

    #[tokio::test]
    async fn dispatch_executes_build_pdf_html_init_commands() {
        // The build arms exit the process on compile failure, so this test
        // needs a working typst; skip where none is installed.
        if !std::process::Command::new("typst")
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
        {
            eprintln!("skipping: typst is not on PATH");
            return;
        }
        let temp = tempfile::tempdir().unwrap();
        let clean_temp = crate::project::clean_canonicalize(temp.path()).unwrap();

        // 1. init
        let init_cli = Cli::parse_from(["cargo-fy-docs", "init"]);
        let code = dispatch(init_cli, &clean_temp).await.unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
        assert!(clean_temp.join("docs/main.typ").is_file());
        assert!(clean_temp.join("docs/fy-spec/lib.typ").is_file());

        // 2. build (HTML + PDF)
        let build_cli = Cli::parse_from(["cargo-fy-docs", "build"]);
        let code = dispatch(build_cli, &clean_temp).await.unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
        assert!(clean_temp.join("docs/target/index.html").is_file());
        assert!(clean_temp.join("docs/target/fy-docs.css").is_file());
        assert!(clean_temp.join("docs/release").is_dir());

        // 3. html only
        let html_cli = Cli::parse_from(["cargo-fy-docs", "html"]);
        let code = dispatch(html_cli, &clean_temp).await.unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
        assert!(clean_temp.join("docs/target/index.html").is_file());

        // 4. pdf only
        let pdf_cli = Cli::parse_from(["cargo-fy-docs", "pdf"]);
        let code = dispatch(pdf_cli, &clean_temp).await.unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }
}
