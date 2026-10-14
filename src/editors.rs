//! The editor catalog: the fixed set of editors the picker can offer.
//!
//! This module is data plus detection: [`CATALOG`] lists every editor the
//! picker can offer, and [`detect`] filters it down to the ones installed on
//! this machine. Launching lives in `src/launch.rs` (S10). Rows are listed in
//! the order the picker renders them, and [`detect`] preserves that order.

use std::process::{Command, Stdio};

/// Placeholder used in [`Editor::args`] for the directory to open. Detection
/// and launch code substitute the resolved absolute path at runtime.
pub(crate) const DIR_PLACEHOLDER: &str = "<dir>";

/// Whether an editor opens in its own GUI window or inside a terminal tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditorKind {
    /// Owns a window; launched as a detached process.
    Gui,
    /// Runs inside a terminal; launched in a new herdr tab.
    Terminal,
}

/// One catalog row.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Editor {
    /// Stable id persisted in the `last-editor` state file and passed through
    /// `OPEN_IN_EDITOR`.
    pub id: &'static str,
    /// Human-readable name shown in the picker.
    pub name: &'static str,
    /// GUI window or terminal tab.
    pub kind: EditorKind,
    /// Bare CLI command name probed on `PATH`.
    pub cmd: &'static str,
    /// Extra command names probed after [`Editor::cmd`], in order, both for
    /// detection ([`detect`]) and for `crate::launch::find_cli`. Some distros
    /// ship an editor only under an alias (Debian and Ubuntu package the Zed
    /// CLI as `zeditor`), so probing the alias keeps such installs detectable
    /// and launchable. Empty for every row whose command name is the only one.
    pub alts: &'static [&'static str],
    /// argv template that opens a directory; the [`DIR_PLACEHOLDER`] element is
    /// replaced by the resolved directory at launch time.
    pub args: &'static [&'static str],
    /// macOS app bundle stem (without the `.app` extension) searched under
    /// `/Applications` and `~/Applications`. `None` for terminal editors.
    pub bundle: Option<&'static str>,
    /// Path of the CLI inside `bundle`, relative to the bundle root, where the
    /// bundle ships one. `None` when the bundle has no CLI (launch then falls
    /// back to `open -na "<bundle>.app" <dir>`).
    pub bundle_cli: Option<&'static str>,
}

/// Every editor the picker can offer, in menu order.
pub(crate) const CATALOG: &[Editor] = &[
    Editor {
        id: "zed",
        name: "Zed",
        kind: EditorKind::Gui,
        cmd: "zed",
        alts: &["zeditor"],
        args: &["-n", DIR_PLACEHOLDER],
        bundle: Some("Zed"),
        bundle_cli: Some("Contents/MacOS/cli"),
    },
    Editor {
        id: "code",
        name: "VS Code",
        kind: EditorKind::Gui,
        cmd: "code",
        alts: &[],
        args: &["-n", DIR_PLACEHOLDER],
        bundle: Some("Visual Studio Code"),
        bundle_cli: Some("Contents/Resources/app/bin/code"),
    },
    Editor {
        id: "cursor",
        name: "Cursor",
        kind: EditorKind::Gui,
        cmd: "cursor",
        alts: &[],
        args: &["-n", DIR_PLACEHOLDER],
        bundle: Some("Cursor"),
        bundle_cli: Some("Contents/Resources/app/bin/cursor"),
    },
    Editor {
        id: "windsurf",
        name: "Windsurf",
        kind: EditorKind::Gui,
        cmd: "windsurf",
        alts: &[],
        args: &["-n", DIR_PLACEHOLDER],
        bundle: Some("Windsurf"),
        bundle_cli: Some("Contents/Resources/app/bin/windsurf"),
    },
    Editor {
        id: "idea",
        name: "IntelliJ IDEA",
        kind: EditorKind::Gui,
        cmd: "idea",
        alts: &[],
        args: &[DIR_PLACEHOLDER],
        bundle: Some("IntelliJ IDEA"),
        bundle_cli: Some("Contents/MacOS/idea"),
    },
    Editor {
        id: "webstorm",
        name: "WebStorm",
        kind: EditorKind::Gui,
        cmd: "webstorm",
        alts: &[],
        args: &[DIR_PLACEHOLDER],
        bundle: Some("WebStorm"),
        bundle_cli: Some("Contents/MacOS/webstorm"),
    },
    Editor {
        id: "pycharm",
        name: "PyCharm",
        kind: EditorKind::Gui,
        cmd: "pycharm",
        alts: &[],
        args: &[DIR_PLACEHOLDER],
        bundle: Some("PyCharm"),
        bundle_cli: Some("Contents/MacOS/pycharm"),
    },
    Editor {
        id: "android-studio",
        name: "Android Studio",
        kind: EditorKind::Gui,
        cmd: "studio",
        alts: &[],
        args: &[DIR_PLACEHOLDER],
        bundle: Some("Android Studio"),
        bundle_cli: Some("Contents/MacOS/studio"),
    },
    Editor {
        id: "nvim",
        name: "nvim",
        kind: EditorKind::Terminal,
        cmd: "nvim",
        alts: &[],
        args: &[DIR_PLACEHOLDER],
        bundle: None,
        bundle_cli: None,
    },
    Editor {
        id: "vim",
        name: "vim",
        kind: EditorKind::Terminal,
        cmd: "vim",
        alts: &[],
        args: &[DIR_PLACEHOLDER],
        bundle: None,
        bundle_cli: None,
    },
    Editor {
        id: "helix",
        name: "helix",
        kind: EditorKind::Terminal,
        cmd: "hx",
        alts: &[],
        args: &[DIR_PLACEHOLDER],
        bundle: None,
        bundle_cli: None,
    },
];

