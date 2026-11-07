//! The terminal: raw mode, the alternate screen and the reports an app asks for, held by one
//! [`Screen`] and restored however the app ends.

use std::io::{self, ErrorKind, Stdout, Write};
use std::panic;
use std::sync::Once;
use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{
    DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
    EnableFocusChange, EnableMouseCapture,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Size;
use ratatui::{Frame, Terminal};

/// Whether a [`Screen`] holds the terminal: one terminal serves the process, and the panic hook
/// must know whether to restore it.
static ENGAGED: AtomicBool = AtomicBool::new(false);

/// Installs the panic hook once per process.
static HOOK: Once = Once::new();

/// What the terminal reports beyond keys and resizes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reports {
    /// Mouse buttons, motion and the wheel.
    pub mouse: bool,
    /// Focus gained and lost.
    pub focus: bool,
    /// Pastes, as one event rather than as keystrokes.
    pub paste: bool,
}

/// The terminal in raw mode on the alternate screen, its cursor hidden, until the screen drops.
///
/// Dropping the screen, a panic or a hangup restores the terminal exactly once.
#[derive(Debug)]
pub struct Screen {
    terminal: Terminal<CrosstermBackend<Output>>,
    _engaged: Engaged,
}

impl Screen {
    /// Takes the terminal and enables `reports`.
    ///
    /// # Errors
    ///
    /// The terminal's refusal of raw mode or of a setup write, and [`ErrorKind::ResourceBusy`]
    /// while another screen holds the terminal.
    pub fn open(reports: Reports) -> io::Result<Self> {
        if ENGAGED.swap(true, Ordering::SeqCst) {
            return Err(io::Error::new(
                ErrorKind::ResourceBusy,
                "another screen holds the terminal",
            ));
        }
        if let Err(error) = enable_raw_mode() {
            ENGAGED.store(false, Ordering::SeqCst);
            return Err(error);
        }
        // Own the restore before any further fallible setup.
        let engaged = Engaged;
        HOOK.call_once(|| {
            let previous = panic::take_hook();
            panic::set_hook(Box::new(move |info| {
                release();
                previous(info);
            }));
        });
        let mut output = Output(io::stdout());
        execute!(output, EnterAlternateScreen, Hide)?;
        if reports.mouse {
            execute!(output, EnableMouseCapture)?;
        }
        if reports.focus {
            execute!(output, EnableFocusChange)?;
        }
        if reports.paste {
            execute!(output, EnableBracketedPaste)?;
        }
        Ok(Self {
            terminal: Terminal::new(CrosstermBackend::new(output))?,
            _engaged: engaged,
        })
    }

    /// Draws one frame.
    ///
    /// # Errors
    ///
    /// The terminal's refusal of the frame's writes.
    pub fn draw(&mut self, render: impl FnOnce(&mut Frame<'_>)) -> io::Result<()> {
        self.terminal.draw(render)?;
        Ok(())
    }

    /// The terminal's size, in cells.
    ///
    /// # Errors
    ///
    /// The terminal's refusal to report its size.
    pub fn size(&self) -> io::Result<Size> {
        self.terminal.size()
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        // Release before the terminal field drops, so that its own cursor write is forgiven.
        release();
    }
}

/// The screen's hold on the terminal's modes, which it restores on drop unless a panic did first.
#[derive(Debug)]
struct Engaged;

impl Drop for Engaged {
    fn drop(&mut self) {
        release();
    }
}

/// Restores the terminal, once per engagement: a panic restores it before its report prints, and
/// the unwinding screen must not restore it again, which would move the cursor over the report.
fn release() {
    if ENGAGED.swap(false, Ordering::SeqCst) {
        let _ = execute!(
            io::stdout(),
            DisableBracketedPaste,
            DisableFocusChange,
            DisableMouseCapture,
            Show,
            LeaveAlternateScreen
        );
        let _ = disable_raw_mode();
    }
}

/// Standard output, whose refusals are forgiven once the terminal is released. A hung-up terminal
/// refuses every write, and ratatui's own `Drop` reports a refused cursor write with `eprintln!`,
/// which panics once standard error is gone too.
#[derive(Debug)]
struct Output(Stdout);

impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self.0.write(bytes) {
            Err(_) if !ENGAGED.load(Ordering::SeqCst) => Ok(bytes.len()),
            written => written,
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.0.flush() {
            Err(_) if !ENGAGED.load(Ordering::SeqCst) => Ok(()),
            flushed => flushed,
        }
    }
}
