use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

use crate::fonts::FontArgs;
use crate::target::{TargetArgs, TemplateDir};
use crate::watch::FileWatcher;
use anyhow::bail;
use clap::Args;
use console::{style, Emoji};
use indicatif::HumanDuration;
use oicana_testing::execution::{TestRunner, TestRunnerContext};
use oicana_testing::{collect::TemplateTests, Snapshot, SnapshotMode, Test};
use walkdir::WalkDir;

#[derive(Debug, Args)]
pub struct TestArgs {
    #[clap(flatten)]
    target: TargetArgs,
    #[clap(flatten)]
    fonts: FontArgs,
    #[arg(short, long, help = "Update snapshot files and create missing ones")]
    update: bool,
    #[arg(short, long, help = "Watch for file changes and re-run affected tests")]
    watch: bool,
}

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍", "");
static TRUCK: Emoji<'_, '_> = Emoji("🚚", "");
static CLIP: Emoji<'_, '_> = Emoji("🔗", "");
static CHECKMARK: Emoji<'_, '_> = Emoji("✔️", "");
static FIRE: Emoji<'_, '_> = Emoji("🔥", "");
static SPARKLE: Emoji<'_, '_> = Emoji("✨", "");

pub fn test(args: TestArgs) -> anyhow::Result<()> {
    if args.watch {
        return watch_tests(args);
    }

    let ok = style("Ok").green();
    let error = style("Error").red();
    let warning = style("Warning").yellow();

    let test_runner_context = TestRunnerContext::with_fonts(args.fonts.load())?;
    let started = Instant::now();

    println!(
        "{} {}  Collecting templates...",
        style("[1/3]").bold().dim(),
        LOOKING_GLASS
    );
    let mut templates = args.target.get_targets()?;
    templates.sort_by_key(|template| template.manifest.package.name.clone());

    println!(
        "  -> Found {} template{}",
        templates.len(),
        if templates.len() == 1 { "" } else { "s" }
    );

    println!(
        "{} {}  Gathering tests...",
        style("[2/3]").bold().dim(),
        TRUCK
    );
    let tests: Vec<(TemplateDir, TemplateTests)> = templates
        .drain(..)
        .map(|template| {
            let mode = if args.update {
                SnapshotMode::Update
            } else {
                SnapshotMode::Compare
            };
            let tests = template.gather_tests(mode)?;
            Ok((template, tests))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    println!(
        "{} {}  Running {} tests...",
        style("[3/3]").bold().dim(),
        CLIP,
        tests
            .iter()
            .map(|(_, tests)| tests.tests.len())
            .sum::<usize>()
    );
    let mut errors: Vec<(TemplateDir, Vec<TestFailure>)> = vec![];

    for (template_dir, TemplateTests { tests, warnings }) in tests {
        let count = tests.len();
        if count == 0 {
            continue;
        }
        let mut runner =
            test_runner_context.get_runner(&template_dir.path, &template_dir.manifest)?;
        let mut failures = vec![];

        println!("  -> {}", style(&template_dir.manifest.package.name).bold());
        for test_warning in warnings {
            println!("    ↳ {warning}: {test_warning}")
        }

        for test in tests {
            let descriptor = test.descriptor.clone();
            let name = test.name.clone();
            match runner.run(test) {
                Err(test_error) => {
                    println!("  ↳ {name} -> {error}");
                    failures.push(TestFailure {
                        descriptor,
                        failure: test_error.to_string(),
                    });
                }
                Ok(warnings) => {
                    println!("  ↳ {descriptor} -> {ok}");
                    for warning in warnings {
                        println!("    ↳ {warning}");
                    }
                }
            };
        }
        let final_emoji = if failures.is_empty() { CHECKMARK } else { FIRE };
        let ok_count = count - failures.len();
        let mut stats = style(format!("({ok_count}/{count})")).bold();
        stats = if ok_count == count {
            stats.green()
        } else {
            stats.red()
        };

        println!(
            "{final_emoji}  {} {stats}",
            style(&template_dir.manifest.package.name).bold(),
        );
        println!();
        if !failures.is_empty() {
            errors.push((template_dir, failures));
        }
    }

    if !errors.is_empty() {
        println!("{FIRE}  {}", style("Test failures").bold())
    }

    for (template_dir, failures) in &errors {
        println!("  -> {}", style(&template_dir.manifest.package.name).bold());
        for error in failures {
            println!("  ↳ {}", error.descriptor);
            println!("    ↳ {}", error.failure);
        }
    }

    println!(
        "{}  Tests took {}\n",
        SPARKLE,
        HumanDuration(started.elapsed())
    );

    if !errors.is_empty() {
        bail!("Tests failed!")
    }
    anyhow::Ok(())
}

struct TestFailure {
    descriptor: String,
    failure: String,
}

#[rustfmt::skip]
pub const TEST_AFTER_HELP: &str = color_print::cstr!("\
<s><u>Examples:</></>
  oicana test
  oicana test templates/invoice
  oicana test -a
  oicana test templates -a
  oicana test --watch
  oicana test -aw
");

struct WatchedTemplate {
    dir: TemplateDir,
    runner: TestRunner,
    /// The test directory for this template.
    test_dir: PathBuf,
    /// Canonicalized root path of the template, used to match new files.
    root: PathBuf,
}

/// Run tests in watch mode: execute all tests, then watch for changes and
/// re-run only the tests of affected templates.
fn watch_tests(args: TestArgs) -> anyhow::Result<()> {
    let snapshot_mode = if args.update {
        SnapshotMode::Update
    } else {
        SnapshotMode::Compare
    };

    let test_runner_context = TestRunnerContext::with_fonts(args.fonts.load())?;

    println!(
        "{} {}  Collecting templates...",
        style("[1/2]").bold().dim(),
        LOOKING_GLASS
    );
    let mut templates = args.target.get_targets()?;
    templates.sort_by_key(|template| template.manifest.package.name.clone());

    println!(
        "  -> Found {} template{}",
        templates.len(),
        if templates.len() == 1 { "" } else { "s" }
    );

    // Build watched templates with runners.
    let mut watched: Vec<WatchedTemplate> = templates
        .into_iter()
        .map(|dir| {
            let runner = test_runner_context.get_runner(&dir.path, &dir.manifest)?;
            let test_dir = dir.path.join(&dir.manifest.tool.oicana.tests);
            let root = dir.path.canonicalize().unwrap_or_else(|_| dir.path.clone());
            Ok(WatchedTemplate {
                dir,
                runner,
                test_dir,
                root,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    println!("{} {}  Running tests...", style("[2/2]").bold().dim(), CLIP);

    // Initial run of all templates.
    let mut all_indices: Vec<usize> = (0..watched.len()).collect();
    run_tests_for_templates(&mut watched, &all_indices, snapshot_mode);

    // Build the file watcher.
    let ignored = collect_snapshot_paths(&watched, snapshot_mode);
    let mut watcher = FileWatcher::new(ignored)?;

    println!("\n{}  Watching for changes...\n", style("watching").bold());

    loop {
        // Collect dependencies and build path-to-template mapping.
        let mapping = build_watch_mapping(&mut watched);
        watcher.update(mapping.all_paths)?;

        // Wait for changes.
        let changed = watcher.wait()?;

        // Determine affected templates.
        let mut affected: HashSet<usize> = HashSet::new();
        for changed_path in &changed {
            // Canonicalize to match the keys in the mapping.
            let canonical = changed_path
                .canonicalize()
                .unwrap_or_else(|_| changed_path.clone());
            if let Some(indices) = mapping.path_to_templates.get(&canonical) {
                affected.extend(indices);
            } else {
                // Check if this is a new file inside a template's root directory.
                for (idx, wt) in watched.iter().enumerate() {
                    if canonical.starts_with(&wt.root) {
                        affected.insert(idx);
                    }
                }
            }
        }

        // If we still couldn't map any changed paths, a previously missing
        // file may have appeared. Re-run all templates as a safe fallback.
        if affected.is_empty() {
            all_indices = (0..watched.len()).collect();
            affected.extend(&all_indices);
        }

        let mut affected_sorted: Vec<usize> = affected.into_iter().collect();
        affected_sorted.sort();

        let names: Vec<&str> = affected_sorted
            .iter()
            .map(|&i| watched[i].dir.manifest.package.name.as_str())
            .collect();
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        println!(
            "[{timestamp}] Re-running tests for: {}",
            style(names.join(", ")).bold()
        );

        run_tests_for_templates(&mut watched, &affected_sorted, snapshot_mode);

        // Update ignored paths in case new snapshots were created.
        watcher.set_ignored(collect_snapshot_paths(&watched, snapshot_mode));
    }
}

/// Run tests for the specified template indices, printing results.
fn run_tests_for_templates(
    watched: &mut [WatchedTemplate],
    indices: &[usize],
    snapshot_mode: SnapshotMode,
) {
    let ok = style("Ok").green();
    let error = style("Error").red();
    let warning = style("Warning").yellow();
    let started = Instant::now();

    let mut errors: Vec<(String, Vec<TestFailure>)> = vec![];

    for &idx in indices {
        let wt = &mut watched[idx];
        let template_name = wt.dir.manifest.package.name.to_string();

        let TemplateTests { tests, warnings } = match wt.dir.gather_tests(snapshot_mode) {
            Ok(tests) => tests,
            Err(e) => {
                eprintln!(
                    "{} Failed to gather tests for {}: {e}",
                    style("error").red().bold(),
                    template_name
                );
                continue;
            }
        };

        let count = tests.len();
        if count == 0 {
            continue;
        }

        wt.runner.reset();

        let mut failures = vec![];
        println!("  -> {}", style(&template_name).bold());
        for test_warning in warnings {
            println!("    ↳ {warning}: {test_warning}");
        }

        for test in tests {
            let descriptor = test.descriptor.clone();
            let name = test.name.clone();
            match wt.runner.run(test) {
                Err(test_error) => {
                    println!("  ↳ {name} -> {error}");
                    failures.push(TestFailure {
                        descriptor,
                        failure: test_error.to_string(),
                    });
                }
                Ok(test_warnings) => {
                    println!("  ↳ {descriptor} -> {ok}");
                    for w in test_warnings {
                        println!("    ↳ {w}");
                    }
                }
            };
        }

        let final_emoji = if failures.is_empty() { CHECKMARK } else { FIRE };
        let ok_count = count - failures.len();
        let mut stats = style(format!("({ok_count}/{count})")).bold();
        stats = if ok_count == count {
            stats.green()
        } else {
            stats.red()
        };

        println!("{final_emoji}  {} {stats}", style(&template_name).bold(),);
        println!();

        if !failures.is_empty() {
            errors.push((template_name, failures));
        }
    }

    if !errors.is_empty() {
        println!("{FIRE}  {}", style("Test failures").bold());
        for (name, failures) in &errors {
            println!("  -> {}", style(name).bold());
            for failure in failures {
                println!("  ↳ {}", failure.descriptor);
                println!("    ↳ {}", failure.failure);
            }
        }
    }

    println!(
        "{}  Tests took {}\n",
        SPARKLE,
        HumanDuration(started.elapsed())
    );
}

/// Result of building the watch mapping.
struct WatchMapping {
    /// All paths to pass to [FileWatcher::update].
    all_paths: Vec<PathBuf>,
    /// Maps canonical file paths to the template indices that depend on them.
    path_to_templates: HashMap<PathBuf, Vec<usize>>,
}

/// Build the path-to-template mapping and collect all paths to watch.
///
/// All paths are canonicalized so that lookups against the absolute paths
/// reported by the file watcher succeed regardless of how the original
/// paths were constructed (relative, symlinked, etc.).
fn build_watch_mapping(watched: &mut [WatchedTemplate]) -> WatchMapping {
    let mut path_to_templates: HashMap<PathBuf, Vec<usize>> = HashMap::new();
    let mut all_paths: Vec<PathBuf> = Vec::new();

    let add_path = |path: PathBuf,
                    idx: usize,
                    map: &mut HashMap<PathBuf, Vec<usize>>,
                    paths: &mut Vec<PathBuf>| {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
        map.entry(canonical.clone()).or_default().push(idx);
        paths.push(canonical);
    };

    for (idx, wt) in watched.iter().enumerate() {
        // Compilation dependencies (template sources, registry packages, etc.).
        for dep in wt.runner.dependencies() {
            add_path(dep, idx, &mut path_to_templates, &mut all_paths);
        }

        let manifest_path = wt.dir.path.join("typst.toml");
        add_path(manifest_path, idx, &mut path_to_templates, &mut all_paths);

        // Watch the template root directory so that new files (like a
        // newly created test directory) are detected.
        all_paths.push(wt.root.clone());

        // Test directory: watch all files and directories so that new file
        // creation events are reported by notify.
        if wt.test_dir.is_dir() {
            for entry in WalkDir::new(&wt.test_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.into_path();
                let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
                if path.is_file() {
                    add_path(path, idx, &mut path_to_templates, &mut all_paths);
                } else {
                    all_paths.push(canonical);
                }
            }
        }
    }

    WatchMapping {
        all_paths,
        path_to_templates,
    }
}

/// Collect all snapshot file paths to use as ignored paths for the watcher.
fn collect_snapshot_paths(
    watched: &[WatchedTemplate],
    snapshot_mode: SnapshotMode,
) -> HashSet<PathBuf> {
    let mut ignored = HashSet::new();

    for wt in watched {
        let Ok(template_tests) = wt.dir.gather_tests(snapshot_mode) else {
            continue;
        };
        for test in &template_tests.tests {
            collect_test_snapshot_paths(test, &mut ignored);
        }
    }

    ignored
}

/// Add snapshot paths from a test to the ignored set.
///
/// Paths are canonicalized where possible so they match the absolute paths
/// reported by notify. For files that don't exist yet (e.g. .compare.png),
/// we construct the absolute path from the canonicalized parent directory.
fn collect_test_snapshot_paths(test: &Test, ignored: &mut HashSet<PathBuf>) {
    match &test.snapshot {
        Snapshot::Some(path, _) | Snapshot::Missing(path, _) => {
            let canonical_parent = path.parent().and_then(|p| p.canonicalize().ok());

            // Ignore the snapshot file itself.
            let canonical = path.canonicalize().unwrap_or_else(|_| {
                canonical_parent
                    .as_deref()
                    .map(|p| p.join(path.file_name().unwrap_or_default()))
                    .unwrap_or_else(|| path.clone())
            });
            ignored.insert(canonical);

            // Also ignore .compare.png variants that may be written.
            if let Some(stem) = path.file_stem() {
                let mut compare_name = stem.to_os_string();
                compare_name.push(".compare.png");
                let compare_path = canonical_parent
                    .as_deref()
                    .unwrap_or_else(|| path.parent().unwrap_or(path))
                    .join(compare_name);
                ignored.insert(compare_path);
            }
        }
        Snapshot::None => {}
    }
}
