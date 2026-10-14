//! The interactive editor picker: a tiny raw-mode TUI that renders one row per
//! detected editor and returns the chosen one.
//!
//! Raw mode is entered only through an `stty` child process — no FFI, no
//! `unsafe`, no extra dependency. The saved `stty -g` settings and the cursor
//! are restored on every exit path: an RAII guard restores on drop and [`run`]
//! also restores explicitly before returning, so the terminal is sane even
//! when a later write fails. Every picker path is `#[cfg(unix)]`; non-unix
//! builds get a stub that returns an error.
//!
//! The key loop and the frame builder are split from the terminal plumbing so
//! tests can drive them over an in-memory `Read`/`Write` pair without a real
//! terminal.
//!
//! Raw mode is set with `min 0 time 1` (VMIN=0, VTIME=1): every `read` returns
//! within 0.1 s instead of blocking, so on a real terminal a lone Escape is
//! followed by a timed-out read rather than a wait for a second byte. A 0-byte
//! read therefore means "no key within the timeout" on a tty; on any other
//! reader (the slices the tests drive, or a closed stdin) it means end of
//! input and ends the picker. [`run`] decides which meaning applies with
//! `IsTerminal` and passes it down to the key loop.

use crate::editors::Editor;

#[cfg(unix)]
use std::io::{ErrorKind, IsTerminal, Read, Write};
#[cfg(unix)]
use std::process::{Command, Stdio};

/// Error returned when the caller hands the picker an empty catalog.
#[cfg(unix)]
const NO_EDITORS: &str = "no editors to pick from";

/// Hides the hardware cursor while the picker redraws.
#[cfg(unix)]
const HIDE_CURSOR: &str = "\x1b[?25l";

/// Shows the hardware cursor again; written on every restore path.
#[cfg(unix)]
const SHOW_CURSOR: &str = "\x1b[?25h";

/// Moves the cursor to the top-left corner before redrawing.
#[cfg(unix)]
const HOME: &str = "\x1b[H";

/// Clears from the cursor to the end of the screen before redrawing.
#[cfg(unix)]
const CLEAR_TAIL: &str = "\x1b[J";

/// Cyan SGR that colours the selected row's marker arrow.
#[cfg(unix)]
const MARKER: &str = "\x1b[36m";

/// The one-column arrow drawn before the selected row's name. Together with
/// the following space it forms the same two-column prefix the unselected
/// rows use.
#[cfg(unix)]
const ARROW: &str = "\u{25b8}";

/// Two-column space prefix drawn before every unselected row so the names stay
/// in the same column as the selected one.
#[cfg(unix)]
const UNSELECTED: &str = "  ";

/// Bold SGR used to mark the selected row's text; no background colour.
#[cfg(unix)]
const SELECTED: &str = "\x1b[1m";

/// Resets all SGR attributes.
#[cfg(unix)]
const RESET: &str = "\x1b[0m";

/// Runs the picker and returns the chosen editor, or `None` when the user
/// cancels. `preselect` is the id of the editor to pre-mark; an unknown id
/// falls back to the first row. An empty catalog is an error.
#[cfg(unix)]
pub(crate) fn run(
    items: &[&'static Editor],
    preselect: Option<&str>,
) -> Result<Option<&'static Editor>, String> {
    if items.is_empty() {
        return Err(NO_EDITORS.to_string());
    }

    // A tty, entered below with `min 0 time 1`, reports a 0-byte read on a
    // timeout; every other reader reports end of input there. Record which one
    // this run reads from so the key loop can tell a bare Escape from a closed
    // stdin.
    let read_timeout = std::io::stdin().is_terminal();

    let mut raw = RawMode::enter()?;
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let outcome = run_with(items, preselect, &mut stdin, &mut stdout, read_timeout);
    // Release the stdout lock before the guard writes the show-cursor sequence.
    drop(stdout);
    drop(stdin);
    // Explicit restore before returning so the terminal is sane even if a later
    // write fails; `RawMode::drop` is the backstop for early returns.
    let restored = raw.restore();

    match (outcome, restored) {
        (Err(outcome), _) => Err(outcome),
        (Ok(_), Err(restore)) => Err(restore),
        (Ok(choice), Ok(())) => Ok(choice),
    }
}

