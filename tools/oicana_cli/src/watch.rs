/// Watch mode for oicana templates.
///
/// Part of the file watcher implementation is adapted from the Typst CLI,
/// used under its MIT License.
use std::iter;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use anyhow::Context;
use console::style;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher as _};
use same_file::is_same_file;

use crate::compile::export::{export_image, export_pdf, ExportFormat, ImageExportFormat};
use crate::compile::{build_file_name, build_inputs, CompileArgs, CHECKMARK};
use oicana::Template;
use oicana_files::native::NativeTemplate;

#[rustfmt::skip]
pub const WATCH_AFTER_HELP: &str = color_print::cstr!("\
<s><u>Examples:</></>
  oicana watch
  oicana watch templates/invoice
  oicana watch -j test=inputs/input1.json -j foo=bar.json -b logo=company.png
");

/// Execute a watching compilation command.
pub fn watch(args: CompileArgs) -> anyhow::Result<()> {
    let path = match args.template {
        None => Path::new(".").to_owned(),
        Some(ref template) => Path::new(template).to_owned(),
    };
    let mut template = Template::<NativeTemplate>::init(&path)?;
    let name = template.manifest().package.name.to_string();

    let out_dir = Path::new(&args.out_dir);
    std::fs::create_dir_all(out_dir)?;

    let file_name = build_file_name(&args, &template);
    let out = out_dir.join(&file_name);

    let mut watcher = FileWatcher::new(out.clone())?;

    println!("watching {}", style(&name).bold());
    compile_and_export(&mut template, &args, &out, &name);

    loop {
        watcher.update(template.dependencies())?;
        watcher.wait()?;

        template.reset();
        compile_and_export(&mut template, &args, &out, &name);
    }
}

fn compile_and_export(
    template: &mut Template<NativeTemplate>,
    args: &CompileArgs,
    out: &Path,
    name: &str,
) {
    let inputs = match build_inputs(args) {
        Ok(inputs) => inputs,
        Err(e) => {
            eprintln!("{} Failed to read inputs: {e}", style("error").red().bold());
            return;
        }
    };

    let result = match template.compile(inputs) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    let document = result.document;
    if let Some(warnings) = result.warnings {
        println!("{warnings}");
    }

    let export_result = match args.format {
        ExportFormat::Pdf => export_pdf(&document, out, template, args.pdf_standards.clone()),
        ExportFormat::Png => export_image(&document, out, ImageExportFormat::Png),
        ExportFormat::Svg => export_image(&document, out, ImageExportFormat::Svg),
    };

    match export_result {
        Ok(()) => {
            let timestamp = chrono::Local::now().format("%H:%M:%S");
            println!(
                "{CHECKMARK}  [{timestamp}] {} compiled to {}",
                style(name).bold(),
                style(out.display()).cyan(),
            );
        }
        Err(e) => {
            eprintln!("{} Failed to export: {e}", style("error").red().bold());
        }
    }
}

/// Watches file system activity.
struct FileWatcher {
    /// The output file. We ignore any events for it.
    output: PathBuf,
    /// The underlying watcher.
    watcher: RecommendedWatcher,
    /// Notify event receiver.
    rx: Receiver<notify::Result<Event>>,
    /// Keeps track of which paths are watched. The boolean is used for
    /// mark-and-sweep garbage collection.
    watched: std::collections::HashMap<PathBuf, bool>,
    /// Files that should be watched but don't exist yet.
    missing: std::collections::HashSet<PathBuf>,
}

impl FileWatcher {
    /// How long to wait for a shortly following file system event.
    const BATCH_TIMEOUT: Duration = Duration::from_millis(100);

    /// The maximum time we spend batching events before quitting wait().
    const STARVE_TIMEOUT: Duration = Duration::from_millis(500);

    /// The interval for polling missing files.
    const POLL_INTERVAL: Duration = Duration::from_millis(300);

    /// Create a new file watcher.
    fn new(output: PathBuf) -> anyhow::Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        let config = notify::Config::default().with_poll_interval(Self::POLL_INTERVAL);
        let watcher =
            RecommendedWatcher::new(tx, config).context("failed to setup file watching")?;

        Ok(Self {
            output,
            rx,
            watcher,
            watched: std::collections::HashMap::new(),
            missing: std::collections::HashSet::new(),
        })
    }

    /// Update the watcher to watch exactly the listed files.
    fn update(&mut self, paths: impl IntoIterator<Item = PathBuf>) -> anyhow::Result<()> {
        // Mark all as unseen.
        for seen in self.watched.values_mut() {
            *seen = false;
        }

        self.missing.clear();

        for path in paths {
            if !path.exists() {
                self.missing.insert(path);
                continue;
            }

            if !self.watched.contains_key(&path) {
                self.watcher
                    .watch(&path, RecursiveMode::NonRecursive)
                    .with_context(|| format!("failed to watch {path:?}"))?;
            }

            self.watched.insert(path, true);
        }

        // Unwatch paths no longer needed.
        self.watched.retain(|path, &mut seen| {
            if !seen {
                self.watcher.unwatch(path).ok();
            }
            seen
        });

        Ok(())
    }

    /// Wait until there is a relevant change to a watched path.
    fn wait(&mut self) -> anyhow::Result<()> {
        loop {
            let first = self.rx.recv_timeout(if self.missing.is_empty() {
                Duration::MAX
            } else {
                Self::POLL_INTERVAL
            });

            let mut relevant = false;
            let batch_start = Instant::now();
            for event in first
                .into_iter()
                .chain(iter::from_fn(|| {
                    self.rx.recv_timeout(Self::BATCH_TIMEOUT).ok()
                }))
                .take_while(|_| batch_start.elapsed() <= Self::STARVE_TIMEOUT)
            {
                let event = event.context("file watching error")?;

                if !is_relevant_event_kind(&event.kind) {
                    continue;
                }

                // Workaround for notify-rs' implicit unwatch on remove/rename
                // (triggered by some editors when saving files).
                if matches!(
                    event.kind,
                    notify::EventKind::Remove(notify::event::RemoveKind::File)
                        | notify::EventKind::Modify(notify::event::ModifyKind::Name(
                            notify::event::RenameMode::From
                        ))
                ) {
                    for path in &event.paths {
                        self.watcher.unwatch(path).ok();
                        self.watched.remove(path);
                    }
                }

                // Don't recompile because the output file changed.
                if event
                    .paths
                    .iter()
                    .all(|path| is_same_file(path, &self.output).unwrap_or(false))
                {
                    continue;
                }

                relevant = true;
            }

            if relevant || self.missing.iter().any(|path| path.exists()) {
                return Ok(());
            }
        }
    }
}

/// Whether a kind of watch event is relevant for compilation.
fn is_relevant_event_kind(kind: &notify::EventKind) -> bool {
    match kind {
        notify::EventKind::Any => true,
        notify::EventKind::Access(_) => false,
        notify::EventKind::Create(_) => true,
        notify::EventKind::Modify(kind) => match kind {
            notify::event::ModifyKind::Any => true,
            notify::event::ModifyKind::Data(_) => true,
            notify::event::ModifyKind::Metadata(_) => false,
            notify::event::ModifyKind::Name(_) => true,
            notify::event::ModifyKind::Other => false,
        },
        notify::EventKind::Remove(_) => true,
        notify::EventKind::Other => false,
    }
}
