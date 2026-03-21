use std::time::Instant;

use crate::target::{TargetArgs, TemplateDir};
use anyhow::bail;
use clap::Args;
use console::{style, Emoji};
use indicatif::HumanDuration;
use oicana_testing::{collect::TemplateTests, execution::TestRunnerContext, SnapshotMode};

#[derive(Debug, Args)]
pub struct TestArgs {
    #[clap(flatten)]
    target: TargetArgs,
    #[arg(short, long, help = "Update snapshot files and create missing ones")]
    update: bool,
}

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍", "");
static TRUCK: Emoji<'_, '_> = Emoji("🚚", "");
static CLIP: Emoji<'_, '_> = Emoji("🔗", "");
static CHECKMARK: Emoji<'_, '_> = Emoji("✔️", "");
static FIRE: Emoji<'_, '_> = Emoji("🔥", "");
static SPARKLE: Emoji<'_, '_> = Emoji("✨", "");

pub fn test(args: TestArgs) -> anyhow::Result<()> {
    let ok = style("Ok").green();
    let error = style("Error").red();
    let warning = style("Warning").yellow();

    let test_runner_context = TestRunnerContext::new()?;
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
");
