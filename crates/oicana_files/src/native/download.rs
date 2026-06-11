use std::io::Write;

use codespan_reporting::term;
use codespan_reporting::term::termcolor::WriteColor;
use typst_kit::downloader::{Progress, ProgressReporter};

use crate::native::terminal;

/// Prints download progress by writing `downloading {0}` followed by repeatedly
/// updating the last terminal line.
pub struct PrintProgress(pub Option<String>);

impl ProgressReporter for PrintProgress {
    fn start(&mut self, progress: &Progress) {
        if let Some(name) = &self.0 {
            // Print that a package downloading is happening.
            let styles = term::Styles::default();

            let mut out = terminal::out();
            let _ = out.set_color(&styles.header_help);
            let _ = write!(out, "downloading");

            let _ = out.reset();
            let _ = writeln!(out, " {name}");
            let _ = writeln!(out);
        }
        self.update(progress);
    }

    fn update(&mut self, progress: &Progress) {
        if self.0.is_some() {
            let mut out = terminal::out();
            let _ = out.clear_last_line();
            let _ = writeln!(out, "{progress}");
        }
    }

    fn finish(&mut self, progress: &Progress) {
        self.update(progress);
    }
}
