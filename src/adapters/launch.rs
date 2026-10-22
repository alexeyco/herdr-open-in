//! Locating and launching a GUI editor.
//!
//! A GUI editor is launched on the resolved workspace directory and then left
//! alone: [`launch_gui`] returns as soon as the process is spawned and never
//! waits for its exit status. A CLI is preferred ([`find_cli`]); on macOS a row
//! whose app bundle is installed but whose CLI cannot be probed falls back to
//! `open -n -a "<bundle>" <dir>` (Android Studio ships no `studio` CLI here).

use std::process::{Command, Stdio};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use crate::adapters::editors::{DIR_PLACEHOLDER, Editor, probe_names};

/// Program used for the macOS app-bundle fallback.
const OPEN_PROGRAM: &str = "open";

/// macOS application directory searched for app bundles.
#[cfg(target_os = "macos")]
const MACOS_APPLICATIONS_DIR: &str = "/Applications";

/// Locates the CLI for `editor`.
///
/// Probes the bare command on `PATH` first, then the known absolute install
/// locations for the current OS. Returns the command string to execute (not a
/// `PathBuf`) so callers can hand it straight to [`Command::new`].
pub(crate) fn find_cli(editor: &Editor) -> Option<String> {
    find_cli_with(editor, &probe)
}

/// Launches `editor` on `dir` and returns once the process has detached.
///
/// Prefers a CLI found by [`find_cli`]; otherwise (macOS only) it opens the app
/// bundle via [`OPEN_PROGRAM`]. A spawn error names the editor and the program.
pub(crate) fn launch_gui(editor: &Editor, dir: &str) -> Result<(), String> {
    if let Some(cli) = find_cli(editor) {
        return spawn_detached(editor, &cli, &cli_argv(editor, dir));
    }
    if let Some(result) = launch_bundle(editor, dir) {
        return result;
    }
    Err(launcher_missing(editor))
}

/// Pure core of [`find_cli`]: `probe` answers whether a command or absolute
/// path is runnable. The bare command and every alias are probed on `PATH`
/// first, then the known absolute install locations. Kept injectable so tests
/// do not depend on the host.
fn find_cli_with(editor: &Editor, probe: &dyn Fn(&str) -> bool) -> Option<String> {
    if let Some(name) = probe_names(editor).find(|name| probe(name)) {
        return Some(name.to_string());
    }
    fallbacks(editor).into_iter().find(|path| probe(path))
}

/// argv template for a CLI launch, with the directory placeholder substituted.
fn cli_argv(editor: &Editor, dir: &str) -> Vec<String> {
    editor
        .args
        .iter()
        .map(|arg| {
            if *arg == DIR_PLACEHOLDER {
                dir.to_string()
            } else {
                (*arg).to_string()
            }
        })
        .collect()
}

/// argv passed to [`OPEN_PROGRAM`] for a bundle launch: a new instance (`-n`)
/// of the application (`-a`) opening `dir`.
fn bundle_argv(bundle: &str, dir: &str) -> Vec<String> {
    vec![
        "-n".to_string(),
        "-a".to_string(),
        bundle.to_string(),
        dir.to_string(),
    ]
}

/// Bundle fallback shared by every platform: returns `None` when the row has
/// no bundle or the bundle is not installed on this host, so [`launch_gui`]
/// can report a missing launcher. Off macOS [`bundle_dir`] never finds an app
/// bundle, so only the CLI path can succeed there.
fn launch_bundle(editor: &Editor, dir: &str) -> Option<Result<(), String>> {
    let stem = editor.bundle?;
    let bundle = bundle_dir(stem)?;
    Some(spawn_detached(
        editor,
        OPEN_PROGRAM,
        &bundle_argv(&bundle, dir),
    ))
}

