use oicana_world::diagnostics::TemplateDiagnostics;
use typst::{foundations::Smart, layout::PagedDocument};
use typst_pdf::{PdfOptions, PdfStandard, PdfStandards};

pub fn export_merged_pdf<Diagnostics: TemplateDiagnostics>(
    document: &PagedDocument,
    diagnostics: &Diagnostics,
) -> Result<Vec<u8>, String> {
    let options = PdfOptions {
        ident: Smart::Auto,
        timestamp: None,
        page_ranges: None,
        tagged: true,
        standards: PdfStandards::new(&[PdfStandard::A_3b])
            .expect("Invalid combination of PDF standards"),
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
            "#set page(width: 200pt, height: 100pt)\nHello".to_owned(),
        );
        PreloadedTemplate::new(files)
    }

    fn multipage_template() -> PreloadedTemplate {
        let mut files = HashMap::new();
        files.insert("typst.toml".to_owned(), manifest().to_owned());
        files.insert(
            "main.typ".to_owned(),
            "#set page(width: 200pt, height: 100pt)\nPage 1\n#pagebreak()\nPage 2\n#pagebreak()\nPage 3".to_owned(),
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
        let pdf = export_merged_pdf(&document, &world).expect("PDF export to work");

        assert_eq!(&pdf[0..4], b"%PDF");
        let end = String::from_utf8_lossy(&pdf[pdf.len() - 10..]);
        assert!(end.contains("%%EOF"));
    }

    #[test]
    fn exports_multipage_document_to_pdf() {
        let (document, world) = compile(multipage_template());
        let pdf = export_merged_pdf(&document, &world).unwrap();

        assert!(pdf.len() > 500);
        assert_eq!(&pdf[0..4], b"%PDF");
        let end = String::from_utf8_lossy(&pdf[pdf.len() - 10..]);
        assert!(end.contains("%%EOF"));
    }

    #[test]
    fn pdf_export_is_deterministic() {
        let (doc1, world1) = compile(simple_template());
        let (doc2, world2) = compile(simple_template());

        let pdf1 = export_merged_pdf(&doc1, &world1).unwrap();
        let pdf2 = export_merged_pdf(&doc2, &world2).unwrap();

        assert_eq!(pdf1, pdf2);
    }

    #[test]
    fn multipage_pdf_is_larger_than_single_page() {
        let (single_doc, single_world) = compile(simple_template());
        let (multi_doc, multi_world) = compile(multipage_template());

        let single_pdf = export_merged_pdf(&single_doc, &single_world).unwrap();
        let multi_pdf = export_merged_pdf(&multi_doc, &multi_world).unwrap();

        assert!(multi_pdf.len() > single_pdf.len());
    }
}
