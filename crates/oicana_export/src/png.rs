use typst::layout::{Abs, PagedDocument};

pub use png::EncodingError;

pub fn export_merged_png(
    document: &PagedDocument,
    pixels_per_pt: f32,
) -> Result<Vec<u8>, EncodingError> {
    typst_render::render_merged(document, pixels_per_pt, Abs::pt(15.), None).encode_png()
}
