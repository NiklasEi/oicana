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
        standards: PdfStandards::new(&[PdfStandard::A_3b])
            .expect("Invalid combination of PDF standards"),
    };

    typst_pdf::pdf(document, &options).map_err(|source_error| {
        String::from_utf8_lossy(&diagnostics.format_diagnostics(source_error)).into()
    })
}