/// Installed editors, in catalog order.
///
/// An editor counts as installed when one of its command names responds on
/// `PATH`, or (on macOS) when its app bundle is present under `/Applications`
/// or `~/Applications`.
pub(crate) fn detect() -> Vec<&'static Editor> {
    detect_with(&probe_on_path, &bundle_installed)
}

/// The command names probed for a row, in order: the bare [`Editor::cmd`]
/// first, then every [`Editor::alts`] entry. Shared by [`detect`] and
/// `crate::launch::find_cli` so detection and launch always agree on which
/// names identify a row.
pub(crate) fn probe_names(editor: &Editor) -> impl Iterator<Item = &'static str> + '_ {
    std::iter::once(editor.cmd).chain(editor.alts.iter().copied())
}

/// Pure core of [`detect`]: `probe` answers whether a CLI command is on
/// `PATH`, `bundle_exists` answers whether the row's app bundle is installed.
/// Kept injectable so tests do not depend on the host.
fn detect_with(
    probe: &dyn Fn(&str) -> bool,
    bundle_exists: &dyn Fn(&Editor) -> bool,
) -> Vec<&'static Editor> {
    CATALOG
        .iter()
        .filter(|editor| probe_names(editor).any(probe) || bundle_exists(editor))
        .collect()
}

/// `<cmd> --version` with both streams discarded; exit status 0 means the CLI
/// is installed and runnable. A missing command makes `status()` fail.
fn probe_on_path(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// App-bundle lookup for a catalog row.
///
/// Reads the row's bundle fields on every platform so the catalog keeps one
/// shape; the platform difference lives in [`bundle_present`]. On macOS the
/// bundle is accepted only when its `.app` directory exists and, when the row
/// also ships an inner CLI, that CLI exists (or is executable) inside the
/// bundle, so a leftover empty bundle directory is not mistaken for a working
/// install.
fn bundle_installed(editor: &Editor) -> bool {
    editor
        .bundle
        .is_some_and(|stem| bundle_present(stem, editor.bundle_cli))
}

/// macOS app-bundle directory probe for a row's bundle stem.
#[cfg(target_os = "macos")]
fn bundle_present(stem: &str, inner_cli: Option<&str>) -> bool {
    applications_dirs().iter().any(|dir| {
        let bundle = dir.join(format!("{stem}.app"));
        if !bundle.is_dir() {
            return false;
        }
        match inner_cli {
            None => true,
            Some(cli) => {
                let cli = bundle.join(cli);
                cli.exists() || is_executable(&cli)
            }
        }
    })
}

/// Linux and Windows have no app bundles; the CLI probe is the only source.
#[cfg(not(target_os = "macos"))]
fn bundle_present(_stem: &str, _inner_cli: Option<&str>) -> bool {
    false
}

#[cfg(target_os = "macos")]
fn applications_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = vec![std::path::PathBuf::from("/Applications")];
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(std::path::PathBuf::from(home).join("Applications"));
    }
    dirs
}

