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
use crate::theme::Palette;
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

/// The one-column arrow drawn before the selected row's name. Unselected rows
/// keep a plain space in the same column so the names stay aligned.
#[cfg(unix)]
const ARROW: &str = "\u{25b8}";

/// The suffix drawn after the name on the last-used row: the row whose id
/// equals the preselect id handed by the caller.
#[cfg(unix)]
const CHECK: &str = " \u{2713}";

/// Bold SGR applied to the selected row on top of its palette colours.
#[cfg(unix)]
const BOLD: &str = "\x1b[1m";

/// Resets all SGR attributes.
#[cfg(unix)]
const RESET: &str = "\x1b[0m";

/// The help footer text: the three affordances the key loop supports, drawn
/// as one static line that is the last line of every frame.
#[cfg(unix)]
const FOOTER: &str = " \u{2191}\u{2193} select  enter open  esc cancel";

/// The drawable area, in `(rows, cols)`, the picker assumes when `stty size`
/// cannot run or its output cannot be parsed: the standard 80×24 terminal.
#[cfg(unix)]
const DEFAULT_SIZE: (usize, usize) = (24, 80);

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

    // Resolve the theme palette exactly once, before the first frame; the
    // dialog never re-reads the config while it is open.
    let palette = crate::theme::resolve_palette();

    // A tty, entered below with `min 0 time 1`, reports a 0-byte read on a
    // timeout; every other reader reports end of input there. Record which one
    // this run reads from so the key loop can tell a bare Escape from a closed
    // stdin.
    let read_timeout = std::io::stdin().is_terminal();

    let mut raw = RawMode::enter()?;
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let outcome = run_with(
        items,
        preselect,
        &palette,
        &mut stdin,
        &mut stdout,
        read_timeout,
        terminal_size,
    );
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
/// `palette` is resolved once by [`run`] before the first frame. `size`
/// reports the terminal's drawable `(rows, cols)` and is called before every
/// frame, so a resize takes effect from the next redraw; [`run`] passes
/// [`terminal_size`] and tests pass a fixed closure so no frame needs a tty.
#[cfg(unix)]
fn run_with<R: Read, W: Write, S: FnMut() -> (usize, usize)>(
    items: &[&'static Editor],
    preselect: Option<&str>,
    palette: &Palette,
    mut reader: R,
    mut writer: W,
    read_timeout: bool,
    mut size: S,
) -> Result<Option<&'static Editor>, String> {
    if items.is_empty() {
        return Err(NO_EDITORS.to_string());
    }

    let mut selected = initial_selection(items, preselect);
    let mut keys = Keys::new(&mut reader, read_timeout);
    loop {
        // The scroll window follows the selection with the minimal
        // bottom-scroll: rows outside `[scroll, scroll + visible)` are not
        // drawn, and the frame carries no scrollbar or row numbers.
        let (rows, cols) = size();
        let visible = visible_rows(rows);
        let scroll = scroll_to(selected, visible);
        let drawn = frame(
            items,
            selected,
            preselect,
            palette,
            (rows, cols),
            scroll,
            visible,
        );
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

/// The list rows one frame can draw: the drawable height minus the fixed
/// chrome lines (the blank top line, the blank spacer line above the footer,
/// and the help footer itself), at least 1.
#[cfg(unix)]
fn visible_rows(drawable_rows: usize) -> usize {
    drawable_rows.saturating_sub(3).max(1)
}

/// The first row of the scroll window: herdr's minimal bottom-scroll, so the
/// window only moves once the selection passes its bottom edge.
#[cfg(unix)]
fn scroll_to(selected: usize, visible: usize) -> usize {
    selected.saturating_sub(visible.saturating_sub(1))
}

/// The terminal's drawable area as `(rows, cols)`, read by running `stty
/// size` as a child process — the same invocation pattern as the raw-mode
/// `stty` calls, bound to the controlling tty. Called before every frame so a
/// resize applies on the next redraw. Any failure (no `stty`, a non-tty
/// stdin, unparsable output) silently falls back to [`DEFAULT_SIZE`]: an
/// undersized dialog is recoverable, a refused redraw is not.
#[cfg(unix)]
fn terminal_size() -> (usize, usize) {
    stty(&["size"])
        .ok()
        .and_then(|output| parse_stty_size(&output))
        .unwrap_or(DEFAULT_SIZE)
}

/// Parses the `"<rows> <cols>"` output of `stty size`: exactly two
/// whitespace-separated positive integers, or `None` so the caller falls back
/// to the default drawable area.
#[cfg(unix)]
fn parse_stty_size(output: &str) -> Option<(usize, usize)> {
    let mut fields = output.split_whitespace();
    let rows: usize = fields.next()?.parse().ok()?;
    let cols: usize = fields.next()?.parse().ok()?;
    if rows == 0 || cols == 0 || fields.next().is_some() {
        return None;
    }
    Some((rows, cols))
}

/// One blank frame line (top, filler, or spacer): the `panel_bg` background
/// painted across the full width with no foreground. Under the ANSI-16
/// fallback `panel_bg` is `Default` and `bg_sgr()` is empty, so the line
/// emits no SGR at all — not even a reset — and plain spaces reach the
/// terminal.
#[cfg(unix)]
fn blank_line(palette: &Palette, width: usize) -> String {
    let bg = palette.panel_bg.bg_sgr();
    let spaces = " ".repeat(width);
    if bg.is_empty() {
        format!("{spaces}\r\n")
    } else {
        format!("{bg}{spaces}{RESET}\r\n")
    }
}

/// Renders one complete frame as a single string: hide the cursor, home,
/// clear the tail, then exactly `height` lines that always fill the whole
/// drawable area: one blank top line, one styled row per editor inside the
/// visible scroll window (`items[scroll..scroll + visible]`, clamped to the
/// list), blank filler lines and one blank spacer line down to the bottom,
/// and the static help footer as the frame's last line, pinned to the bottom
/// of the drawable area. There is no header, title, or border — herdr draws
/// the popup chrome around this frame.
///
/// `size` is the drawable `(rows, cols)`; `height` is its `rows`. The number
/// of blank filler lines is `height - 3 - drawn` (the top line, the spacer,
/// and the footer are the other three lines) and is clamped at zero: when
/// the caller hands a height too small to hold the drawn rows plus the
/// chrome, the filler and then the spacer drop first and the frame emits
/// whatever fits without negative padding. Through [`run_with`] the budget
/// `visible_rows` keeps `drawn` inside the area, so full frames are the
/// normal case.
///
/// The frame ends immediately after the footer line, without a trailing
/// line break: on a full-height frame the extra newline would scroll the
/// top line off the drawable area.
///
/// The function is pure: everything it draws comes from its arguments, so
/// tests can pin exact SGR bytes without a tty. `last_used_id` is the
/// preselect id from the caller and places the ` ✓` mark independently of
/// `selected`; no reverse video (SGR `7`) is ever emitted, and every styled
/// line is padded with spaces to the full `width` inside its SGR styling so
/// the background always spans the row.
#[cfg(unix)]
fn frame(
    items: &[&'static Editor],
    selected: usize,
    last_used_id: Option<&str>,
    palette: &Palette,
    size: (usize, usize),
    scroll: usize,
    visible: usize,
) -> String {
    let (height, width) = size;
    let blank = blank_line(palette, width);
    let mut out = String::new();
    out.push_str(HIDE_CURSOR);
    out.push_str(HOME);
    out.push_str(CLEAR_TAIL);
    // One blank line above the rows.
    out.push_str(&blank);
    let window_end = scroll.saturating_add(visible).min(items.len());
    let drawn = window_end.saturating_sub(scroll);
    for (index, editor) in items.iter().enumerate().take(window_end).skip(scroll) {
        let marker = if index == selected { ARROW } else { " " };
        let check = if last_used_id == Some(editor.id) {
            CHECK
        } else {
            ""
        };
        let row = format!(" {marker} {}{check}", editor.name);
        // Padding is measured in characters, not bytes: the catalog names are
        // ASCII and the marker/check each occupy one column.
        let padding = " ".repeat(width.saturating_sub(row.chars().count()));
        if index == selected {
            // Bold, foreground = panel_bg on background = accent. A Default
            // token contributes no sequence (fg_sgr/bg_sgr return ""), so the
            // ANSI-fallback selected row is bold on a blue background with the
            // terminal's own foreground.
            out.push_str(BOLD);
            out.push_str(&palette.panel_bg.fg_sgr());
            out.push_str(&palette.accent.bg_sgr());
        } else {
            out.push_str(&palette.text.fg_sgr());
            out.push_str(&palette.panel_bg.bg_sgr());
        }
        out.push_str(&row);
        out.push_str(&padding);
        out.push_str(RESET);
        out.push_str("\r\n");
    }
    // Fill the rest of the drawable area with blank lines: `height - 3 -
    // drawn` filler lines plus the one spacer line above the footer; the
    // saturating subtract drops the fill to zero (spacer last) on a height
    // too small for the chrome, so no line is ever emitted below the footer.
    let fill = height.saturating_sub(drawn + 2);
    for _ in 0..fill {
        out.push_str(&blank);
    }
    // The help footer is the frame's last line, pinned to the drawable
    // bottom: identical text and style in every frame, fg = overlay1 on
    // bg = panel_bg, padded to the full width.
    let footer_padding = " ".repeat(width.saturating_sub(FOOTER.chars().count()));
    out.push_str(&palette.overlay1.fg_sgr());
    out.push_str(&palette.panel_bg.bg_sgr());
    out.push_str(FOOTER);
    out.push_str(&footer_padding);
    out.push_str(RESET);
    // The frame ends immediately after the footer's last byte: no trailing
    // line break. On a full-height frame the extra newline would scroll the
    // terminal one row up and push the blank top line off the drawable area.
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
    use crate::theme::TokenColor;

    /// The drawable width the frame-renderer tests pin.
    const TEST_WIDTH: usize = 40;

    /// The drawable height the frame-renderer tests pin: generous enough
    /// that the four-row fixture list leaves blank filler above the spacer,
    /// so the bottom-pinned geometry shows in every snapshot.
    const TEST_HEIGHT: usize = 10;

    /// The drawable size the loop tests pin: tall enough (visible = 21 rows)
    /// that the test lists always fit, so those frames stay unscrolled.
    const TEST_SIZE: (usize, usize) = (24, TEST_WIDTH);

    /// The first four catalog rows: Android Studio, Cursor, helix,
    /// IntelliJ IDEA.
    fn items() -> Vec<&'static Editor> {
        CATALOG.iter().take(4).collect()
    }

    /// The first eight catalog rows: long enough that a 6-row window must
    /// scroll, for the scroll-window snapshots.
    fn eight_items() -> Vec<&'static Editor> {
        CATALOG.iter().take(8).collect()
    }

    /// A fixed truecolor palette (the built-in table's catppuccin tokens)
    /// injected into the renderer tests so SGR bytes are deterministic
    /// without reading a herdr config.
    fn test_palette() -> Palette {
        Palette {
            accent: TokenColor::Rgb(137, 180, 250),
            panel_bg: TokenColor::Rgb(24, 24, 37),
            text: TokenColor::Rgb(205, 214, 244),
            overlay1: TokenColor::Rgb(127, 132, 156),
        }
    }

    /// Renders a frame whose scroll window covers the whole list, at the
    /// pinned `TEST_HEIGHT` drawable area.
    fn render(items: &[&'static Editor], selected: usize, last_used_id: Option<&str>) -> String {
        frame(
            items,
            selected,
            last_used_id,
            &test_palette(),
            (TEST_HEIGHT, TEST_WIDTH),
            0,
            items.len(),
        )
    }

    /// Strips SGR escape sequences from one frame line; the row and footer
    /// lines the tests inspect contain only terminated `\x1b[...m` sequences.
    fn plain(line: &str) -> String {
        let mut out = String::new();
        let mut rest = line;
        while let Some(start) = rest.find('\x1b') {
            out.push_str(&rest[..start]);
            rest = &rest[start..];
            let end = rest
                .find('m')
                .expect("frame test lines carry only terminated SGR sequences");
            rest = &rest[end + 1..];
        }
        out.push_str(rest);
        out
    }

    /// The frame's last line: the help footer (the frame ends with it,
    /// without a trailing line break).
    fn footer_of(drawn: &str) -> &str {
        let lines: Vec<&str> = drawn.split("\r\n").collect();
        lines.last().copied().unwrap_or_default()
    }

    /// Drives the loop over an in-memory byte slice with no terminal involved.
    /// A slice is not a tty, so a 0-byte read is end of input, not a timeout.
    fn run_bytes(input: &[u8], preselect: Option<&str>) -> Result<Option<&'static Editor>, String> {
        let items = items();
        let mut out = Vec::new();
        let mut reader = input;
        run_with(
            &items,
            preselect,
            &test_palette(),
            &mut reader,
            &mut out,
            false,
            || TEST_SIZE,
        )
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
        run_with(
            &items,
            preselect,
            &test_palette(),
            &mut reader,
            &mut out,
            true,
            || TEST_SIZE,
        )
    }

    /// The frame's lines with the clear-sequence prefix removed: one entry
    /// per drawn line, the footer last. The frame carries no trailing line
    /// break, so splitting `\r\n` yields exactly the drawn lines and a
    /// frame that ends on a break (the old bug) surfaces here as a final
    /// empty piece.
    fn frame_lines(drawn: &str) -> Vec<&str> {
        let prefix = format!("{HIDE_CURSOR}{HOME}{CLEAR_TAIL}");
        assert!(
            drawn.starts_with(&prefix),
            "frame must start with the clear sequences: {drawn:?}"
        );
        let lines: Vec<&str> = drawn[prefix.len()..].split("\r\n").collect();
        assert!(
            lines.last().copied() != Some(""),
            "the frame must end on the footer's last byte, not a trailing \\r\\n: {drawn:?}"
        );
        lines
    }

    /// Asserts the shared frame shape: the clear-sequence prefix, then
    /// exactly `height` lines that fill the drawable area — one blank top
    /// line, the window rows starting directly below it, blank filler and
    /// the spacer line, and the help footer as the last line with nothing
    /// below it. The frame holds exactly `height - 1` line breaks: it ends
    /// on the footer's last byte without a trailing `\r\n`.
    fn assert_frame_shape(drawn: &str, height: usize) {
        let lines = frame_lines(drawn);
        assert_eq!(
            lines.len(),
            height,
            "the frame must fill the drawable area with exactly {height} lines: {drawn:?}"
        );
        assert_eq!(
            drawn.matches("\r\n").count(),
            height - 1,
            "the frame must end on the footer's last byte with exactly {} line breaks: {drawn:?}",
            height - 1
        );
        assert!(
            plain(lines.first().copied().unwrap_or_default())
                .chars()
                .all(|c| c == ' '),
            "the frame must start with one blank top line: {drawn:?}"
        );
        assert!(
            plain(lines.get(1).copied().unwrap_or_default())
                .chars()
                .any(|c| c != ' '),
            "the row window must start directly below the blank top line: {drawn:?}"
        );
        assert!(
            footer_of(drawn).contains("select"),
            "the last line must be the help footer: {drawn:?}"
        );
    }

    // --- Byte-exact ANSI snapshots ---------------------------------------
    //
    // The style constants below are the SGR bytes the fixture palette
    // (`test_palette()`, the catppuccin tokens) must produce on the wire.
    // They are written as literals rather than recomputed through
    // `fg_sgr()`/`bg_sgr()` so the snapshots pin the terminal-visible bytes;
    // `snapshot_fixture_styles_do_not_drift_from_theme` guards them against
    // silent drift from `src/theme.rs`.

    /// Selected-row styling: bold, fg = panel_bg, bg = accent.
    const SELECTED_STYLE: &str = "\x1b[1m\x1b[38;2;24;24;37m\x1b[48;2;137;180;250m";
    /// Unselected-row styling: fg = text, bg = panel_bg.
    const UNSELECTED_STYLE: &str = "\x1b[38;2;205;214;244m\x1b[48;2;24;24;37m";
    /// Footer styling: fg = overlay1, bg = panel_bg.
    const FOOTER_STYLE: &str = "\x1b[38;2;127;132;156m\x1b[48;2;24;24;37m";
    /// Blank-line styling (top, filler, spacer): bg = panel_bg only, no
    /// foreground sequence.
    const BLANK_STYLE: &str = "\x1b[48;2;24;24;37m";

    /// One line: `style`, `text`, spaces painting the rest of `TEST_WIDTH`,
    /// the reset, and the line break — exactly as `frame()` emits it.
    fn painted(style: &str, text: &str) -> String {
        format!(
            "{style}{text}{}{RESET}\r\n",
            " ".repeat(TEST_WIDTH - text.chars().count())
        )
    }

    /// One blank line as `frame()` emits it under the truecolor fixture
    /// palette: the panel background alone across the full width.
    fn snap_blank() -> String {
        format!("{BLANK_STYLE}{}{RESET}\r\n", " ".repeat(TEST_WIDTH))
    }

    fn snap_selected(name: &str, checked: bool) -> String {
        painted(
            SELECTED_STYLE,
            &format!(" {ARROW} {name}{}", if checked { CHECK } else { "" }),
        )
    }

    fn snap_unselected(name: &str, checked: bool) -> String {
        painted(
            UNSELECTED_STYLE,
            &format!("   {name}{}", if checked { CHECK } else { "" }),
        )
    }

    fn snap_footer() -> String {
        painted(FOOTER_STYLE, FOOTER)
    }

    /// A whole expected frame of `height` lines: the clear-sequence prefix,
    /// one blank top line, `rows` row lines, the blank filler and the one
    /// spacer that carry the panel down to the bottom, then the footer as
    /// the last line. The footer is emitted without its line break: the
    /// frame ends on the footer's last byte.
    fn snap_frame(height: usize, rows: usize, body: String) -> String {
        let mut out = format!("{HIDE_CURSOR}{HOME}{CLEAR_TAIL}{}", snap_blank());
        out.push_str(&body);
        for _ in 0..height.saturating_sub(rows + 2) {
            out.push_str(&snap_blank());
        }
        out.push_str(snap_footer().trim_end_matches("\r\n"));
        out
    }

    /// The footer line's bytes with the styling and the reset but no line
    /// break — what `frame()` must end on. `text` is the footer padding
    /// width for the frame under test.
    fn snap_footer_tail(style: &str, padding: usize) -> String {
        format!("{style}{FOOTER}{}{RESET}", " ".repeat(padding))
    }

    #[test]
    fn down_down_enter_picks_the_third_item() {
        let picked = run_bytes(b"\x1b[B\x1b[B\r", None).expect("picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some("helix"));
    }

    #[test]
    fn j_then_enter_picks_the_second_item() {
        let picked = run_bytes(b"j\r", None).expect("picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some("cursor"));
    }

    #[test]
    fn ss3_arrows_are_recognised() {
        let picked = run_bytes(b"\x1bOB\n", None).expect("picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some("cursor"));
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
        assert_eq!(picked.map(|editor| editor.id), Some("cursor"));
    }

    #[test]
    fn incomplete_csi_followed_by_timeout_is_ignored() {
        // A CSI prefix that never completes is ignored rather than cancelled,
        // so the following Enter confirms the first row.
        let picked = run_scripted(&[Some(0x1b), Some(b'['), None, Some(b'\r')], None)
            .expect("picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some("android-studio"));
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
        assert_eq!(picked.map(|editor| editor.id), Some("android-studio"));
    }

    #[test]
    fn unknown_keys_are_ignored() {
        let picked = run_bytes(b"xyz\r", None).expect("picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some("android-studio"));
    }

    #[test]
    fn preselect_is_honoured() {
        let picked = run_bytes(b"\r", Some("idea")).expect("picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some("idea"));
    }

    #[test]
    fn unknown_preselect_falls_back_to_the_first_row() {
        // The catalog's first row is now Android Studio, so the preselect
        // fallback lands there, not on the old menu-first Zed.
        let picked = run_bytes(b"\r", Some("does-not-exist")).expect("picker should return");
        assert_eq!(
            picked.map(|editor| editor.id),
            Some("android-studio"),
            "the fallback marks the first row"
        );
    }

    #[test]
    fn empty_items_is_an_error_and_draws_nothing() {
        let items: &[&'static Editor] = &[];
        let mut out = Vec::new();
        let mut reader: &[u8] = b"\r";
        let result = run_with(
            items,
            None,
            &test_palette(),
            &mut reader,
            &mut out,
            false,
            || TEST_SIZE,
        );
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
        // The loop frames use TEST_SIZE's width; the single row is selected, so
        // it carries the bold panel_bg-on-accent styling around ` ▸ <name>`.
        let palette = test_palette();
        let row = format!(" {ARROW} {name}");
        let marked = format!(
            "{BOLD}{}{}{}{}{RESET}",
            palette.panel_bg.fg_sgr(),
            palette.accent.bg_sgr(),
            row,
            " ".repeat(TEST_SIZE.1 - row.chars().count())
        );

        let mut out = Vec::new();
        let mut reader: &[u8] = b"\r";
        let picked = run_with(
            &items,
            None,
            &test_palette(),
            &mut reader,
            &mut out,
            false,
            || TEST_SIZE,
        )
        .expect("a single-item picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some(items[0].id));
        let drawn = String::from_utf8(out).expect("the frame is UTF-8");
        assert_frame_shape(&drawn, TEST_SIZE.0);
        assert!(
            drawn.contains(&marked),
            "the single row is not marked: {drawn:?}"
        );

        let mut out = Vec::new();
        let mut reader: &[u8] = b"q";
        let picked = run_with(
            &items,
            None,
            &test_palette(),
            &mut reader,
            &mut out,
            false,
            || TEST_SIZE,
        )
        .expect("a single-item picker should return");
        assert!(picked.is_none(), "q must cancel with Ok(None)");
        let drawn = String::from_utf8(out).expect("the frame is UTF-8");
        assert_frame_shape(&drawn, TEST_SIZE.0);
        assert!(
            drawn.contains(&marked),
            "the single row is not marked: {drawn:?}"
        );
    }

    #[test]
    fn frame_has_a_blank_top_line_and_a_footer_last_line() {
        let items = items();
        let drawn = render(&items, 0, None);
        assert_frame_shape(&drawn, TEST_HEIGHT);
    }

    #[test]
    fn frame_lists_every_editor_name() {
        let items = items();
        let drawn = render(&items, 0, None);
        for editor in &items {
            assert!(
                drawn.contains(editor.name),
                "frame is missing {}: {drawn:?}",
                editor.name
            );
        }
        assert_frame_shape(&drawn, TEST_HEIGHT);
    }

    #[test]
    fn frame_keeps_catalog_order() {
        let items = items();
        let drawn = render(&items, 0, None);
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
        assert_frame_shape(&drawn, TEST_HEIGHT);
    }

    #[test]
    fn frame_marks_the_selected_row() {
        let items = items();
        let palette = test_palette();
        let drawn = render(&items, 1, None);
        assert_frame_shape(&drawn, TEST_HEIGHT);
        let selected_row = format!(" {ARROW} {}", items[1].name);
        assert!(
            drawn.contains(&format!(
                "{BOLD}{}{}{}{}{RESET}",
                palette.panel_bg.fg_sgr(),
                palette.accent.bg_sgr(),
                selected_row,
                " ".repeat(TEST_WIDTH - selected_row.chars().count())
            )),
            "the selected row is not bold panel_bg-on-accent and padded: {drawn:?}"
        );
        let unselected_row = format!("   {}", items[0].name);
        assert!(
            drawn.contains(&format!(
                "{}{}{}{}{RESET}",
                palette.text.fg_sgr(),
                palette.panel_bg.bg_sgr(),
                unselected_row,
                " ".repeat(TEST_WIDTH - unselected_row.chars().count())
            )),
            "the unselected row is not text-on-panel_bg and padded: {drawn:?}"
        );
        assert_eq!(
            drawn.matches(ARROW).count(),
            1,
            "exactly one row must carry the marker: {drawn:?}"
        );
        assert!(
            !drawn.contains("\u{1b}[7m"),
            "the frame must not use reverse video: {drawn:?}"
        );
        assert!(
            drawn.contains(HIDE_CURSOR) && drawn.contains(HOME) && drawn.contains(CLEAR_TAIL),
            "frame does not reset the screen: {drawn:?}"
        );
    }

    #[test]
    fn every_row_blank_and_footer_line_is_padded_to_the_full_width() {
        let items = items();
        let drawn = render(&items, 1, Some("cursor"));
        // Every line below the top line — rows, filler, spacer, footer —
        // is painted with spaces out to the full drawable width. The last
        // split piece is the footer itself (the frame carries no trailing
        // break), so a stray `\r\n` would fail the width check as an empty
        // line.
        for line in drawn.split("\r\n").skip(1) {
            assert_eq!(
                plain(line).chars().count(),
                TEST_WIDTH,
                "the line is not painted to the full drawable width: {line:?}"
            );
        }
    }

    #[test]
    fn check_mark_stays_on_the_last_used_row() {
        let items = items();
        // The ✓ marks the "cursor" row while the selection sits elsewhere.
        let drawn = render(&items, 0, Some("cursor"));
        assert_eq!(
            drawn.matches(CHECK).count(),
            1,
            "exactly one row must carry the last-used mark: {drawn:?}"
        );
        assert!(
            drawn.contains(&format!("   Cursor{CHECK}")),
            "the last-used row must end with the mark: {drawn:?}"
        );
        // Moving the selection onto that row keeps one ✓ and adds the ▸.
        // The cursor row sits at index 1 in the alphabetical catalog.
        let moved = render(&items, 1, Some("cursor"));
        assert_eq!(moved.matches(CHECK).count(), 1);
        assert!(moved.contains(&format!(" {ARROW} Cursor{CHECK}")));
        assert_eq!(moved.matches(ARROW).count(), 1);
    }

    #[test]
    fn no_check_mark_without_a_match() {
        let items = items();
        for last_used in [None, Some("does-not-exist")] {
            let drawn = render(&items, 0, last_used);
            assert!(
                !drawn.contains(CHECK),
                "no row may carry the mark when no id matches: {drawn:?}"
            );
        }
    }

    #[test]
    fn footer_is_styled_and_identical_in_every_frame() {
        let items = items();
        let palette = test_palette();
        let first = render(&items, 0, None);
        let later = render(&items, items.len() - 1, Some("cursor"));
        let footer = footer_of(&first);
        assert_eq!(footer, footer_of(&later), "the footer must be static");
        for affordance in ["\u{2191}\u{2193} select", "enter open", "esc cancel"] {
            assert!(
                footer.contains(affordance),
                "the footer must name {affordance:?}: {footer:?}"
            );
        }
        assert!(
            footer.contains(&format!(
                "{}{}",
                palette.overlay1.fg_sgr(),
                palette.panel_bg.bg_sgr()
            )),
            "the footer is not overlay1-on-panel_bg: {footer:?}"
        );
        assert!(
            footer.ends_with(RESET),
            "the footer styling must be closed: {footer:?}"
        );
    }

    #[test]
    fn snapshot_fixture_styles_do_not_drift_from_theme() {
        // The literal SGR bytes the snapshots pin must stay identical to
        // what the `test_palette()` fixture (the catppuccin row of the
        // built-in table in `src/theme.rs`) makes the token helpers emit.
        let palette = test_palette();
        assert_eq!(
            format!(
                "{BOLD}{}{}",
                palette.panel_bg.fg_sgr(),
                palette.accent.bg_sgr()
            ),
            SELECTED_STYLE
        );
        assert_eq!(
            format!("{}{}", palette.text.fg_sgr(), palette.panel_bg.bg_sgr()),
            UNSELECTED_STYLE
        );
        assert_eq!(
            format!("{}{}", palette.overlay1.fg_sgr(), palette.panel_bg.bg_sgr()),
            FOOTER_STYLE
        );
        assert_eq!(palette.panel_bg.bg_sgr(), BLANK_STYLE);
    }

    #[test]
    fn snapshot_multi_row_frame_is_byte_exact() {
        // Scenario "Multi-row frame": the clear sequences, one blank top
        // line, four padded rows in list order with ▸ and bold
        // panel_bg-on-accent on the selected row only, the filler and one
        // spacer blank line carrying the panel down to the bottom, and the
        // footer as the terminal's last line — the whole frame pinned byte
        // for byte.
        let items = items();
        let drawn = render(&items, 1, None);
        assert_eq!(
            drawn,
            snap_frame(
                TEST_HEIGHT,
                4,
                snap_unselected("Android Studio", false)
                    + &snap_selected("Cursor", false)
                    + &snap_unselected("helix", false)
                    + &snap_unselected("IntelliJ IDEA", false)
            )
        );
    }

    #[test]
    fn snapshot_footer_and_frame_boundaries_are_byte_exact() {
        // Scenario "Footer present and styled" plus the frame boundaries:
        // a one-row frame of a `TEST_HEIGHT`-line area is exactly the
        // clear-sequence prefix, one blank top line, the styled row, the
        // filler and spacer blanks down to the bottom, and the footer line
        // — nothing else.
        let items: Vec<&'static Editor> = CATALOG.iter().take(1).collect();
        let drawn = frame(
            &items,
            0,
            None,
            &test_palette(),
            (TEST_HEIGHT, TEST_WIDTH),
            0,
            1,
        );
        // The footer line's own bytes, without the row helpers: the
        // overlay1-on-panel_bg styling, the static text, the paint to the
        // 40-column width, and the reset — the frame's last line, pinned to
        // the bottom with one blank spacer directly above it, and the
        // frame's final byte is the last byte of that reset.
        assert_eq!(FOOTER.chars().count(), 34);
        let lines = frame_lines(&drawn);
        assert_eq!(lines.len(), TEST_HEIGHT);
        assert_eq!(
            lines[lines.len() - 2],
            snap_blank().trim_end_matches("\r\n")
        );
        assert_eq!(
            drawn,
            snap_frame(TEST_HEIGHT, 1, snap_selected("Android Studio", false))
        );
        assert!(
            drawn.ends_with(&snap_footer_tail(FOOTER_STYLE, 6)),
            "the footer must be the styled, fully painted last line ending \
             on its own last byte: {drawn:?}"
        );
        assert!(
            !drawn.ends_with("\r\n"),
            "the frame must not end with a trailing line break: {drawn:?}"
        );
    }

    #[test]
    fn snapshot_check_mark_placement_is_byte_exact() {
        // Scenario "Check mark on the last-used row": ` ✓` is pinned on the
        // last-used row while the ▸ sits elsewhere, and stays byte-identical
        // (aside from gaining the ▸ style) once the selection arrives.
        let items = items();
        let away = render(&items, 0, Some("cursor"));
        assert_eq!(
            away,
            snap_frame(
                TEST_HEIGHT,
                4,
                snap_selected("Android Studio", false)
                    + &snap_unselected("Cursor", true)
                    + &snap_unselected("helix", false)
                    + &snap_unselected("IntelliJ IDEA", false)
            )
        );
        let arrived = render(&items, 1, Some("cursor"));
        assert_eq!(
            arrived,
            snap_frame(
                TEST_HEIGHT,
                4,
                snap_unselected("Android Studio", false)
                    + &snap_selected("Cursor", true)
                    + &snap_unselected("helix", false)
                    + &snap_unselected("IntelliJ IDEA", false)
            )
        );
    }

    #[test]
    fn snapshot_unknown_preselect_matches_the_unmarked_frame() {
        // Scenario "No check mark without a match": with no preselect or an
        // unknown id the frame is byte-identical to the plain one — the
        // `snapshot_multi_row_frame_is_byte_exact` expectation.
        let items = items();
        let plain = render(&items, 1, None);
        assert_eq!(plain, render(&items, 1, Some("does-not-exist")));
        assert_eq!(
            plain,
            snap_frame(
                TEST_HEIGHT,
                4,
                snap_unselected("Android Studio", false)
                    + &snap_selected("Cursor", false)
                    + &snap_unselected("helix", false)
                    + &snap_unselected("IntelliJ IDEA", false)
            )
        );
    }

    #[test]
    fn snapshot_fallback_palette_is_byte_exact() {
        // Scenario "Fallback palette rendering" (and "Terminal or unknown
        // name falls back"): the selected row is bold on the ANSI blue bg
        // with NO foreground sequence, unselected rows emit no fg or bg SGR
        // at all, and the footer carries only the bright-black `90` fg.
        // Scenario "Blanks painted as panel", AND-clause: under the ANSI-16
        // fallback the blank lines emit no SGR at all — not even a reset —
        // so they are plain padded spaces on the terminal's own background.
        let items = items();
        let palette = crate::theme::fallback_palette();
        let drawn = frame(
            &items,
            1,
            None,
            &palette,
            (TEST_HEIGHT, TEST_WIDTH),
            0,
            items.len(),
        );
        // A blank line under the fallback: just the width's worth of
        // spaces, no escape bytes anywhere in it.
        let blank = format!("{}\r\n", " ".repeat(TEST_WIDTH));
        // One line per row: text, the spaces painting it out to the full
        // TEST_WIDTH, the reset. Column counts: `   Android Studio` 17 → 23,
        // ` ▸ Cursor` 9 → 31, `   helix` 8 → 32, `   IntelliJ IDEA` 16 → 24,
        // footer 34 → 6.
        let unselected_android_studio = format!("   Android Studio{}{RESET}\r\n", " ".repeat(23));
        let selected_cursor = format!(
            "{BOLD}\x1b[44m \u{25b8} Cursor{}{RESET}\r\n",
            " ".repeat(31)
        );
        let unselected_helix = format!("   helix{}{RESET}\r\n", " ".repeat(32));
        let unselected_idea = format!("   IntelliJ IDEA{}{RESET}\r\n", " ".repeat(24));
        // The footer is the frame's last line and carries no trailing break.
        let footer_line = format!("\x1b[90m{FOOTER}{}{RESET}", " ".repeat(6));
        // TEST_HEIGHT (10) lines: top, 4 rows, 3 filler + 1 spacer, footer.
        let expected = format!(
            "{HIDE_CURSOR}{HOME}{CLEAR_TAIL}{blank}{unselected_android_studio}\
             {selected_cursor}{unselected_helix}{unselected_idea}{}{footer_line}",
            blank.repeat(TEST_HEIGHT - 4 - 2)
        );
        assert_eq!(drawn, expected);
        let lines = frame_lines(&drawn);
        assert_eq!(lines.len(), TEST_HEIGHT);
        for blank_line in [&lines[0], &lines[5], &lines[lines.len() - 2]] {
            assert!(
                !blank_line.contains('\x1b'),
                "fallback blanks must emit no SGR at all: {blank_line:?}"
            );
        }
    }

    #[test]
    fn snapshot_footer_pinned_to_the_bottom_is_byte_exact() {
        // Scenario "Footer pinned to the bottom": 3 drawn rows in a
        // 10-line drawable area (below the 7-row visible budget, so the
        // list does not reach the bottom) leave `10 - 3 - 3 = 4` blank
        // filler lines between the last drawn row and the one blank
        // spacer, and the help footer is the frame's final line at the
        // bottom of the area — 10 lines in total, exactly 9 `\r\n`
        // breaks, and the frame's final byte is the last character of
        // the footer line with no trailing `\r\n`.
        let items = items();
        let height = TEST_HEIGHT;
        let drawn = frame(&items, 0, None, &test_palette(), (height, TEST_WIDTH), 0, 3);
        assert_eq!(
            drawn,
            snap_frame(
                height,
                3,
                snap_selected("Android Studio", false)
                    + &snap_unselected("Cursor", false)
                    + &snap_unselected("helix", false)
            )
        );
        // Spell the geometry out line by line: top, 3 rows, 4 filler + 1
        // spacer, footer last.
        let lines = frame_lines(&drawn);
        assert_eq!(lines.len(), height, "the frame is exactly `height` lines");
        assert_eq!(
            drawn.matches("\r\n").count(),
            height - 1,
            "a `height`-line frame carries `height - 1` line breaks"
        );
        assert!(
            drawn.ends_with(&snap_footer_tail(
                FOOTER_STYLE,
                TEST_WIDTH - FOOTER.chars().count()
            )),
            "the frame's final byte is the last character of the footer line: {drawn:?}"
        );
        assert!(
            !drawn.ends_with("\r\n"),
            "the frame must not end with a trailing line break: {drawn:?}"
        );
        let spacer_above_footer = format!("{BLANK_STYLE}{}{RESET}", " ".repeat(TEST_WIDTH));
        assert_eq!(lines[height - 2], spacer_above_footer);
        assert!(lines[height - 1].contains("select"));
        for line in &lines[4..height - 2] {
            assert_eq!(
                *line, spacer_above_footer,
                "every filler line is a panel blank"
            );
        }
    }

    #[test]
    fn snapshot_blanks_painted_as_panel_is_byte_exact() {
        // Scenario "Blanks painted as panel": under the truecolor fixture
        // palette every blank line — the top line, the filler lines, and
        // the spacer above the footer — carries exactly the `panel_bg`
        // background SGR plus the width's worth of padding spaces and the
        // reset, and no foreground sequence anywhere, so the popup reads
        // as one continuous panel.
        let items = items();
        let drawn = render(&items, 1, None);
        let blank = format!("{BLANK_STYLE}{}{RESET}", " ".repeat(TEST_WIDTH));
        let lines = frame_lines(&drawn);
        assert_eq!(lines.len(), TEST_HEIGHT);
        // Top line (0), filler and spacer (5..len-1); rows (1..5) and the
        // footer (last) are styled by their own snapshots.
        let blank_indexes = [0usize, 5, 6, 7, TEST_HEIGHT - 2];
        for index in blank_indexes {
            assert_eq!(
                lines[index], blank,
                "blank line {index} must be panel_bg with no foreground: {:?}",
                lines[index]
            );
            assert!(
                !lines[index].contains("38;"),
                "blank lines must carry no foreground SGR: {:?}",
                lines[index]
            );
        }
    }

    #[test]
    fn invariant_exactly_one_arrow_and_no_reverse_video() {
        // Scenario "No reverse video" and the frame-rendering invariant,
        // swept over every selection index, both marks states, and both
        // the truecolor and the fallback palette.
        let items = items();
        let visible = visible_rows(8);
        for palette in [test_palette(), crate::theme::fallback_palette()] {
            for selected in 0..items.len() {
                for last_used in [None, Some("cursor")] {
                    let drawn = frame(
                        &items,
                        selected,
                        last_used,
                        &palette,
                        (8, TEST_WIDTH),
                        scroll_to(selected, visible),
                        visible,
                    );
                    assert_eq!(
                        drawn.matches(ARROW).count(),
                        1,
                        "exactly one row must carry the marker: {drawn:?}"
                    );
                    assert!(
                        !drawn.contains("\u{1b}[7m"),
                        "the frame must not use reverse video: {drawn:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn scroll_snapshot_downward_selection_scrolls_the_window_by_one() {
        // Scenario "Minimal bottom-scroll on downward selection": N=8,
        // visible=6 (a 9-row area budgets 9 - 3 = 6), selection at index 6
        // renders rows 1..=6 with the selected row at the window bottom;
        // the full window leaves only the spacer between the rows and the
        // footer.
        let items = eight_items();
        let visible = 6;
        let height = 9;
        let scroll = scroll_to(6, visible);
        assert_eq!(scroll, 1);
        let drawn = frame(
            &items,
            6,
            None,
            &test_palette(),
            (height, TEST_WIDTH),
            scroll,
            visible,
        );
        assert_eq!(
            drawn,
            snap_frame(
                height,
                6,
                snap_unselected("Cursor", false)
                    + &snap_unselected("helix", false)
                    + &snap_unselected("IntelliJ IDEA", false)
                    + &snap_unselected("nvim", false)
                    + &snap_unselected("PyCharm", false)
                    + &snap_selected("vim", false)
            )
        );
    }

    #[test]
    fn scroll_snapshot_selection_back_to_the_first_row_resets_the_window() {
        // Scenario "Scrolling back up": selection back at 0 renders rows
        // 0..=5.
        let items = eight_items();
        let visible = 6;
        let height = 9;
        let scroll = scroll_to(0, visible);
        assert_eq!(scroll, 0);
        let drawn = frame(
            &items,
            0,
            None,
            &test_palette(),
            (height, TEST_WIDTH),
            scroll,
            visible,
        );
        assert_eq!(
            drawn,
            snap_frame(
                height,
                6,
                snap_selected("Android Studio", false)
                    + &snap_unselected("Cursor", false)
                    + &snap_unselected("helix", false)
                    + &snap_unselected("IntelliJ IDEA", false)
                    + &snap_unselected("nvim", false)
                    + &snap_unselected("PyCharm", false)
            )
        );
    }

    #[test]
    fn scroll_snapshot_last_index_clamps_the_window_to_the_list_tail() {
        // The tail of "Minimal bottom-scroll on downward selection": N=8,
        // visible=6, selection at index 7 renders rows 2..=7.
        let items = eight_items();
        let visible = 6;
        let height = 9;
        let scroll = scroll_to(7, visible);
        assert_eq!(scroll, 2);
        let drawn = frame(
            &items,
            7,
            None,
            &test_palette(),
            (height, TEST_WIDTH),
            scroll,
            visible,
        );
        assert_eq!(
            drawn,
            snap_frame(
                height,
                6,
                snap_unselected("helix", false)
                    + &snap_unselected("IntelliJ IDEA", false)
                    + &snap_unselected("nvim", false)
                    + &snap_unselected("PyCharm", false)
                    + &snap_unselected("vim", false)
                    + &snap_selected("VS Code", false)
            )
        );
    }

    #[test]
    fn scroll_snapshot_fitting_list_stays_static() {
        // Scenario "List fits — static frame": six rows under a 10-row
        // area's 7-row budget draw all six even with the selection at the
        // last index, with blank filler and the spacer padding the frame
        // down to the footer on the last line.
        let items: Vec<&'static Editor> = CATALOG.iter().take(6).collect();
        let visible = 6;
        let height = 10;
        let scroll = scroll_to(5, visible);
        assert_eq!(scroll, 0);
        let drawn = frame(
            &items,
            5,
            None,
            &test_palette(),
            (height, TEST_WIDTH),
            scroll,
            visible,
        );
        assert_eq!(
            drawn,
            snap_frame(
                height,
                6,
                snap_unselected("Android Studio", false)
                    + &snap_unselected("Cursor", false)
                    + &snap_unselected("helix", false)
                    + &snap_unselected("IntelliJ IDEA", false)
                    + &snap_unselected("nvim", false)
                    + &snap_selected("PyCharm", false)
            )
        );
    }

    #[test]
    fn frame_draws_only_the_scroll_window() {
        let items = items();
        // The loop derives the window with `visible_rows` and `scroll_to`;
        // frame() draws only the rows it is given and pads the rest of the
        // 8-line area with blanks.
        let drawn = frame(
            &items,
            3,
            Some("android-studio"),
            &test_palette(),
            (8, TEST_WIDTH),
            2,
            2,
        );
        assert_frame_shape(&drawn, 8);
        assert!(
            drawn.contains(items[2].name) && drawn.contains(items[3].name),
            "the window rows must be drawn: {drawn:?}"
        );
        assert!(
            !drawn.contains(items[0].name) && !drawn.contains(items[1].name),
            "rows outside the window must not be drawn: {drawn:?}"
        );
        assert_eq!(drawn.matches(ARROW).count(), 1);
        assert!(!drawn.contains("\u{1b}[7m"));
    }

    #[test]
    fn parse_stty_size_accepts_a_rows_and_cols_pair() {
        assert_eq!(parse_stty_size("24 80\n"), Some((24, 80)));
        assert_eq!(parse_stty_size(" 12   40 "), Some((12, 40)));
    }

    #[test]
    fn parse_stty_size_rejects_unparsable_output() {
        // Scenario "Size read failure uses defaults": anything that is not
        // exactly two positive integers becomes `None` and the caller falls
        // back to the 80×24 default.
        for output in [
            "", "  ", "24", "x y", "24 x", "24 80 99", "0 0", "24 0", "-1 80",
        ] {
            assert_eq!(parse_stty_size(output), None, "{output:?} must not parse");
        }
    }

    #[test]
    fn visible_budget_subtracts_the_chrome_and_floors_at_one() {
        // The chrome is three lines: the blank top line, the blank spacer
        // above the footer, and the footer itself.
        assert_eq!(visible_rows(24), 21);
        assert_eq!(visible_rows(3), 1);
        assert_eq!(visible_rows(2), 1);
        assert_eq!(visible_rows(1), 1);
        assert_eq!(visible_rows(0), 1);
    }

    #[test]
    fn popup_height_16_frames_the_full_catalog_with_no_filler() {
        // Re-confirms the checked 4.2 arithmetic against the revised
        // spacer-inclusive geometry: the 16-row popup leaves a 14-row
        // drawable area after herdr's two border lines, and
        // `14 = 1 blank top + 11 catalog rows + 1 blank spacer + 1 footer`
        // — the full 11-entry catalog fills the list area exactly, with
        // zero filler and the footer on the bottom line.
        let catalog: Vec<&'static Editor> = CATALOG.iter().collect();
        assert_eq!(catalog.len(), 11);
        let height = 14;
        let visible = visible_rows(height);
        assert_eq!(visible, 11);
        let scroll = scroll_to(catalog.len() - 1, visible);
        assert_eq!(scroll, 0, "the full catalog fits: no scroll");
        let drawn = frame(
            &catalog,
            catalog.len() - 1,
            None,
            &test_palette(),
            (height, TEST_WIDTH),
            scroll,
            visible,
        );
        assert_frame_shape(&drawn, height);
        let lines = frame_lines(&drawn);
        let blank = format!("{BLANK_STYLE}{}{RESET}", " ".repeat(TEST_WIDTH));
        // Exactly one blank between the last row and the footer: the
        // spacer, no filler lines anywhere in the frame.
        assert_eq!(lines[height - 2], blank);
        for editor in &catalog {
            assert!(
                drawn.contains(editor.name),
                "{} missing: {drawn:?}",
                editor.name
            );
        }
        assert!(
            !lines[..height - 2]
                .iter()
                .skip(1)
                .any(|line| *line == blank.as_str()),
            "the 11 rows leave no room for filler at height 14"
        );
    }

    #[test]
    fn the_scroll_window_follows_the_selection_minimally() {
        let items = items();
        // A 5-row drawable area leaves a 2-row visible budget.
        let height = 5;
        let visible = visible_rows(height);
        assert_eq!(visible, 2);
        // Scenario "List fits — static frame": rows 0 and 1 with the
        // selection inside them keep the window at the top.
        let scroll = scroll_to(1, visible);
        assert_eq!(scroll, 0);
        let drawn = frame(
            &items,
            1,
            None,
            &test_palette(),
            (height, TEST_WIDTH),
            scroll,
            visible,
        );
        assert_frame_shape(&drawn, height);
        assert!(drawn.contains(items[0].name) && drawn.contains(items[1].name));
        assert!(
            !drawn.contains(items[2].name) && !drawn.contains(items[3].name),
            "rows below the window must not be drawn: {drawn:?}"
        );
        // Scenario "Minimal bottom-scroll on downward selection": selection
        // moves to index 2 → rows 1..=2 only, selected at the window bottom.
        let scroll = scroll_to(2, visible);
        assert_eq!(scroll, 1);
        let drawn = frame(
            &items,
            2,
            None,
            &test_palette(),
            (height, TEST_WIDTH),
            scroll,
            visible,
        );
        assert!(
            !drawn.contains(items[0].name),
            "the scrolled-away row must not be drawn: {drawn:?}"
        );
        assert!(drawn.contains(items[1].name) && drawn.contains(items[2].name));
        // Scenario "Scrolling back up": selection to the first row → window
        // clamps back to rows 0..2.
        let scroll = scroll_to(0, visible);
        assert_eq!(scroll, 0);
        let drawn = frame(
            &items,
            0,
            None,
            &test_palette(),
            (height, TEST_WIDTH),
            scroll,
            visible,
        );
        assert!(drawn.contains(items[0].name) && drawn.contains(items[1].name));
        assert!(
            !drawn.contains(items[2].name),
            "rows above the window must not be drawn after scrolling back: {drawn:?}"
        );
    }

    #[test]
    fn the_size_is_re_read_before_every_frame() {
        // A resize applies on the next redraw: the loop reads the size once
        // per drawn frame. Down then Enter draws two frames, so two reads.
        let items = items();
        let mut out = Vec::new();
        let mut reader: &[u8] = b"\x1b[B\r";
        let mut reads = 0;
        let picked = run_with(
            &items,
            None,
            &test_palette(),
            &mut reader,
            &mut out,
            false,
            || {
                reads += 1;
                TEST_SIZE
            },
        )
        .expect("picker should return");
        assert_eq!(picked.map(|editor| editor.id), Some("cursor"));
        assert_eq!(reads, 2, "exactly one size read per drawn frame");
    }
}
