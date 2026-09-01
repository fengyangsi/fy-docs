//! Terminal output: progress logging and people-friendly path display.

use std::path::Path;

/// Writes a progress line to stderr. A closed output pipe (e.g. the server is
/// piped into `head`) must not panic the watcher thread, so write errors are
/// deliberately ignored.
pub(crate) fn log(message: &str) {
    use std::io::Write as _;
    let _ = writeln!(std::io::stderr().lock(), "{message}");
}

/// Drops Windows' internal verbatim prefix (`\\?\`, and `\\?\UNC\` for network
/// paths) from path-like text so terminal output stays readable. Occurrences
/// anywhere in the text are removed, not just a leading one, because typst
/// embeds these paths inside its diagnostics.
pub(crate) fn strip_verbatim(text: &str) -> String {
    if cfg!(windows) {
        text.replace(r"\\?\UNC\", r"\\").replace(r"\\?\", "")
    } else {
        text.to_owned()
    }
}

/// Formats a path for people instead of exposing Windows' internal verbatim
/// path prefix (for example `\\?\D:\...`) in terminal output.
pub(crate) fn display_path(path: &Path) -> String {
    strip_verbatim(&path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_executes_safely() {
        log("test log output");
    }

    #[test]
    fn displays_normal_windows_paths_without_verbatim_prefix() {
        #[cfg(windows)]
        assert_eq!(display_path(Path::new(r"\\?\D:\Code\fy")), r"D:\Code\fy");
    }

    #[cfg(windows)]
    #[test]
    fn strips_verbatim_prefixes_anywhere_in_text() {
        // typst embeds the path inside its diagnostic, not at the start.
        assert_eq!(
            strip_verbatim(r"warning: x ┌─ \\?\D:\Code\fy\docs\main.typ:42:1"),
            r"warning: x ┌─ D:\Code\fy\docs\main.typ:42:1"
        );
        assert_eq!(
            strip_verbatim(r"\\?\UNC\server\share\a.typ"),
            r"\\server\share\a.typ"
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn preserves_native_unix_paths() {
        assert_eq!(display_path(Path::new("/tmp/fy-docs")), "/tmp/fy-docs");
    }
}
