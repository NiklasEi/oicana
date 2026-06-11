use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use log::warn;
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
            match files.file(*file_id) {
                Ok(data) => self.load_fonts_from_bytes(data),
                Err(error) => warn!(
                    "Skipping font file {}: {error}",
                    file_id.vpath().get_with_slash()
                ),
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use typst::diag::{FileError, FileResult};
    use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};

    struct UnreadableFontTemplate {
        fonts: Vec<FileId>,
    }

    impl TemplateFiles for UnreadableFontTemplate {
        fn source(&self, id: FileId) -> FileResult<Source> {
            Err(FileError::NotFound(id.vpath().get_with_slash().into()))
        }

        fn file(&self, id: FileId) -> FileResult<Bytes> {
            Err(FileError::NotFound(id.vpath().get_with_slash().into()))
        }

        fn font_files(&self) -> &Vec<FileId> {
            &self.fonts
        }
    }

    #[test]
    fn unreadable_template_font_is_skipped() {
        let template = UnreadableFontTemplate {
            fonts: vec![FileId::new(RootedPath::new(
                VirtualRoot::Project,
                VirtualPath::new("fonts/missing.ttf").unwrap(),
            ))],
        };

        let mut collection = FontCollection::new();
        collection.collect(&template);

        assert!(!collection.fonts.is_empty());
        assert!(collection.book.info(collection.fonts.len() - 1).is_some());
        assert!(collection.book.info(collection.fonts.len()).is_none());
    }
}
