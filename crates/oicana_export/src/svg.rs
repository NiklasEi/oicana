use typst::layout::{Abs, PagedDocument};

pub fn export_merged_svg(document: &PagedDocument) -> Vec<u8> {
    let svg = typst_svg::svg_merged(document, Abs::pt(15.));
    svg.into_bytes()
}
