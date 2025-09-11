use crate::compile::export::{export_image, export_pdf, ExportFormat, ImageExportFormat};
use anyhow::{Context, Ok};
use clap::Args;
use log::{info, warn};
use oicana::Template;
use oicana_files::native::NativeTemplate;
use oicana_input::input::blob::BlobInput;
use oicana_input::input::json::JsonInput;
use oicana_input::{CompilationConfig, TemplateInputs};
use std::fs::{read, read_to_string};
use std::path::Path;

mod export;

#[rustfmt::skip]
pub const COMPILE_AFTER_HELP: &str = color_print::cstr!("\
<s><u>Examples:</></>
  oicana compile
  oicana compile templates/invoice
  oicana compile -j test=inputs/input1.json -j foo=bar.json -b logo=company.png
");

#[derive(Debug, Args)]
pub struct CompileArgs {
    #[arg(
        help = "Path to the template. If not given, the current directory is expected to be a template."
    )]
    template: Option<String>,
    #[arg(short, long, help = "Output format", default_value = "pdf")]
    format: ExportFormat,
    #[clap(
        short,
        long,
        help = "Values for json inputs",
        value_name = "KEY=VALUE",
        num_args = 0..
    )]
    json: Vec<String>,
    #[arg(
        short,
        long,
        help = "Values for blob inputs",
        value_name = "KEY=VALUE",
        num_args = 0..
    )]
    blob: Vec<String>,
    #[arg(short, long, help = "Compile the template in development mode")]
    development: bool,
}

pub fn compile(args: CompileArgs) -> anyhow::Result<()> {
    let inputs = build_inputs(&args)?;

    let path = match args.template {
        None => Path::new("."),
        Some(ref template) => Path::new(template),
    };
    let mut template = Template::<NativeTemplate>::init(path)?;
    let name: String = template.manifest().package.name.to_string();
    info!("Compiling template '{name}'.");

    let result = template.compile(inputs)?;

    let document = result.document;
    if let Some(warnings) = result.warnings {
        println!("{warnings}");
    }

    match args.format {
        ExportFormat::Pdf => export_pdf(&document, &name, &template)?,
        ExportFormat::Png => export_image(&document, ImageExportFormat::Png, &name)?,
        ExportFormat::Svg => export_image(&document, ImageExportFormat::Svg, &name)?,
    }

    Ok(())
}

pub fn build_inputs(args: &CompileArgs) -> anyhow::Result<TemplateInputs> {
    let mut inputs = TemplateInputs::new();
    if !args.development {
        inputs.with_config(CompilationConfig::production());
    }
    for pair in &args.json {
        let parts: Vec<&str> = pair.splitn(2, '=').collect();
        if parts.len() == 2 {
            let input = read_to_string(parts[1]).context("Failed to read json input file")?;
            inputs.with_input(JsonInput::new(parts[0], input));
        } else {
            warn!("Ignoring invalid key-value pair: {pair}");
        }
    }

    for pair in &args.blob {
        let parts: Vec<&str> = pair.splitn(2, '=').collect();
        if parts.len() == 2 {
            let blob = read(parts[1]).context("Failed to read blob input file")?;
            inputs.with_input(BlobInput::new(parts[0], blob));
        } else {
            warn!("Ignoring invalid key-value pair: {pair}");
        }
    }

    Ok(inputs)
}