/// Builds the command used by [`spawn_detached`]: `program` with `argv`, all
/// standard streams discarded, and (on unix) the child placed in its own
/// process group.
///
/// The picker runs as a herdr popup pane command, which makes it the session
/// leader of its own pty. When the popup exits, the pty is torn down and the
/// kernel hangs up the terminal, which sends `SIGHUP` to the foreground
/// process group of that pty — the group the picker itself belonged to. A GUI
/// editor spawned as a plain child would inherit that group, receive the
/// `SIGHUP` and die right after flashing on screen.
///
/// `process_group(0)` puts the child in a fresh process group whose id is the
/// child's own pid. That addresses both halves of the problem: the terminal's
/// foreground group is still the picker's group, so the child is not the
/// recipient of the terminal's `SIGHUP`, and the child is no longer in the
/// group the picker exits out of. The child does share the picker's session
/// and controlling terminal, but its standard streams are `/dev/null` and it is
/// not the foreground group, so no terminal signal reaches it. `process_group`
/// is the safe std equivalent of the `setpgid` half of a `setsid`; std exposes
/// no safe `setsid`, and `pre_exec` would need `unsafe`, which is forbidden.
fn detached_command(program: &str, argv: &[String]) -> Command {
    let mut command = Command::new(program);
    command
        .args(argv)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    command.process_group(0);
    command
}

/// Spawns `program` with `argv`, all standard streams discarded, and does not
/// wait: the caller succeeds as soon as the process has detached. See
/// [`detached_command`] for why the child is put in its own process group.
fn spawn_detached(editor: &Editor, program: &str, argv: &[String]) -> Result<(), String> {
    detached_command(program, argv)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("failed to launch {} with {program}: {e}", editor.name))
}

/// `<cmd> --version` with both streams discarded; exit status 0 means the CLI
/// is installed and runnable. A missing command makes `status()` fail.
fn probe(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Known absolute CLI locations after the bare command, per OS. Every command
/// name (the bare command and each alias) is tried in each location.
#[cfg(target_os = "macos")]
fn fallbacks(editor: &Editor) -> Vec<String> {
    let mut paths: Vec<String> = probe_names(editor)
        .map(|name| format!("/usr/local/bin/{name}"))
        .collect();
    if let (Some(stem), Some(inner)) = (editor.bundle, editor.bundle_cli) {
        paths.push(format!("/Applications/{stem}.app/{inner}"));
    }
    paths
}

/// Linux install locations, including snap's two layouts. Every command name
/// is tried in each directory, so `/usr/bin/zeditor` (the Debian and Ubuntu
/// package name of the Zed CLI) is found alongside `/usr/bin/zed`.
#[cfg(target_os = "linux")]
fn fallbacks(editor: &Editor) -> Vec<String> {
    let dirs = ["/usr/bin", "/usr/snap/bin", "/snap/bin"];
    let mut paths = Vec::new();
    for dir in dirs {
        for name in probe_names(editor) {
            paths.push(format!("{dir}/{name}"));
        }
    }
    paths
}

/// Windows probes `PATH` only.
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn fallbacks(_editor: &Editor) -> Vec<String> {
    Vec::new()
}

/// macOS app-bundle directory lookup for a row's bundle stem under
/// `/Applications` and `~/Applications`. Returns the `.app` path as a string
/// when the directory exists.
#[cfg(target_os = "macos")]
fn bundle_dir(stem: &str) -> Option<String> {
    let name = format!("{stem}.app");
    let mut candidates = vec![std::path::Path::new(MACOS_APPLICATIONS_DIR).join(&name)];
    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(std::path::Path::new(&home).join("Applications").join(&name));
    }
    candidates
        .into_iter()
        .find(|path| path.is_dir())
        .map(|path| path.display().to_string())
}

/// Linux and Windows have no app bundles; no directory can match.
#[cfg(not(target_os = "macos"))]
fn bundle_dir(_stem: &str) -> Option<String> {
    None
}

