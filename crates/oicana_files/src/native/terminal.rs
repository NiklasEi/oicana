use std::io::{self, Write};

use codespan_reporting::term::termcolor;
use termcolor::{ColorChoice, WriteColor};
use typst::utils::singleton;

/// Returns a handle to the optionally colored terminal output.
pub fn out() -> TermOut {
    TermOut {
        inner: singleton!(TermOutInner, TermOutInner::new()),
    }
}

/// The stuff that has to be shared between instances of [`TermOut`].
struct TermOutInner {
    stream: termcolor::StandardStream,
}

impl TermOutInner {
    fn new() -> Self {
        let stream = termcolor::StandardStream::stderr(ColorChoice::Auto);
        TermOutInner { stream }
    }
}

/// A utility that allows users to write colored terminal output.
/// If colors are not supported by the terminal, they are disabled.
/// This type also allows for deletion of previously written lines.
#[derive(Clone)]
pub struct TermOut {
    inner: &'static TermOutInner,
}

impl TermOut {
    /// Clears the previously written line.
    pub fn clear_last_line(&mut self) -> io::Result<()> {
        // We don't want to clear anything that is not a TTY.
        if self.inner.stream.supports_color() {
            // First, move the cursor up `lines` lines.
            // Then, clear everything between the cursor to end of screen.
            let mut stream = self.inner.stream.lock();
            write!(stream, "\x1B[1F\x1B[0J")?;
            stream.flush()?;
        }
        Ok(())
    }
}

impl Write for TermOut {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.stream.lock().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.stream.lock().flush()
    }
}

impl WriteColor for TermOut {
    fn supports_color(&self) -> bool {
        self.inner.stream.supports_color()
    }

    fn set_color(&mut self, spec: &termcolor::ColorSpec) -> io::Result<()> {
        self.inner.stream.lock().set_color(spec)
    }

    fn reset(&mut self) -> io::Result<()> {
        self.inner.stream.lock().reset()
    }
}
