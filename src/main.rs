mod assets;
mod compiler;
mod project;
mod scaffold;
mod server;
mod state;
mod watcher;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
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

    /// Also compile a print-edition PDF into docs/release/ (for build/html subcommands)
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
async fn main() -> Result<()> {
    let cli = Cli::parse_from(cargo_external_args(std::env::args_os()));
    let cwd = crate::project::clean_canonicalize(&std::env::current_dir()?)?;
    dispatch(cli, &cwd).await
}

async fn dispatch(cli: Cli, cwd: &Path) -> Result<()> {
    // Init does not require an existing docs/ directory.
    if matches!(cli.command, Some(Command::Init)) {
        return scaffold::init(cwd);
    }

    let project = project::detect(cwd, cli.root.as_deref())?;
    let state = Arc::new(AppState::new(project));

    match cli.command {
        Some(Command::Init) => unreachable!(),
        None | Some(Command::Build) => {
            // Default: Full build of both HTML and PDF
            compiler::generate_into(&state, true, cli.lang.as_deref());
            state.write_build();
            state::log(&format!(
                "[fy-docs] generated {}",
                state::display_path(&state.project.target_dir)
            ));
            if cli.open {
                let index_path = state.project.target_dir.join(compiler::INDEX_FILE);
                let _ = open::that_detached(index_path);
            }
        }
        Some(Command::Html) => {
            // HTML only (or with PDF if explicitly asked)
            compiler::generate_into(&state, cli.with_pdf, cli.lang.as_deref());
            state.write_build();
            state::log(&format!(
                "[fy-docs] HTML generated in {}",
                state::display_path(&state.project.target_dir)
            ));
            if cli.open {
                let index_path = state.project.target_dir.join(compiler::INDEX_FILE);
                let _ = open::that_detached(index_path);
            }
        }
        Some(Command::Pdf) => {
            // PDF 2.0 specifications only
            let paths = compiler::compile_pdf(&state.project, cli.lang.as_deref())?;
            for path in paths {
                state::log(&format!(
                    "[fy-docs] PDF written to {}",
                    state::display_path(&path)
                ));
            }
        }
        Some(Command::Dev) | Some(Command::Serve) => {
            // Dev mode: live server + watcher
            compiler::generate_into(&state, cli.with_pdf, cli.lang.as_deref());
            state.write_build();
            watcher::spawn(state.clone())?;

            let app = server::router(&state);
            let listener = bind(cli.port).await?;
            let url = format!("http://{}", listener.local_addr()?);
            state::log(&format!("[fy-docs] serving at {url}"));
            if !cli.no_open {
                if let Err(err) = open::that_detached(&url) {
                    state::log(&format!(
                        "[fy-docs] could not open a browser ({err}); open {url} manually"
                    ));
                }
            }
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                    state::log("[fy-docs] shutting down");
                })
                .await?;
        }
    }
    Ok(())
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
        let temp = std::env::temp_dir().join(format!("fy-docs-main-run-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).unwrap();
        let clean_temp = crate::project::clean_canonicalize(&temp).unwrap();

        // 1. init
        let init_cli = Cli::parse_from(["cargo-fy-docs", "init"]);
        dispatch(init_cli, &clean_temp).await.unwrap();
        assert!(clean_temp.join("docs/main.typ").is_file());
        assert!(clean_temp.join("docs/fy-spec/lib.typ").is_file());

        // 2. build (HTML + PDF)
        let build_cli = Cli::parse_from(["cargo-fy-docs", "build"]);
        dispatch(build_cli, &clean_temp).await.unwrap();
        assert!(clean_temp.join("docs/target/index.html").is_file());
        assert!(clean_temp.join("docs/target/fy-docs.css").is_file());
        assert!(clean_temp.join("docs/release").is_dir());

        // 3. html only
        let html_cli = Cli::parse_from(["cargo-fy-docs", "html"]);
        dispatch(html_cli, &clean_temp).await.unwrap();
        assert!(clean_temp.join("docs/target/index.html").is_file());

        // 4. pdf only
        let pdf_cli = Cli::parse_from(["cargo-fy-docs", "pdf"]);
        dispatch(pdf_cli, &clean_temp).await.unwrap();

        let _ = std::fs::remove_dir_all(&temp);
    }
}
