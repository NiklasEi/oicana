//! The Rust integration refuses templates it cannot interpret.

use oicana::files::packed::PackedTemplate;
use oicana::Template;
use std::fs::read;
use std::io::Cursor;

#[test]
fn refuses_a_template_packed_by_a_newer_oicana() {
    let template_file = read("../../../assets/templates/future-manifest-0.1.0.zip")
        .expect("read test template fixture");

    let error = Template::<PackedTemplate>::init(Cursor::new(template_file))
        .err()
        .expect("a template declaring a newer manifest version must not initialize");

    assert!(
        error.to_string().contains("manifest_version 99"),
        "got: {error}"
    );
}