/// Non-unix stub: the raw-mode picker needs a POSIX terminal.
#[cfg(not(unix))]
pub(crate) fn run(
    _items: &[&'static Editor],
    _preselect: Option<&str>,
) -> Result<Option<&'static Editor>, String> {
    Err("the editor picker requires a terminal and is only supported on unix".to_string())
}

/// Draw loop and key loop, driven by injected reader/writer seams.
#[cfg(unix)]
fn run_with<R: Read, W: Write>(
    items: &[&'static Editor],
    preselect: Option<&str>,
    mut reader: R,
    mut writer: W,
    read_timeout: bool,
) -> Result<Option<&'static Editor>, String> {
    if items.is_empty() {
        return Err(NO_EDITORS.to_string());
    }

    let mut selected = initial_selection(items, preselect);
    let mut keys = Keys::new(&mut reader, read_timeout);
    loop {
        let drawn = frame(items, selected);
        writer
            .write_all(drawn.as_bytes())
            .and_then(|()| writer.flush())
            .map_err(|e| format!("cannot draw the editor picker: {e}"))?;

        match read_key(&mut keys)? {
            Key::Up => selected = selected.saturating_sub(1),
            Key::Down => selected = (selected + 1).min(items.len() - 1),
            Key::Enter => return Ok(Some(items[selected])),
            Key::Cancel | Key::Eof => return Ok(None),
            Key::Timeout | Key::Other => {}
        }
    }
}

