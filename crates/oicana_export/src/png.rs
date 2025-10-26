use typst::layout::{Abs, PagedDocument};

pub use png::EncodingError;

pub fn export_merged_png(
    document: &PagedDocument,
    pixels_per_pt: f32,
) -> Result<Vec<u8>, EncodingError> {
    typst_render::render_merged(document, pixels_per_pt, Abs::pt(15.), None).encode_png()
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
        let result = export_merged_png(&document, 1.0);

        assert!(result.is_ok());
    }

    #[test]
    fn png_output_has_valid_signature() {
        let document = compile(simple_template());
        let png = export_merged_png(&document, 1.0).unwrap();

        assert_eq!(&png[0..8], PNG_SIGNATURE);
    }

    #[test]
    fn exports_with_high_dpi() {
        let document = compile(simple_template());
        let png = export_merged_png(&document, 2.0).unwrap();

        assert!(png.len() > 100);
        assert_eq!(&png[0..8], PNG_SIGNATURE);
    }

    #[test]
    fn exports_with_various_dpi_values() {
        let document = compile(simple_template());

        for dpi in [0.5, 1.0, 1.5, 2.0, 3.0, 4.0] {
            let result = export_merged_png(&document, dpi);
            assert!(result.is_ok());

            let png = result.unwrap();
            assert_eq!(&png[0..8], PNG_SIGNATURE);
        }
    }

    #[test]
    fn exports_multipage_document() {
        let document = compile(multipage_template());
        let png = export_merged_png(&document, 1.0).unwrap();

        assert!(png.len() > 500);
        assert_eq!(&png[0..8], PNG_SIGNATURE);
    }

    #[test]
    fn higher_dpi_produces_larger_output() {
        let document = compile(simple_template());

        let png_1x = export_merged_png(&document, 1.0).unwrap();
        let png_2x = export_merged_png(&document, 2.0).unwrap();

        assert!(png_2x.len() > png_1x.len());
    }

    #[test]
    fn png_export_is_deterministic() {
        let doc1 = compile(simple_template());
        let doc2 = compile(simple_template());

        let png1 = export_merged_png(&doc1, 1.5).unwrap();
        let png2 = export_merged_png(&doc2, 1.5).unwrap();

        assert_eq!(png1, png2);
    }

    #[test]
    fn exports_with_very_low_dpi() {
        let document = compile(simple_template());
        let png = export_merged_png(&document, 0.1).unwrap();

        assert!(png.len() > 50);
        assert_eq!(&png[0..8], PNG_SIGNATURE);
    }

    #[test]
    fn multipage_png_is_larger_than_single_page() {
        let single_doc = compile(simple_template());
        let multi_doc = compile(multipage_template());

        let single_png = export_merged_png(&single_doc, 1.0).unwrap();
        let multi_png = export_merged_png(&multi_doc, 1.0).unwrap();

        assert!(multi_png.len() > single_png.len());
    }
}
