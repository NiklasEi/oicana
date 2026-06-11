use thiserror::Error;
use typst::layout::{Abs, PagedDocument};

use crate::pages::{select_pages, PageRange};

/// An error that occurred while exporting a document to SVG.
#[derive(Debug, Error)]
pub enum SvgExportError {
    /// The requested page range selected none of the document's pages.
    #[error("the requested page range selected no pages of the document")]
    NoPagesSelected,
}

/// Export the document to a single, vertically stacked SVG.
///
/// When `pages` is `None` the whole document is exported; otherwise only the
/// pages in the range are exported.
pub fn export_svg(
    document: &PagedDocument,
    pages: Option<&PageRange>,
) -> Result<Vec<u8>, SvgExportError> {
    let selected = select_pages(document, pages);
    if selected.pages.is_empty() {
        return Err(SvgExportError::NoPagesSelected);
    }
    Ok(typst_svg::svg_merged(&selected, Abs::pt(15.)).into_bytes())
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

    #[test]
    fn exports_simple_document_to_svg() {
        let document = compile(simple_template());
        let svg = export_svg(&document, None).unwrap();

        assert!(svg.len() > 10);
    }

    #[test]
    fn svg_output_contains_svg_tag() {
        let document = compile(simple_template());
        let svg = export_svg(&document, None).unwrap();
        let svg_str = String::from_utf8_lossy(&svg);

        assert!(svg_str.contains("<svg"));
    }

    #[test]
    fn svg_output_is_valid_utf8() {
        let document = compile(simple_template());
        let svg = export_svg(&document, None).unwrap();

        assert!(String::from_utf8(svg).is_ok());
    }

    #[test]
    fn svg_output_has_closing_tag() {
        let document = compile(simple_template());
        let svg = export_svg(&document, None).unwrap();
        let svg_str = String::from_utf8_lossy(&svg);

        assert!(svg_str.contains("</svg>"));
    }

    #[test]
    fn svg_output_has_xmlns() {
        let document = compile(simple_template());
        let svg = export_svg(&document, None).unwrap();
        let svg_str = String::from_utf8_lossy(&svg);

        assert!(svg_str.contains("xmlns"));
    }

    #[test]
    fn exports_multipage_document() {
        let document = compile(multipage_template());
        let svg = export_svg(&document, None).unwrap();

        assert!(svg.len() > 200);
    }

    #[test]
    fn svg_export_is_deterministic() {
        let doc1 = compile(simple_template());
        let doc2 = compile(simple_template());

        let svg1 = export_svg(&doc1, None).unwrap();
        let svg2 = export_svg(&doc2, None).unwrap();

        assert_eq!(svg1, svg2);
    }

    #[test]
    fn multipage_svg_is_larger_than_single_page() {
        let single_doc = compile(simple_template());
        let multi_doc = compile(multipage_template());

        let single_svg = export_svg(&single_doc, None).unwrap();
        let multi_svg = export_svg(&multi_doc, None).unwrap();

        assert!(multi_svg.len() > single_svg.len());
    }

    #[test]
    fn exports_a_single_page() {
        let document = compile(multipage_template());

        let single = export_svg(&document, Some(&PageRange::single(0))).unwrap();
        let merged = export_svg(&document, None).unwrap();
        let single_str = String::from_utf8_lossy(&single);

        assert!(single_str.contains("<svg"));
        assert!(single.len() < merged.len());
    }

    #[test]
    fn out_of_bounds_range_is_rejected() {
        let document = compile(multipage_template());

        assert!(matches!(
            export_svg(&document, Some(&PageRange::single(2))),
            Err(SvgExportError::NoPagesSelected)
        ));
    }
}