/// Error for a GUI editor that has neither a runnable CLI nor an installed
/// bundle, naming the editor and the command that was expected.
fn launcher_missing(editor: &Editor) -> String {
    format!(
        "cannot launch {}: no `{}` CLI found on PATH or at the known install locations",
        editor.name, editor.cmd
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::editors::CATALOG;

    fn row(id: &str) -> &'static Editor {
        match CATALOG.iter().find(|editor| editor.id == id) {
            Some(editor) => editor,
            None => panic!("catalog row `{id}` is missing"),
        }
    }

    #[test]
    fn cli_argv_substitutes_the_directory_for_the_placeholder() {
        assert_eq!(cli_argv(row("zed"), "/home/u/demo"), ["-n", "/home/u/demo"]);
    }

    #[test]
    fn cli_argv_keeps_rows_without_the_new_window_flag() {
        assert_eq!(
            cli_argv(row("android-studio"), "/home/u/demo"),
            ["/home/u/demo"]
        );
        assert_eq!(cli_argv(row("nvim"), "/home/u/demo"), ["/home/u/demo"]);
    }

    #[test]
    fn cli_argv_substitutes_the_placeholder_exactly_once_for_every_row() {
        for editor in CATALOG {
            let argv = cli_argv(editor, "/home/u/demo");
            let hits = argv
                .iter()
                .filter(|arg| arg.as_str() == "/home/u/demo")
                .count();
            assert_eq!(hits, 1, "{}: {argv:?}", editor.id);
        }
    }

    #[test]
    fn bundle_argv_opens_a_new_instance_of_the_app_on_the_dir() {
        assert_eq!(
            bundle_argv("/Applications/Android Studio.app", "/home/u/demo"),
            [
                "-n",
                "-a",
                "/Applications/Android Studio.app",
                "/home/u/demo"
            ]
        );
    }

    #[test]
    fn find_cli_returns_the_bare_command_when_it_probes_true() {
        let probe = |cmd: &str| cmd == "zed";
        assert_eq!(find_cli_with(row("zed"), &probe).as_deref(), Some("zed"));
    }

    #[test]
    fn find_cli_returns_none_when_nothing_probes_true() {
        let probe = |_cmd: &str| false;
        assert_eq!(find_cli_with(row("zed"), &probe), None);
        assert_eq!(find_cli_with(row("android-studio"), &probe), None);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn find_cli_falls_back_to_the_usr_local_bin_location() {
        let probe = |cmd: &str| cmd == "/usr/local/bin/zed";
        assert_eq!(
            find_cli_with(row("zed"), &probe).as_deref(),
            Some("/usr/local/bin/zed")
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn find_cli_falls_back_to_the_bundle_inner_cli() {
        let probe = |cmd: &str| cmd == "/Applications/Zed.app/Contents/MacOS/cli";
        assert_eq!(
            find_cli_with(row("zed"), &probe).as_deref(),
            Some("/Applications/Zed.app/Contents/MacOS/cli")
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn find_cli_falls_back_to_linux_install_locations() {
        let probe = |cmd: &str| cmd == "/snap/bin/zed";
        assert_eq!(
            find_cli_with(row("zed"), &probe).as_deref(),
            Some("/snap/bin/zed")
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn find_cli_falls_back_to_the_distro_zeditor_path() {
        let probe = |cmd: &str| cmd == "/usr/bin/zeditor";
        assert_eq!(
            find_cli_with(row("zed"), &probe).as_deref(),
            Some("/usr/bin/zeditor")
        );
    }

    #[test]
    fn launcher_missing_names_the_editor_and_the_command() {
        let message = launcher_missing(row("android-studio"));
        assert!(message.contains("Android Studio"), "unhelpful: {message}");
        assert!(message.contains("studio"), "unhelpful: {message}");
    }

    /// Process-group id of `pid`, read with `ps -o pgid=`. `None` when `ps`
    /// cannot answer or the output is not a number.
    #[cfg(unix)]
    fn process_group_of(pid: u32) -> Option<u32> {
        let output = Command::new("ps")
            .args(["-o", "pgid=", "-p", &pid.to_string()])
            .output()
            .ok()?;
        String::from_utf8_lossy(&output.stdout).trim().parse().ok()
    }

    /// The detached launch path puts the child in its own process group, so the
    /// `SIGHUP` the pty sends to the picker's foreground group on popup exit
    /// does not reach it. Spawns a real child through the production command
    /// builder and compares its process group with the test process's own.
    #[cfg(unix)]
    #[test]
    fn detached_child_leads_its_own_process_group() {
        // `sleep` does not fork, so killing the direct child reclaims it.
        let argv = vec!["5".to_string()];
        let mut child = match detached_command("sleep", &argv).spawn() {
            Ok(child) => child,
            Err(e) => panic!("failed to spawn the process-group test child: {e}"),
        };
        let child_pgid = process_group_of(child.id());
        let parent_pgid = process_group_of(std::process::id());
        // Clean up before asserting so a failed assertion leaks no `sleep`.
        let _ = child.kill();
        let _ = child.wait();

        assert!(
            child_pgid.is_some(),
            "could not read the child's process group with `ps`"
        );
        assert_ne!(
            child_pgid, parent_pgid,
            "the detached child shares the parent's process group"
        );
        assert_eq!(
            child_pgid,
            Some(child.id()),
            "process_group(0) must make the child its own group leader"
        );
    }
}
