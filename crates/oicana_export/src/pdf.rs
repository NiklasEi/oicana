use oicana_world::diagnostics::TemplateDiagnostics;
use typst::{
    foundations::Smart,
    layout::{PageRanges, PagedDocument},
};
use typst_pdf::{PdfOptions, PdfStandards};

use crate::pages::PageRange;

fn to_typst_standard(standard: oicana_template::PdfStandard) -> typst_pdf::PdfStandard {
    match standard {
        oicana_template::PdfStandard::V_1_4 => typst_pdf::PdfStandard::V_1_4,
        oicana_template::PdfStandard::V_1_5 => typst_pdf::PdfStandard::V_1_5,
        oicana_template::PdfStandard::V_1_6 => typst_pdf::PdfStandard::V_1_6,
        oicana_template::PdfStandard::V_1_7 => typst_pdf::PdfStandard::V_1_7,
        oicana_template::PdfStandard::V_2_0 => typst_pdf::PdfStandard::V_2_0,
        oicana_template::PdfStandard::A_1b => typst_pdf::PdfStandard::A_1b,
        oicana_template::PdfStandard::A_1a => typst_pdf::PdfStandard::A_1a,
        oicana_template::PdfStandard::A_2b => typst_pdf::PdfStandard::A_2b,
        oicana_template::PdfStandard::A_2u => typst_pdf::PdfStandard::A_2u,
        oicana_template::PdfStandard::A_2a => typst_pdf::PdfStandard::A_2a,
        oicana_template::PdfStandard::A_3b => typst_pdf::PdfStandard::A_3b,
        oicana_template::PdfStandard::A_3u => typst_pdf::PdfStandard::A_3u,
        oicana_template::PdfStandard::A_3a => typst_pdf::PdfStandard::A_3a,
        oicana_template::PdfStandard::A_4 => typst_pdf::PdfStandard::A_4,
        oicana_template::PdfStandard::A_4f => typst_pdf::PdfStandard::A_4f,
        oicana_template::PdfStandard::A_4e => typst_pdf::PdfStandard::A_4e,
        oicana_template::PdfStandard::Ua_1 => typst_pdf::PdfStandard::Ua_1,
    }
}

/// Check whether the given list of Oicana PDF standards forms a combination
/// Typst can produce (at most one version, at most one validator, and a
/// version+validator pair must be compatible).
pub fn validate_pdf_standards(standards: &[oicana_template::PdfStandard]) -> Result<(), String> {
    let typst_standards: Vec<_> = standards.iter().map(|s| to_typst_standard(*s)).collect();
    PdfStandards::new(&typst_standards)
        .map(|_| ())
        .map_err(|e| format!("Invalid combination of PDF standards: {}", e))
}

