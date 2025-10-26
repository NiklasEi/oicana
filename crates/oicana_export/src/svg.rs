use typst::layout::{Abs, PagedDocument};

pub fn export_merged_svg(document: &PagedDocument) -> Vec<u8> {
    let svg = typst_svg::svg_merged(document, Abs::pt(15.));
    svg.into_bytes()
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
        let svg = export_merged_svg(&document);

        assert!(svg.len() > 10);
    }

    #[test]
    fn svg_output_contains_svg_tag() {
        let document = compile(simple_template());
        let svg = export_merged_svg(&document);
        let svg_str = String::from_utf8_lossy(&svg);

        assert!(svg_str.contains("<svg"));
    }

    #[test]
    fn svg_output_is_valid_utf8() {
        let document = compile(simple_template());
        let svg = export_merged_svg(&document);

        assert!(String::from_utf8(svg).is_ok());
    }

    #[test]
    fn svg_output_has_closing_tag() {
        let document = compile(simple_template());
        let svg = export_merged_svg(&document);
        let svg_str = String::from_utf8_lossy(&svg);

        assert!(svg_str.contains("</svg>"));
    }

    #[test]
    fn svg_output_has_xmlns() {
        let document = compile(simple_template());
        let svg = export_merged_svg(&document);
        let svg_str = String::from_utf8_lossy(&svg);

        assert!(svg_str.contains("xmlns"));
    }

    #[test]
    fn exports_multipage_document() {
        let document = compile(multipage_template());
        let svg = export_merged_svg(&document);

        assert!(svg.len() > 200);
    }

    #[test]
    fn svg_export_is_deterministic() {
        let doc1 = compile(simple_template());
        let doc2 = compile(simple_template());

        let svg1 = export_merged_svg(&doc1);
        let svg2 = export_merged_svg(&doc2);

        assert_eq!(svg1, svg2);
    }

    #[test]
    fn multipage_svg_is_larger_than_single_page() {
        let single_doc = compile(simple_template());
        let multi_doc = compile(multipage_template());

        let single_svg = export_merged_svg(&single_doc);
        let multi_svg = export_merged_svg(&multi_doc);

        assert!(multi_svg.len() > single_svg.len());
    }
}
