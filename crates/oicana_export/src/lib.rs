pub mod pages;

#[cfg(feature = "pdf")]
pub use oicana_template::PdfStandard;

#[cfg(feature = "pdf")]
pub mod pdf;
#[cfg(feature = "png")]
pub mod png;
#[cfg(feature = "svg")]
pub mod svg;
