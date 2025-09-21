use anyhow::{bail, Context};
use clap::ValueEnum;
use oicana::Template;
use oicana_export::pdf::export_merged_pdf;
use oicana_export::png::export_merged_png;
use oicana_export::svg::export_merged_svg;
use oicana_files::native::NativeTemplate;
use std::fs;
use std::path::Path;
use typst::layout::PagedDocument;

pub fn export_pdf(
    document: &PagedDocument,
    out: &Path,
    world: &Template<NativeTemplate>,
) -> anyhow::Result<()> {
    let pdf_buffer = match export_merged_pdf(document, world) {
        Ok(pdf_buffer) => pdf_buffer,
        Err(diagnostics) => {
            bail!("Failed to compile PDF\n{diagnostics}");
        }
    };

    fs::write(out, pdf_buffer).context("Failed to write PDF")?;

    Ok(())
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