#[cfg(target_os = "macos")]
fn is_executable(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_lists_eleven_rows_in_menu_order() {
        let ids: Vec<&str> = CATALOG.iter().map(|e| e.id).collect();
        assert_eq!(
            ids,
            [
                "zed",
                "code",
                "cursor",
                "windsurf",
                "idea",
                "webstorm",
                "pycharm",
                "android-studio",
                "nvim",
                "vim",
                "helix",
            ]
        );
    }

    #[test]
    fn every_row_has_a_non_empty_name_command_and_argv() {
        for e in CATALOG {
            assert!(!e.name.is_empty(), "{}: empty name", e.id);
            assert!(!e.cmd.is_empty(), "{}: empty cmd", e.id);
            assert!(!e.args.is_empty(), "{}: empty args", e.id);
        }
    }

    #[test]
    fn every_argv_template_contains_the_dir_placeholder_once() {
        for e in CATALOG {
            let hits = e.args.iter().filter(|a| **a == DIR_PLACEHOLDER).count();
            assert_eq!(hits, 1, "{}: {} placeholder(s) in {:?}", e.id, hits, e.args);
        }
    }

    #[test]
    fn gui_rows_have_a_bundle() {
        for e in CATALOG.iter().filter(|e| e.kind == EditorKind::Gui) {
            assert!(e.bundle.is_some(), "{}: GUI row without bundle", e.id);
        }
    }

    #[test]
    fn bundle_cli_paths_are_relative_and_paired_with_a_bundle() {
        for e in CATALOG {
            if let Some(cli) = e.bundle_cli {
                assert!(!cli.is_empty(), "{}: empty bundle_cli", e.id);
                assert!(!cli.starts_with('/'), "{}: absolute bundle_cli", e.id);
                assert!(e.bundle.is_some(), "{}: bundle_cli without bundle", e.id);
            }
        }
    }

    #[test]
    fn detect_accepts_a_command_alias() {
        // Debian and Ubuntu ship the Zed CLI only as `zeditor`; with no bare
        // `zed` command the row must still be detected through its alias.
        let probe = |cmd: &str| cmd == "zeditor";
        let bundle = |_editor: &Editor| false;
        let found = detect_with(&probe, &bundle);
        let ids: Vec<&str> = found.iter().map(|e| e.id).collect();
        assert_eq!(ids, ["zed"]);
        // The same alias list is what `find_cli` probes on `PATH`.
        let zed = CATALOG.iter().find(|e| e.id == "zed").expect("zed row");
        assert_eq!(
            probe_names(zed).collect::<Vec<_>>(),
            ["zed", "zeditor"],
            "one alias per non-command name, after the command"
        );
        let vim = CATALOG.iter().find(|e| e.id == "vim").expect("vim row");
        assert_eq!(probe_names(vim).collect::<Vec<_>>(), ["vim"]);
    }

    #[test]
    fn detect_keeps_probe_hits_in_catalog_order() {
        let probe = |cmd: &str| cmd == "windsurf" || cmd == "nvim";
        let bundle = |_editor: &Editor| false;
        let found = detect_with(&probe, &bundle);
        let ids: Vec<&str> = found.iter().map(|e| e.id).collect();
        assert_eq!(ids, ["windsurf", "nvim"]);
    }

    #[test]
    fn detect_accepts_a_macos_bundle_without_a_cli_probe() {
        let probe = |_cmd: &str| false;
        let bundle = |editor: &Editor| editor.bundle == Some("Android Studio");
        let found = detect_with(&probe, &bundle);
        let ids: Vec<&str> = found.iter().map(|e| e.id).collect();
        assert_eq!(ids, ["android-studio"]);
    }

    #[test]
    fn detect_returns_empty_when_nothing_is_installed() {
        let probe = |_cmd: &str| false;
        let bundle = |_editor: &Editor| false;
        let found = detect_with(&probe, &bundle);
        assert!(found.is_empty(), "expected no editors, got {found:?}");
    }

    #[test]
    fn terminal_rows_are_nvim_vim_and_helix_and_have_no_bundle() {
        let terminals: Vec<&str> = CATALOG
            .iter()
            .filter(|e| e.kind == EditorKind::Terminal)
            .map(|e| e.id)
            .collect();
        assert_eq!(terminals, ["nvim", "vim", "helix"]);
        for e in CATALOG.iter().filter(|e| e.kind == EditorKind::Terminal) {
            assert!(e.bundle.is_none(), "{}: terminal row with bundle", e.id);
        }
    }
}
