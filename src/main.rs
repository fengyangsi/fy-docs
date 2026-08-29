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
use std::path::PathBuf;
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

    // Init does not require an existing docs/ directory.
    if matches!(cli.command, Some(Command::Init)) {
        return scaffold::init(&cwd);
    }

    let project = project::detect(&cwd, cli.root.as_deref())?;
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
}