/// Export the document to PDF.
///
/// When `pages` is `None` the whole document is exported; otherwise only the
/// pages in the range are exported.
pub fn export_pdf<Diagnostics: TemplateDiagnostics>(
    document: &PagedDocument,
    diagnostics: &Diagnostics,
    standards: &[oicana_template::PdfStandard],
    pages: Option<&PageRange>,
) -> Result<Vec<u8>, String> {
    let typst_standards: Vec<_> = standards.iter().map(|s| to_typst_standard(*s)).collect();

    // In Typst 0.14 producing a tagged PDF while skipping pages trips an
    // internal assertion. Only disable tagging when pages are actually skipped.
    let skips_pages = pages.is_some_and(|range| {
        range.selected_indices(document.pages.len()).len() != document.pages.len()
    });

    let options = PdfOptions {
        ident: Smart::Auto,
        timestamp: None,
        page_ranges: pages.map(PageRanges::from),
        tagged: !skips_pages,
        standards: PdfStandards::new(&typst_standards)
            .map_err(|e| format!("Invalid combination of PDF standards: {}", e))?,
    };

    typst_pdf::pdf(document, &options).map_err(|source_error| {
        String::from_utf8_lossy(&diagnostics.format_diagnostics(source_error)).into()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use oicana_files::preloaded::PreloadedTemplate;
    use oicana_input::TemplateInputs;
    use oicana_world::manifest::OicanaWorldFiles;
    use oicana_world::world::OicanaWorld;
    use std::collections::HashMap;

    fn manifest() -> &'static str {
        r#"
        [package]
        name = "test"
        version = "0.1.0"
        entrypoint = "main.typ"
        
        [tool.oicana]
        manifest_version = 1
        "#
    }

    fn simple_template() -> PreloadedTemplate {
        let mut files = HashMap::new();
        files.insert("typst.toml".to_owned(), manifest().to_owned());
        files.insert(
            "main.typ".to_owned(),
            "#set page(width: 200pt, height: 100pt)\n#set document(date: datetime(year: 2020,month: 10,day: 4))\nHello".to_owned(),
        );
        PreloadedTemplate::new(files)
    }

    fn multipage_template() -> PreloadedTemplate {
        let mut files = HashMap::new();
        files.insert("typst.toml".to_owned(), manifest().to_owned());
        files.insert(
            "main.typ".to_owned(),
            "#set page(width: 200pt, height: 100pt)\n#set document(date: datetime(year: 2020,month: 10,day: 4))\nPage 1\n#pagebreak()\nPage 2\n#pagebreak()\nPage 3".to_owned(),
        );
        PreloadedTemplate::new(files)
    }

    fn compile(template: PreloadedTemplate) -> (PagedDocument, OicanaWorld<PreloadedTemplate>) {
        let manifest = template.manifest().unwrap();
        let mut world = OicanaWorld::new(template, TemplateInputs::new(), manifest).unwrap();
        let compiled = world.compile().unwrap();
        (compiled.document, world)
    }

    #[test]
    fn exports_simple_document_to_pdf() {
        let (document, world) = compile(simple_template());
        let pdf = export_pdf(
            &document,
            &world,
            &[oicana_template::PdfStandard::A_3b],
            None,
        )
        .expect("PDF export to work");

        assert_eq!(&pdf[0..4], b"%PDF");
        let end = String::from_utf8_lossy(&pdf[pdf.len() - 10..]);
        assert!(end.contains("%%EOF"));
    }

    #[test]
    fn exports_multipage_document_to_pdf() {
        let (document, world) = compile(multipage_template());
        let pdf = export_pdf(
            &document,
            &world,
            &[oicana_template::PdfStandard::A_3b],
            None,
        )
        .unwrap();

        assert!(pdf.len() > 500);
        assert_eq!(&pdf[0..4], b"%PDF");
        let end = String::from_utf8_lossy(&pdf[pdf.len() - 10..]);
        assert!(end.contains("%%EOF"));
    }

    #[test]
    fn pdf_export_is_deterministic() {
        let (doc1, world1) = compile(simple_template());
        let (doc2, world2) = compile(simple_template());

        let pdf1 = export_pdf(&doc1, &world1, &[oicana_template::PdfStandard::A_3b], None).unwrap();
        let pdf2 = export_pdf(&doc2, &world2, &[oicana_template::PdfStandard::A_3b], None).unwrap();

        assert_eq!(pdf1, pdf2);
    }

    #[test]
    fn multipage_pdf_is_larger_than_single_page() {
        let (single_doc, single_world) = compile(simple_template());
        let (multi_doc, multi_world) = compile(multipage_template());

        let single_pdf = export_pdf(
            &single_doc,
            &single_world,
            &[oicana_template::PdfStandard::A_3b],
            None,
        )
        .unwrap();
        let multi_pdf = export_pdf(
            &multi_doc,
            &multi_world,
            &[oicana_template::PdfStandard::A_3b],
            None,
        )
        .unwrap();

        assert!(multi_pdf.len() > single_pdf.len());
    }

    #[test]
    fn exports_a_page_range() {
        let (document, world) = compile(multipage_template());

        let single = export_pdf(
            &document,
            &world,
            &[oicana_template::PdfStandard::A_3b],
            Some(&PageRange::single(0)),
        )
        .unwrap();
        let full = export_pdf(
            &document,
            &world,
            &[oicana_template::PdfStandard::A_3b],
            None,
        )
        .unwrap();

        assert_eq!(&single[0..4], b"%PDF");
        assert!(single.len() < full.len());
    }

    #[test]
    fn exports_with_different_standard() {
        let (document, world) = compile(simple_template());
        let pdf = export_pdf(
            &document,
            &world,
            &[oicana_template::PdfStandard::A_4],
            None,
        )
        .unwrap();

        assert_eq!(&pdf[0..4], b"%PDF");
        let end = String::from_utf8_lossy(&pdf[pdf.len() - 10..]);
        assert!(end.contains("%%EOF"));
    }

    #[test]
    fn rejects_incompatible_standards() {
        let (document, world) = compile(simple_template());
        let result = export_pdf(
            &document,
            &world,
            &[
                oicana_template::PdfStandard::A_4,
                oicana_template::PdfStandard::Ua_1,
            ],
            None,
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Invalid combination of PDF standards"));
    }

    #[test]
    fn validate_pdf_standards_accepts_single_validator() {
        assert!(validate_pdf_standards(&[oicana_template::PdfStandard::A_3b]).is_ok());
    }

    #[test]
    fn validate_pdf_standards_accepts_version_plus_validator() {
        assert!(validate_pdf_standards(&[
            oicana_template::PdfStandard::V_2_0,
            oicana_template::PdfStandard::A_4,
        ])
        .is_ok());
    }

    #[test]
    fn validate_pdf_standards_rejects_two_validators() {
        let err = validate_pdf_standards(&[
            oicana_template::PdfStandard::A_4,
            oicana_template::PdfStandard::Ua_1,
        ])
        .unwrap_err();
        assert!(err.contains("Invalid combination of PDF standards"));
    }

    #[test]
    fn validate_pdf_standards_rejects_two_versions() {
        let err = validate_pdf_standards(&[
            oicana_template::PdfStandard::V_1_7,
            oicana_template::PdfStandard::V_2_0,
        ])
        .unwrap_err();
        assert!(err.contains("Invalid combination of PDF standards"));
    }
}
