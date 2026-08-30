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
    about = "Generate and view Typst specification documents in the browser, like cargo doc for your docs/"
)]
struct Cli {
    /// Typst compile root; auto-detected from the absolute imports when omitted
    #[arg(long, global = true)]
    root: Option<PathBuf>,

    /// Also compile a print-edition PDF into docs/release/
    #[arg(long, global = true)]
    with_pdf: bool,

    /// Server port (increments on collision)
    #[arg(long, global = true, default_value_t = 8181)]
    port: u16,

    /// Do not open the browser
    #[arg(long, global = true)]
    no_open: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Scaffold a new docs/ directory with a starter template
    Init,
    /// Generate and serve with live reload (the default command)
    Serve,
    /// Generate only; the page lands in docs/target/
    Build,
    /// Compile only the print-edition PDF into docs/release/
    Pdf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse_from(cargo_external_args(std::env::args_os()));
    let cwd = std::env::current_dir()?.canonicalize()?;
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
        Some(Command::Build) => {
            compiler::generate_into(&state, cli.with_pdf);
            state.write_build();
            state::log(&format!(
                "[fy-docs] generated {}",
                state::display_path(&state.project.target_dir)
            ));
        }
        Some(Command::Pdf) => {
            let path = compiler::compile_pdf(&state.project)?;
            state::log(&format!(
                "[fy-docs] PDF written to {}",
                state::display_path(&path)
            ));
        }
        None | Some(Command::Serve) => {
            compiler::generate_into(&state, cli.with_pdf);
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
            Err(_) => continue,
        }
    }
    anyhow::bail!("no free port in {port}..={last}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_cargo_forwarded_subcommand_name() {
        let args = cargo_external_args([
            OsString::from("cargo-fy-docs"),
            OsString::from("fy-docs"),
            OsString::from("build"),
        ]);
        assert_eq!(args, ["cargo-fy-docs", "build"]);
    }

    #[test]
    fn keeps_direct_executable_arguments() {
        let args = cargo_external_args([OsString::from("cargo-fy-docs"), OsString::from("build")]);
        assert_eq!(args, ["cargo-fy-docs", "build"]);
    }

    #[test]
    fn cli_parses_all_subcommands_and_flags() {
        use clap::Parser;

        // 1. Preview default
        let cli = Cli::parse_from(["cargo-fy-docs", "--port", "9090", "--no-open"]);
        assert_eq!(cli.port, 9090);
        assert!(cli.no_open);
        assert!(cli.command.is_none());

        // 2. init
        let cli = Cli::parse_from(["cargo-fy-docs", "init"]);
        assert!(matches!(cli.command, Some(Command::Init)));

        // 3. build
        let cli = Cli::parse_from(["cargo-fy-docs", "build", "--with-pdf"]);
        assert!(matches!(cli.command, Some(Command::Build)));
        assert!(cli.with_pdf);

        // 4. pdf
        let cli = Cli::parse_from(["cargo-fy-docs", "pdf"]);
        assert!(matches!(cli.command, Some(Command::Pdf)));
    }

    #[tokio::test]
    async fn bind_allocates_available_listener() {
        let listener = bind(18585).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(port >= 18585);
    }

    #[tokio::test]
    async fn dispatch_executes_build_pdf_and_init_commands() {
        let temp = std::env::temp_dir().join(format!("fy-docs-main-run-{}", std::process::id()));
        let docs = temp.join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(docs.join("main.typ"), "= Test Main\n\nMain content\n").unwrap();

        // 1. Build command (with PDF)
        let cli_build = Cli {
            root: Some(temp.clone()),
            with_pdf: true,
            port: 8181,
            no_open: true,
            command: Some(Command::Build),
        };
        let res_build = dispatch(cli_build, &temp).await;
        assert!(res_build.is_ok());
        assert!(docs.join("target").join("index.html").exists());

        // 2. Pdf command
        let cli_pdf = Cli {
            root: Some(temp.clone()),
            with_pdf: false,
            port: 8181,
            no_open: true,
            command: Some(Command::Pdf),
        };
        let res_pdf = dispatch(cli_pdf, &temp).await;
        assert!(res_pdf.is_ok());

        // 3. Init command on empty dir
        let temp_init =
            std::env::temp_dir().join(format!("fy-docs-main-init-{}", std::process::id()));
        std::fs::create_dir_all(&temp_init).unwrap();
        let cli_init = Cli {
            root: None,
            with_pdf: false,
            port: 8181,
            no_open: true,
            command: Some(Command::Init),
        };
        assert!(dispatch(cli_init, &temp_init).await.is_ok());
        assert!(temp_init.join("docs").join("main.typ").exists());

        let _ = std::fs::remove_dir_all(temp);
        let _ = std::fs::remove_dir_all(temp_init);
    }
}
