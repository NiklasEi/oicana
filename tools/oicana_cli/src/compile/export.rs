use anyhow::{bail, Context};
use chrono::Utc;
use clap::ValueEnum;
use oicana::Template;
use oicana_export::pdf::export_merged_pdf;
use oicana_files::native::NativeTemplate;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::fs;
use std::path::Path;
use typst::layout::PagedDocument;

pub fn export_pdf(
    document: &PagedDocument,
    template: &str,
    world: &Template<NativeTemplate>,
) -> anyhow::Result<()> {
    let output = Path::new(".").to_path_buf().join("output").join(format!(
        "{}_{}.pdf",
        template,
        Utc::now().timestamp_millis()
    ));

    let pdf_buffer = match export_merged_pdf(document, world) {
        Ok(pdf_buffer) => pdf_buffer,
        Err(diagnostics) => {
            bail!("Failed to compile PDF\n{diagnostics}");
        }
    };

    fs::create_dir_all(Path::new(".").to_path_buf().join("output"))
        .context("Failed to create the an output directory")?;
    fs::write(output, pdf_buffer).context("Failed to write PDF")?;

    Ok(())
}

/// A format to export in.
#[derive(Debug, Clone, ValueEnum)]
pub enum ExportFormat {
    Pdf,
    Png,
    Svg,
}

pub enum ImageExportFormat {
    Png,
    Svg,
}

/// Export to one or multiple png files.
// Todo: move single page exports to oicana_export
pub fn export_image(
    document: &PagedDocument,
    fmt: ImageExportFormat,
    template: &str,
) -> anyhow::Result<()> {
    let output = Path::new(".").to_path_buf().join("output").join(format!(
        "{template}_{}_{{n}}",
        Utc::now().timestamp_millis()
    ));
    let string = output.to_str().unwrap_or_default();

    // Find a number width that accommodates all pages. For instance, the
    // first page should be numbered "001" if there are between 100 and
    // 999 pages.
    let width = 1 + document.pages.len().checked_ilog10().unwrap_or(0) as usize;

    document
        .pages
        .par_iter()
        .enumerate()
        .map(|(i, page)| {
            let storage = string.replace("{n}", &format!("{:0width$}", i + 1));
            let path = Path::new(&storage);

            match fmt {
                ImageExportFormat::Png => {
                    let pixmap = typst_render::render(page, 144.0 / 72.0);
                    if let Err(encoding_error) = pixmap.save_png(path.with_extension("png")) {
                        bail!("Failed to encode image {encoding_error:?}");
                    }
                }
                ImageExportFormat::Svg => {
                    let svg = typst_svg::svg(page);
                    if let Err(io_error) = fs::write(path.with_extension("svg"), svg.as_bytes()) {
                        bail!("Failed to encode image {io_error:?}");
                    }
                }
            }

            Ok(())
        })
        .collect::<Result<(), anyhow::Error>>()?;

    Ok(())
}
