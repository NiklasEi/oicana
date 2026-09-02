//! The manifest of a template is available to the host.

use oicana::files::packed::PackedTemplate;
use oicana::input::input_definition::InputDefinition;
use oicana::template::PdfStandard;
use oicana::Template;
use std::fs::read;
use std::io::Cursor;

#[test]
fn exposes_the_package_section_and_the_oicana_config() {
    let template_file = read("../../../e2e-tests/template/oicana-e2e-test-x.y.z.zip")
        .expect("read the e2e test template");
    let template =
        Template::<PackedTemplate>::init(Cursor::new(template_file)).expect("init template");

    let manifest = template.manifest();

    assert_eq!(manifest.package.name, "oicana-e2e-test");
    assert_eq!(manifest.package.version.to_string(), "0.1.0");
    assert_eq!(manifest.tool.oicana.manifest_version, 1);
    assert!(manifest.tool.oicana.validate_json_inputs_by_default);
    assert_eq!(manifest.pdf_standards(), [PdfStandard::A_3b]);
    assert!(manifest.pdf_tagged());
    assert!(manifest.required_font_families().is_empty());

    let json = manifest
        .tool
        .oicana
        .inputs
        .iter()
        .find_map(|input| match input {
            InputDefinition::Json(json) if json.key == "development-json" => Some(json),
            _ => None,
        })
        .expect("the template declares the 'development-json' input");
    assert_eq!(json.schema.as_deref(), Some("input.schema.json"));
    assert_eq!(json.development.as_deref(), Some("development.json"));
    assert_eq!(json.default, None);
    assert!(json.validate);

    let blob = manifest
        .tool
        .oicana
        .inputs
        .iter()
        .find_map(|input| match input {
            InputDefinition::Blob(blob) if blob.key == "default-blob" => Some(blob),
            _ => None,
        })
        .expect("the template declares the 'default-blob' input");
    let default = blob.default.as_ref().expect("the input has a default blob");
    assert_eq!(default.file, "default.txt");
    assert_eq!(
        default
            .meta
            .as_ref()
            .and_then(|meta| meta.get("image_format"))
            .and_then(|format| format.as_str()),
        Some("png")
    );
    assert!(blob.development.is_none());
}
