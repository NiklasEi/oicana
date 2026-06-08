use thiserror::Error;
use typst::layout::{Abs, PagedDocument};

pub use png::EncodingError;

use crate::pages::{select_pages, PageRange};

/// An error that occurred while exporting a document to PNG.
#[derive(Debug, Error)]
pub enum PngExportError {
    /// The requested scale (`pixels_per_pt`) is not a positive, finite number.
    #[error("pixels per point must be a positive, finite number, got {0}")]
    InvalidScale(f32),
    /// The requested page range selected none of the document's pages.
    #[error("the requested page range selected no pages of the document")]
    NoPagesSelected,
    /// Encoding the rendered pixmap to PNG failed.
    #[error(transparent)]
    Encoding(#[from] EncodingError),
}

/// Export the document to a single, vertically stacked PNG.
///
/// When `pages` is `None` the whole document is exported; otherwise only the
/// pages in the range are exported.
pub fn export_png(
    document: &PagedDocument,
    pixels_per_pt: f32,
    pages: Option<&PageRange>,
) -> Result<Vec<u8>, PngExportError> {
    if !pixels_per_pt.is_finite() || pixels_per_pt <= 0.0 {
        return Err(PngExportError::InvalidScale(pixels_per_pt));
    }
    let selected = select_pages(document, pages);
    if selected.pages.is_empty() {
        return Err(PngExportError::NoPagesSelected);
    }
    Ok(typst_render::render_merged(&selected, pixels_per_pt, Abs::pt(15.), None).encode_png()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oicana_files::preloaded::PreloadedTemplate;
    use oicana_input::TemplateInputs;
    use oicana_world::manifest::OicanaWorldFiles;
    use oicana_world::world::OicanaWorld;
    use std::collections::HashMap;

    fn simple_template() -> PreloadedTemplate {
        let mut files = HashMap::new();
        files.insert(
            "typst.toml".to_owned(),
            r#"
[package]
name = "test"
version = "0.1.0"
entrypoint = "main.typ"

[tool.oicana]
manifest_version = 1
"#
            .to_owned(),
        );
        files.insert(
            "main.typ".to_owned(),
            "#set page(width: 200pt, height: 100pt)\nHello".to_owned(),
        );
        PreloadedTemplate::new(files)
    }

    fn multipage_template() -> PreloadedTemplate {
        let mut files = HashMap::new();
        files.insert(
            "typst.toml".to_owned(),
            r#"
[package]
name = "test"
version = "0.1.0"
entrypoint = "main.typ"

[tool.oicana]
manifest_version = 1
"#
            .to_owned(),
        );
        files.insert(
            "main.typ".to_owned(),
            "#set page(width: 200pt, height: 100pt)\nPage 1\n#pagebreak()\nPage 2".to_owned(),
        );
        PreloadedTemplate::new(files)
    }

    fn compile(template: PreloadedTemplate) -> PagedDocument {
        let manifest = template.manifest().unwrap();
        let mut world = OicanaWorld::new(template, TemplateInputs::new(), manifest).unwrap();
        world.compile().unwrap().document
    }

    const PNG_SIGNATURE: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10];

    #[test]
    fn exports_simple_document_to_png() {
        let document = compile(simple_template());
        let result = export_png(&document, 1.0, None);

        assert!(result.is_ok());
    }

    #[test]
    fn png_output_has_valid_signature() {
        let document = compile(simple_template());
        let png = export_png(&document, 1.0, None).unwrap();

        assert_eq!(&png[0..8], PNG_SIGNATURE);
    }

    #[test]
    fn exports_with_high_dpi() {
        let document = compile(simple_template());
        let png = export_png(&document, 2.0, None).unwrap();

        assert!(png.len() > 100);
        assert_eq!(&png[0..8], PNG_SIGNATURE);
    }

    #[test]
    fn exports_with_various_dpi_values() {
        let document = compile(simple_template());

        for dpi in [0.5, 1.0, 1.5, 2.0, 3.0, 4.0] {
            let result = export_png(&document, dpi, None);
            assert!(result.is_ok());

            let png = result.unwrap();
            assert_eq!(&png[0..8], PNG_SIGNATURE);
        }
    }

    #[test]
    fn exports_multipage_document() {
        let document = compile(multipage_template());
        let png = export_png(&document, 1.0, None).unwrap();

        assert!(png.len() > 500);
        assert_eq!(&png[0..8], PNG_SIGNATURE);
    }

    #[test]
    fn higher_dpi_produces_larger_output() {
        let document = compile(simple_template());

        let png_1x = export_png(&document, 1.0, None).unwrap();
        let png_2x = export_png(&document, 2.0, None).unwrap();

        assert!(png_2x.len() > png_1x.len());
    }

    #[test]
    fn png_export_is_deterministic() {
        let doc1 = compile(simple_template());
        let doc2 = compile(simple_template());

        let png1 = export_png(&doc1, 1.5, None).unwrap();
        let png2 = export_png(&doc2, 1.5, None).unwrap();

        assert_eq!(png1, png2);
    }

    #[test]
    fn exports_with_very_low_dpi() {
        let document = compile(simple_template());
        let png = export_png(&document, 0.1, None).unwrap();

        assert!(png.len() > 50);
        assert_eq!(&png[0..8], PNG_SIGNATURE);
    }

    #[test]
    fn degenerate_dpi_is_rejected() {
        let document = compile(simple_template());

        for dpi in [
            0.0f32,
            -1.0,
            -0.5,
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
        ] {
            assert!(
                matches!(
                    export_png(&document, dpi, None),
                    Err(PngExportError::InvalidScale(_))
                ),
                "dpi {dpi} should be rejected as an invalid scale"
            );
        }

        assert!(export_png(&document, f32::MIN_POSITIVE, None).is_ok());
        assert!(export_png(&document, 0.1, None).is_ok());
    }

    #[test]
    fn multipage_png_is_larger_than_single_page() {
        let single_doc = compile(simple_template());
        let multi_doc = compile(multipage_template());

        let single_png = export_png(&single_doc, 1.0, None).unwrap();
        let multi_png = export_png(&multi_doc, 1.0, None).unwrap();

        assert!(multi_png.len() > single_png.len());
    }

    #[test]
    fn exports_individual_pages_to_png() {
        let document = compile(multipage_template());
        assert_eq!(document.pages.len(), 2);

        for page in 1..=document.pages.len() {
            let png = export_png(&document, 1.0, Some(&PageRange::single(page))).unwrap();
            assert_eq!(&png[0..8], PNG_SIGNATURE);
        }
    }

    #[test]
    fn single_page_png_is_smaller_than_merged() {
        let document = compile(multipage_template());

        let page = export_png(&document, 1.0, Some(&PageRange::single(1))).unwrap();
        let merged = export_png(&document, 1.0, None).unwrap();

        assert!(page.len() < merged.len());
    }

    #[test]
    fn out_of_bounds_range_is_rejected() {
        let document = compile(multipage_template());

        assert!(matches!(
            export_png(&document, 1.0, Some(&PageRange::single(3))),
            Err(PngExportError::NoPagesSelected)
        ));
    }

    #[test]
    fn single_page_rejects_degenerate_dpi() {
        let document = compile(simple_template());

        assert!(matches!(
            export_png(&document, 0.0, Some(&PageRange::single(1))),
            Err(PngExportError::InvalidScale(_))
        ));
    }
}
