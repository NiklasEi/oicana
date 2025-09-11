use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use oicana_files::TemplateFiles;
use typst::foundations::Bytes;
use typst::text::{Font, FontBook};

/// Collects all fonts.
pub struct FontCollection {
    /// Metadata about all discovered fonts.
    pub book: FontBook,
    /// Slots that the fonts are loaded into.
    pub fonts: Vec<FontSlot>,
}

/// Holds details about the location of a font and lazily the font itself.
#[derive(Debug)]
pub struct FontSlot {
    /// The path at which the font can be found on the system.
    path: PathBuf,
    /// The index of the font in its collection. Zero if the path does not point
    /// to a collection.
    index: u32,
    /// The lazily loaded font.
    font: OnceLock<Option<Font>>,
}

impl FontSlot {
    /// Get the font for this slot.
    pub fn get(&self) -> Option<Font> {
        self.font
            .get_or_init(|| {
                let data = Bytes::new(fs::read(&self.path).ok()?);
                Font::new(data, self.index)
            })
            .clone()
    }
}

impl FontCollection {
    /// Create a new, empty font collection.
    pub fn new() -> Self {
        Self {
            book: FontBook::new(),
            fonts: vec![],
        }
    }

    pub fn collect<Files: TemplateFiles>(&mut self, files: &Files) {
        // Fonts from the template have the highest priority
        self.add_template_fonts(files);
        self.add_embedded_fonts();
    }

    fn add_template_fonts<Files: TemplateFiles>(&mut self, files: &Files) {
        for file_id in files.font_files() {
            let data = files.file(*file_id).expect("Failed to read font file");
            self.load_fonts_from_bytes(data);
        }
    }

    fn add_embedded_fonts(&mut self) {
        for data in typst_assets::fonts() {
            let buffer = Bytes::new(data);
            self.load_fonts_from_bytes(buffer);
        }
    }

    fn load_fonts_from_bytes(&mut self, data: Bytes) {
        for (i, font) in Font::iter(data).enumerate() {
            self.book.push(font.info().clone());
            self.fonts.push(FontSlot {
                path: PathBuf::new(),
                index: i as u32,
                font: OnceLock::from(Some(font)),
            });
        }
    }
}