/// Index of the row to pre-mark: the `preselect` id when it is in `items`,
/// otherwise the first row.
#[cfg(unix)]
fn initial_selection(items: &[&'static Editor], preselect: Option<&str>) -> usize {
    preselect
        .and_then(|id| items.iter().position(|editor| editor.id == id))
        .unwrap_or(0)
}

/// Renders one full frame: hide the cursor, home, clear the tail, then one
/// blank line, one padded row per editor with the selected row marked by a
/// cyan `▸` arrow and bold text, and a trailing blank line. The whole frame is
/// returned as a single string so the loop can write it in one call.
#[cfg(unix)]
fn frame(items: &[&'static Editor], selected: usize) -> String {
    let width = items
        .iter()
        .map(|editor| editor.name.len())
        .max()
        .unwrap_or(0);
    let mut out = String::new();
    out.push_str(HIDE_CURSOR);
    out.push_str(HOME);
    out.push_str(CLEAR_TAIL);
    // One blank line above the rows.
    out.push_str("\r\n");
    for (index, editor) in items.iter().enumerate() {
        if index == selected {
            out.push_str(MARKER);
            out.push_str(ARROW);
            out.push_str(RESET);
            out.push(' ');
            out.push_str(SELECTED);
        } else {
            out.push_str(UNSELECTED);
        }
        out.push_str(editor.name);
        out.push_str(&" ".repeat(width.saturating_sub(editor.name.len())));
        if index == selected {
            out.push_str(RESET);
        }
        out.push_str("\r\n");
    }
    // One blank line below the rows.
    out.push_str("\r\n");
    out
}

/// One result from [`Keys::next_byte`].
#[cfg(unix)]
enum Byte {
    /// A byte arrived.
    Value(u8),
    /// The `min 0 time 1` read timeout elapsed with no key on a tty.
    TimedOut,
    /// End of input on a reader that is not a tty.
    Eof,
}

/// One decoded key press.
#[cfg(unix)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Key {
    Up,
    Down,
    Enter,
    Cancel,
    Eof,
    /// No key arrived within the read timeout; keep the loop running.
    Timeout,
    Other,
}

/// Decodes the next key, translating the arrow, movement, enter, cancel and
/// end-of-input cases and ignoring everything else.
#[cfg(unix)]
fn read_key<R: Read>(keys: &mut Keys<R>) -> Result<Key, String> {
    let byte = match keys.next_byte()? {
        Byte::Value(byte) => byte,
        // A timed-out read with no key pending is not a cancel; return to the
        // loop so the picker keeps running.
        Byte::TimedOut => return Ok(Key::Timeout),
        Byte::Eof => return Ok(Key::Eof),
    };
    match byte {
        b'\r' | b'\n' => Ok(Key::Enter),
        b'q' => Ok(Key::Cancel),
        b'k' => Ok(Key::Up),
        b'j' => Ok(Key::Down),
        0x1b => read_escape(keys),
        _ => Ok(Key::Other),
    }
}

/// Decodes an escape sequence. A lone `\x1b` (nothing follows it) is a cancel;
/// a partially read sequence is ignored rather than treated as a bare Escape,
/// and a byte that does not belong to the sequence is pushed back so the next
/// key is decoded normally.
///
/// A preceding bare Escape is recognised both when the next read times out (a
/// real tty under `min 0 time 1`) and when it hits end of input (a slice in
/// tests).
#[cfg(unix)]
fn read_escape<R: Read>(keys: &mut Keys<R>) -> Result<Key, String> {
    let introducer = match keys.next_byte()? {
        Byte::Value(byte) => byte,
        Byte::TimedOut | Byte::Eof => return Ok(Key::Cancel),
    };
    match introducer {
        b'[' | b'O' => {
            let final_byte = match keys.next_byte()? {
                Byte::Value(byte) => byte,
                // Incomplete CSI/SS3 sequence: ignore, do not cancel.
                Byte::TimedOut | Byte::Eof => return Ok(Key::Other),
            };
            match final_byte {
                b'A' => Ok(Key::Up),
                b'B' => Ok(Key::Down),
                // A valid CSI final byte is consumed as part of the sequence;
                // anything else is a fresh key and is pushed back.
                0x40..=0x7e => Ok(Key::Other),
                other => {
                    keys.unread(other);
                    Ok(Key::Other)
                }
            }
        }
        other => {
            // Not a CSI/SS3 introducer (for example Alt+<key>): push the byte
            // back and ignore just the Escape.
            keys.unread(other);
            Ok(Key::Other)
        }
    }
}

/// A byte source with one byte of pushback.
#[cfg(unix)]
struct Keys<R> {
    reader: R,
    pending: Option<u8>,
    /// Whether a 0-byte read means the `min 0 time 1` timeout rather than end
    /// of input.
    read_timeout: bool,
}

#[cfg(unix)]
impl<R: Read> Keys<R> {
    fn new(reader: R, read_timeout: bool) -> Self {
        Self {
            reader,
            pending: None,
            read_timeout,
        }
    }

    /// Reads the next byte, retrying on `Interrupted`. A 0-byte read becomes
    /// [`Byte::TimedOut`] when `read_timeout` is set and [`Byte::Eof`]
    /// otherwise.
    ///
    /// A 0-byte read is ambiguous on its own: on a tty in raw mode with
    /// `min 0 time 1` (see [`RawMode::enter`]) it means the 0.1 s read timeout
    /// elapsed with no key, while on any other reader (a regular file, a
    /// closed stdin, or the in-memory slices the tests drive) it means end of
    /// input. `read_timeout` records which reader the caller reads from so a
    /// bare Escape followed by a timeout cancels while a closed stdin still
    /// exits the picker.
    fn next_byte(&mut self) -> Result<Byte, String> {
        if let Some(byte) = self.pending.take() {
            return Ok(Byte::Value(byte));
        }
        let mut buf = [0u8; 1];
        loop {
            match self.reader.read(&mut buf) {
                Ok(0) if self.read_timeout => return Ok(Byte::TimedOut),
                Ok(0) => return Ok(Byte::Eof),
                Ok(_) => return Ok(Byte::Value(buf[0])),
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(format!("cannot read from the terminal: {e}")),
            }
        }
    }

    /// Queues `byte` to be returned by the next [`Keys::next_byte`] call.
    fn unread(&mut self, byte: u8) {
        self.pending = Some(byte);
    }
}

/// Saved terminal settings plus a restore flag; restores raw mode and the
/// cursor on drop so no exit path can leave the terminal broken.
#[cfg(unix)]
struct RawMode {
    saved: String,
    restored: bool,
}

#[cfg(unix)]
impl RawMode {
    /// Saves the current settings with `stty -g`, then switches the terminal to
    /// raw mode with echo off and a 0.1 s read timeout (`min 0 time 1`, that
    /// is VMIN=0 and VTIME=1) so a `read` never blocks waiting for another
    /// byte. Errors when `stty` is missing or fails.
    fn enter() -> Result<Self, String> {
        let saved = stty(&["-g"])?.trim().to_string();
        if saved.is_empty() {
            return Err("stty -g did not report the terminal settings".to_string());
        }
        stty(&["raw", "-echo", "min", "0", "time", "1"])?;
        Ok(Self {
            saved,
            restored: false,
        })
    }

    /// Restores the saved settings and shows the cursor. Idempotent.
    fn restore(&mut self) -> Result<(), String> {
        if self.restored {
            return Ok(());
        }
        self.restored = true;
        let settings = stty(&[self.saved.as_str()]);
        let cursor = show_cursor();
        settings.and(cursor)
    }
}

#[cfg(unix)]
impl Drop for RawMode {
    fn drop(&mut self) {
        // Best effort: the explicit restore in `run` reports the error.
        let _ = self.restore();
    }
}

/// Shows the hardware cursor on stdout.
#[cfg(unix)]
fn show_cursor() -> Result<(), String> {
    let mut out = std::io::stdout();
    out.write_all(SHOW_CURSOR.as_bytes())
        .and_then(|()| out.flush())
        .map_err(|e| format!("cannot restore the terminal cursor: {e}"))
}

/// Runs `stty` with its stdin bound to the controlling terminal, returning its
/// stdout. An unset or unopenable `stty` is an error, never a silent fallback
/// into cooked mode.
#[cfg(unix)]
fn stty(args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("stty");
    cmd.args(args);
    match std::fs::File::open("/dev/tty") {
        Ok(tty) => cmd.stdin(tty),
        Err(_) => cmd.stdin(Stdio::inherit()),
    };
    let output = cmd.output().map_err(|e| {
        format!(
            "failed to run stty {}; is it installed? {e}",
            args.join(" ")
        )
    })?;
    if !output.status.success() {
        return Err(format!(
            "stty {} failed with {}",
            args.join(" "),
            output.status
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|e| format!("stty {} printed invalid UTF-8: {e}", args.join(" ")))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::editors::CATALOG;

    /// The first four catalog rows: Zed, VS Code, Cursor, Windsurf.
    fn items() -> Vec<&'static Editor> {
        CATALOG.iter().take(4).collect()
    }

    /// Drives the loop over an in-memory byte slice with no terminal involved.
    /// A slice is not a tty, so a 0-byte read is end of input, not a timeout.
    fn run_bytes(input: &[u8], preselect: Option<&str>) -> Result<Option<&'static Editor>, String> {
        let items = items();
        let mut out = Vec::new();
        let mut reader = input;
        run_with(&items, preselect, &mut reader, &mut out, false)
    }

    /// A key source for the timeout tests. Each entry is either a byte to yield
    /// or `None` to model a 0-byte read that timed out (a tty under
    /// `min 0 time 1`). An exhausted script is an error so a broken loop fails
    /// the test instead of spinning.
    struct ScriptedKeys {
        script: Vec<Option<u8>>,
        position: usize,
    }

    impl Read for ScriptedKeys {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if buf.is_empty() {
                return Ok(0);
            }
            match self.script.get(self.position) {
                Some(Some(byte)) => {
                    self.position += 1;
                    buf[0] = *byte;
                    Ok(1)
                }
                Some(None) => {
                    self.position += 1;
                    Ok(0)
                }
                None => Err(std::io::Error::other("scripted key source exhausted")),
            }
        }
    }

    /// Drives the loop over a scripted source with empty reads treated as
    /// timeouts, as a real tty under `min 0 time 1` behaves.
    fn run_scripted(
        script: &[Option<u8>],
        preselect: Option<&str>,
    ) -> Result<Option<&'static Editor>, String> {
        let items = items();
        let mut out = Vec::new();
        let mut reader = ScriptedKeys {
            script: script.to_vec(),
            position: 0,
        };
        run_with(&items, preselect, &mut reader, &mut out, true)
    }

    /// Asserts the shared frame shape: the cursor-hide, home and clear-tail
    /// sequences, exactly one blank line, the editor rows, then exactly one
    /// trailing blank line.
    fn assert_one_blank_line_above_and_below(drawn: &str) {
        let prefix = format!("{HIDE_CURSOR}{HOME}{CLEAR_TAIL}\r\n");
        assert!(
            drawn.starts_with(&prefix),
            "frame must start with the clear sequence and one blank line: {drawn:?}"
        );
        let body = &drawn[prefix.len()..];
        assert!(
            !body.starts_with("\r\n"),
            "frame must not start with a second blank line: {drawn:?}"
        );
        assert!(
            body.ends_with("\r\n\r\n"),
            "frame must end with exactly one blank line: {drawn:?}"
        );
        assert!(
            !body.ends_with("\r\n\r\n\r\n"),
            "frame must not end with a second blank line: {drawn:?}"
        );
    }

    #[test]
    fn down_down_enter_picks_the_third_item() {
        let picked = run_bytes(b"\x1b[B\x1b[B\r", None).expect("picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some("cursor"));
    }

    #[test]
    fn j_then_enter_picks_the_second_item() {
        let picked = run_bytes(b"j\r", None).expect("picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some("code"));
    }

    #[test]
    fn ss3_arrows_are_recognised() {
        let picked = run_bytes(b"\x1bOB\n", None).expect("picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some("code"));
    }

    #[test]
    fn escape_cancels() {
        let picked = run_bytes(b"\x1b", None).expect("picker should return");
        assert!(picked.is_none());
    }

    #[test]
    fn escape_then_timeout_cancels_without_another_byte() {
        // Regression for AC9: on a real tty (`min 0 time 1`) a lone Escape is
        // followed by a 0-byte read, not end of input, and must still cancel.
        let picked = run_scripted(&[Some(0x1b), None], None).expect("picker should return");
        assert!(picked.is_none());
    }

    #[test]
    fn timeout_with_no_key_does_not_exit_the_picker() {
        // A 0-byte read means "no key yet": the loop keeps running and the
        // following `j` + Enter still select the second row.
        let picked =
            run_scripted(&[None, Some(b'j'), Some(b'\r')], None).expect("picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some("code"));
    }

    #[test]
    fn incomplete_csi_followed_by_timeout_is_ignored() {
        // A CSI prefix that never completes is ignored rather than cancelled,
        // so the following Enter confirms the first row.
        let picked = run_scripted(&[Some(0x1b), Some(b'['), None, Some(b'\r')], None)
            .expect("picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some("zed"));
    }

    #[test]
    fn q_cancels() {
        let picked = run_bytes(b"q", None).expect("picker should return");
        assert!(picked.is_none());
    }

    #[test]
    fn eof_cancels() {
        let picked = run_bytes(b"", None).expect("picker should return");
        assert!(picked.is_none());
    }

    #[test]
    fn incomplete_escape_is_not_a_cancel() {
        // A partial CSI prefix must be ignored, so the following Enter confirms
        // the first row instead of cancelling the picker.
        let picked = run_bytes(b"\x1b[\r", None).expect("picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some("zed"));
    }

    #[test]
    fn unknown_keys_are_ignored() {
        let picked = run_bytes(b"xyz\r", None).expect("picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some("zed"));
    }

    #[test]
    fn preselect_is_honoured() {
        let picked = run_bytes(b"\r", Some("windsurf")).expect("picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some("windsurf"));
    }

    #[test]
    fn unknown_preselect_falls_back_to_the_first_row() {
        let picked = run_bytes(b"\r", Some("does-not-exist")).expect("picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some("zed"));
    }

    #[test]
    fn empty_items_is_an_error_and_draws_nothing() {
        let items: &[&'static Editor] = &[];
        let mut out = Vec::new();
        let mut reader: &[u8] = b"\r";
        let result = run_with(items, None, &mut reader, &mut out, false);
        assert!(result.is_err(), "an empty catalog must be an error");
        assert!(out.is_empty(), "no frame should be drawn for an empty list");
    }

    #[test]
    fn a_single_item_list_still_renders_and_enter_or_q_work() {
        // AC6: a catalog with exactly one detected editor must still open the
        // dialog, not be skipped. Enter selects the only row; `q` cancels with
        // `Ok(None)`, and both paths draw a frame marking that row.
        let items: Vec<&'static Editor> = CATALOG.iter().take(1).collect();
        let name = items[0].name;
        let marked = format!("{MARKER}{ARROW}{RESET} {SELECTED}{name}");

        let mut out = Vec::new();
        let mut reader: &[u8] = b"\r";
        let picked = run_with(&items, None, &mut reader, &mut out, false)
            .expect("a single-item picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some(items[0].id));
        let drawn = String::from_utf8(out).expect("the frame is UTF-8");
        assert_one_blank_line_above_and_below(&drawn);
        assert!(
            drawn.contains(&marked),
            "the single row is not marked: {drawn:?}"
        );

        let mut out = Vec::new();
        let mut reader: &[u8] = b"q";
        let picked = run_with(&items, None, &mut reader, &mut out, false)
            .expect("a single-item picker should return");
        assert!(picked.is_none(), "q must cancel with Ok(None)");
        let drawn = String::from_utf8(out).expect("the frame is UTF-8");
        assert_one_blank_line_above_and_below(&drawn);
        assert!(
            drawn.contains(&marked),
            "the single row is not marked: {drawn:?}"
        );
    }

    #[test]
    fn frame_has_one_blank_line_above_and_below() {
        let items = items();
        let drawn = frame(&items, 0);
        assert_one_blank_line_above_and_below(&drawn);

        // One terminator opens the leading blank margin, one closes each row,
        // and one closes the trailing blank margin.
        assert_eq!(
            drawn.matches("\r\n").count(),
            items.len() + 2,
            "the frame must carry one row per editor plus two blank margins: {drawn:?}"
        );
    }

    #[test]
    fn frame_lists_every_editor_name() {
        let items = items();
        let drawn = frame(&items, 0);
        for editor in &items {
            assert!(
                drawn.contains(editor.name),
                "frame is missing {}: {drawn:?}",
                editor.name
            );
        }
        assert_one_blank_line_above_and_below(&drawn);
    }

    #[test]
    fn frame_keeps_catalog_order() {
        let items = items();
        let drawn = frame(&items, 0);
        let mut last = 0;
        for editor in &items {
            let position = match drawn.find(editor.name) {
                Some(position) => position,
                None => panic!("frame is missing {}", editor.name),
            };
            assert!(
                position >= last,
                "{} is out of catalog order: {drawn:?}",
                editor.name
            );
            last = position;
        }
        assert_one_blank_line_above_and_below(&drawn);
    }

    #[test]
    fn frame_marks_the_selected_row() {
        let items = items();
        let drawn = frame(&items, 1);
        assert_one_blank_line_above_and_below(&drawn);
        assert!(
            drawn.contains(&format!(
                "{MARKER}{ARROW}{RESET} {SELECTED}{}",
                items[1].name
            )),
            "selected row is not marked: {drawn:?}"
        );
        assert!(
            drawn.contains(&format!("{UNSELECTED}{}", items[0].name)),
            "unselected row is not plain: {drawn:?}"
        );
        assert_eq!(
            drawn.matches(ARROW).count(),
            1,
            "exactly one row must carry the marker: {drawn:?}"
        );
        assert!(
            !drawn.contains("\u{1b}[7m"),
            "the selected row must not use reverse video: {drawn:?}"
        );
        assert!(
            drawn.contains(HIDE_CURSOR) && drawn.contains(HOME) && drawn.contains(CLEAR_TAIL),
            "frame does not reset the screen: {drawn:?}"
        );
    }
}
