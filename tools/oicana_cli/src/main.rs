//! A command line interface for Oicana.
//!
//! Among other things, this CLI can validate and package Oicana templates.

mod compile;
mod pack;
mod target;
mod test;
mod validate;

use crate::compile::compile;
use crate::pack::{pack, PackArgs};
use crate::validate::{validate, ValidateArgs};
use anyhow::Error;
use clap::Parser;
use clap_verbosity::{Verbosity, WarnLevel};
use compile::{CompileArgs, COMPILE_AFTER_HELP};
use log::trace;
use pack::PACK_AFTER_HELP;
use test::{test, TestArgs, TEST_AFTER_HELP};
use validate::VALIDATE_AFTER_HELP;

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbosity.log_level_filter())
        .init();
    trace!("{cli:?}");

    match cli.command {
        Oicana::Compile(args) => compile(args)?,
        Oicana::Validate(validate_args) => validate(validate_args)?,
        Oicana::Pack(package_args) => pack(package_args)?,
        Oicana::Test(test_args) => test(test_args)?,
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[clap(name = "oicana", version, author)]
#[command(name = "oicana")]
#[command(after_help = AFTER_HELP)]
#[command(about = "PDF templating with Typst", long_about = LONG_ABOUT)]
struct Cli {
    #[command(subcommand)]
    command: Oicana,
    #[command(flatten)]
    verbosity: Verbosity<WarnLevel>,
}

#[derive(Parser, Debug)]
enum Oicana {
    #[clap(about = "Compile oicana templates", after_help = COMPILE_AFTER_HELP)]
    Compile(CompileArgs),
    #[clap(about = "Validate oicana templates", after_help = VALIDATE_AFTER_HELP)]
    Validate(ValidateArgs),
    #[clap(about = "Package oicana templates", after_help = PACK_AFTER_HELP)]
    Pack(PackArgs),
    #[clap(about = "Test oicana templates", after_help = TEST_AFTER_HELP)]
    Test(TestArgs),
}

/// Adds a list of useful links after the normal help text.
#[rustfmt::skip]
const AFTER_HELP: &str = color_print::cstr!("\
<s><u>Resources:</></>
  <s>Website:</>  https://oicana.com/
  <s>Code:</>     https://github.com/oicana/oicana/
");

const LONG_ABOUT: &str = r#"
Oicana is a set of tools and libraries to write document templates
using Typst and create PDFs from these templates out of code.
Oicana templates can define inputs and receive values for
these inputs through several libraries Oicana offers for different
programming languages and platforms.
"#;
