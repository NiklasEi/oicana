use anyhow::{bail, Context};
use clap::ValueEnum;
use oicana::Template;
use oicana_export::pdf::export_merged_pdf;
use oicana_export::png::export_merged_png;
use oicana_export::svg::export_merged_svg;
use oicana_files::native::NativeTemplate;
use oicana_template::PdfStandard;
use std::fs;
use std::path::Path;
use typst::layout::PagedDocument;

pub fn export_pdf(
    document: &PagedDocument,
    out: &Path,
    world: &Template<NativeTemplate>,
    cli_standards: Option<Vec<String>>,
) -> anyhow::Result<()> {
    let standards = if let Some(cli_stds) = cli_standards {
        parse_pdf_standards(&cli_stds)?
    } else {
        world.manifest().pdf_standards().to_vec()
    };

    let pdf_buffer = match export_merged_pdf(document, world, &standards) {
        Ok(pdf_buffer) => pdf_buffer,
        Err(diagnostics) => {
            bail!("Failed to compile PDF\n{diagnostics}");
        }
    };

    fs::write(out, pdf_buffer).context("Failed to write PDF")?;

    Ok(())
}

fn parse_pdf_standards(standards: &[String]) -> anyhow::Result<Vec<PdfStandard>> {
    standards
        .iter()
        .map(|s| {
            serde_json::from_value(serde_json::Value::String(s.clone()))
                .with_context(|| format!("Invalid PDF standard: '{}'. Valid values: 1.4, 1.5, 1.6, 1.7, 2.0, a-1b, a-1a, a-2b, a-2u, a-2a, a-3b, a-3u, a-3a, a-4, a-4f, a-4e, ua-1", s))
        })
        .collect()
}

/// A format to export in.
#[derive(Debug, Clone, ValueEnum)]
pub enum ExportFormat {
    Pdf,
    Png,
    Svg,
}

impl ExportFormat {
    pub(crate) fn file_ending(&self) -> &'static str {
        match self {
            ExportFormat::Pdf => "pdf",
            ExportFormat::Png => "png",
            ExportFormat::Svg => "svg",
        }
    }
}

pub enum ImageExportFormat {
    Png,
    Svg,
}

pub fn export_image(
    document: &PagedDocument,
    out: &Path,
    fmt: ImageExportFormat,
) -> anyhow::Result<()> {
    let buffer = match fmt {
        ImageExportFormat::Png => export_merged_png(document, 1.)?,
        ImageExportFormat::Svg => export_merged_svg(document),
    };

    fs::write(out, buffer).context("Failed to write image")?;

    Ok(())
}
