#[cfg(not(target_arch = "wasm32"))]
use std::fs;
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;

use log::warn;
use oicana_files::TemplateFiles;
use typst::foundations::Bytes;
use typst::text::{Font, FontBook, FontInfo};

/// A font made available to a template by its host.
///
/// Create a source once and hand the same one to every world that needs it.
#[derive(Debug, Clone)]
pub enum FontSource {
    /// Font data held in memory.
    ///
    /// The faces are parsed once when the source is created and shared by every
    /// world it is passed to.
    Bytes(Arc<Vec<Font>>),
    /// A font file on disk, whose data is not retained until it is used.
    ///
    /// The file is read once to collect the face metadata and then dropped; it
    /// is read again, and kept, the first time a glyph actually needs it.
    #[cfg(not(target_arch = "wasm32"))]
    Path(Arc<PathFontSource>),
}

impl FontSource {
    /// Parse font data into a source, sharing the parsed faces.
    ///
    /// Returns `None` if the data does not contain any font Typst can read.
    pub fn from_bytes(data: impl Into<Bytes>) -> Option<Self> {
        let fonts: Vec<Font> = Font::iter(data.into()).collect();
        if fonts.is_empty() {
            return None;
        }
        Some(FontSource::Bytes(Arc::new(fonts)))
    }

    /// Read the face metadata of a font file without retaining its data.
    ///
    /// The file is read to collect the metadata and then dropped, so nothing
    /// large stays in memory until a glyph needs it. Call this once per font
    /// and reuse the result.
    ///
    /// Returns `None` if the file cannot be read or holds no font Typst can
    /// read; the reason is logged.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_path(path: impl Into<PathBuf>) -> Option<Self> {
        let path = path.into();
        let data = match fs::read(&path) {
            Ok(data) => data,
            Err(error) => {
                warn!("Skipping font file {path:?}: {error}");
                return None;
            }
        };
        let infos: Vec<FontInfo> = FontInfo::iter(&data).collect();
        if infos.is_empty() {
            warn!("Skipping font file {path:?}: no font faces found");
            return None;
        }
        Some(FontSource::Path(Arc::new(PathFontSource {
            path,
            infos,
            data: OnceLock::new(),
        })))
    }

    /// The families of all faces in this source, in face order.
    pub fn families(&self) -> Vec<String> {
        match self {
            FontSource::Bytes(fonts) => fonts
                .iter()
                .map(|font| font.info().family.clone())
                .collect(),
            #[cfg(not(target_arch = "wasm32"))]
            FontSource::Path(source) => source
                .infos
                .iter()
                .map(|info| info.family.clone())
                .collect(),
        }
    }

    /// The file this source was read from, if it came from disk.
    pub fn path(&self) -> Option<&Path> {
        match self {
            FontSource::Bytes(_) => None,
            #[cfg(not(target_arch = "wasm32"))]
            FontSource::Path(source) => Some(&source.path),
        }
    }
}

/// A font file whose data is only read when a glyph needs it.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct PathFontSource {
    path: PathBuf,
    infos: Vec<FontInfo>,
    /// The file content, shared by all faces of this file so a collection is
    /// never read more than once.
    data: OnceLock<Option<Bytes>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl PathFontSource {
    /// Load the face at `index`, reading the file on first access.
    fn face(&self, index: u32) -> Option<Font> {
        let data = self
            .data
            .get_or_init(|| match fs::read(&self.path) {
                Ok(data) => Some(Bytes::new(data)),
                Err(error) => {
                    warn!("Failed to read font file {:?}: {error}", self.path);
                    None
                }
            })
            .clone()?;
        Font::new(data, index)
    }
}

/// Collects all fonts.
pub struct FontCollection {
    /// Metadata about all discovered fonts.
    pub book: FontBook,
    /// Slots that the fonts are loaded into.
    pub fonts: Vec<FontSlot>,
}

/// Holds a font, either loaded or waiting to be read from its file.
#[derive(Debug)]
pub enum FontSlot {
    /// A font that is already loaded.
    Loaded(Font),
    /// A face of a font file that is read on first use.
    #[cfg(not(target_arch = "wasm32"))]
    Lazy {
        /// The file the face belongs to.
        source: Arc<PathFontSource>,
        /// The index of the face in its file.
        index: u32,
        /// The lazily loaded font.
        font: OnceLock<Option<Font>>,
    },
}

impl FontSlot {
    /// Get the font for this slot.
    pub fn get(&self) -> Option<Font> {
        match self {
            FontSlot::Loaded(font) => Some(font.clone()),
            #[cfg(not(target_arch = "wasm32"))]
            FontSlot::Lazy {
                source,
                index,
                font,
            } => font.get_or_init(|| source.face(*index)).clone(),
        }
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

    /// Collect the fonts available to a template.
    ///
    /// Fonts are added in decreasing priority, because Typst's fallback
    /// resolves ties by the order faces were pushed into the book: fonts packed
    /// with the template win over fonts the host registered, which in turn win
    /// over the fonts embedded in Typst.
    pub fn collect<Files: TemplateFiles>(&mut self, files: &Files, host_fonts: &[FontSource]) {
        self.add_template_fonts(files);
        self.add_host_fonts(host_fonts);
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

    fn add_host_fonts(&mut self, host_fonts: &[FontSource]) {
        for source in host_fonts {
            match source {
                FontSource::Bytes(fonts) => {
                    for font in fonts.iter() {
                        self.push(font.info().clone(), FontSlot::Loaded(font.clone()));
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                FontSource::Path(path_source) => {
                    for (index, info) in path_source.infos.iter().enumerate() {
                        self.push(
                            info.clone(),
                            FontSlot::Lazy {
                                source: Arc::clone(path_source),
                                index: index as u32,
                                font: OnceLock::new(),
                            },
                        );
                    }
                }
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
        for font in Font::iter(data) {
            self.push(font.info().clone(), FontSlot::Loaded(font));
        }
    }

    fn push(&mut self, info: FontInfo, slot: FontSlot) {
        self.book.push(info);
        self.fonts.push(slot);
    }
}

impl Default for FontCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// Font families that are required but not present in the given book.
///
/// Matching follows Typst's own family resolution, which is case insensitive.
pub fn missing_font_families(book: &FontBook, required: &[String]) -> Vec<String> {
    required
        .iter()
        .filter(|family| book.select_family(&family.to_lowercase()).next().is_none())
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use typst::diag::{FileError, FileResult};
    use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};

    struct TestTemplate {
        fonts: Vec<FileId>,
    }

    impl TestTemplate {
        fn without_fonts() -> Self {
            TestTemplate { fonts: vec![] }
        }

        fn with_unreadable_font() -> Self {
            TestTemplate {
                fonts: vec![FileId::new(RootedPath::new(
                    VirtualRoot::Project,
                    VirtualPath::new("fonts/missing.ttf").unwrap(),
                ))],
            }
        }
    }

    impl TemplateFiles for TestTemplate {
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

    fn embedded_font_bytes() -> Bytes {
        Bytes::new(
            typst_assets::fonts()
                .next()
                .expect("typst ships embedded fonts"),
        )
    }

    fn embedded_only() -> FontCollection {
        let mut collection = FontCollection::new();
        collection.collect(&TestTemplate::without_fonts(), &[]);
        collection
    }

    #[test]
    fn unreadable_template_font_is_skipped() {
        let mut collection = FontCollection::new();
        collection.collect(&TestTemplate::with_unreadable_font(), &[]);

        assert!(!collection.fonts.is_empty());
        assert!(collection.book.info(collection.fonts.len() - 1).is_some());
        assert!(collection.book.info(collection.fonts.len()).is_none());
    }

    #[test]
    fn host_fonts_are_added_before_the_embedded_ones() {
        let host = FontSource::from_bytes(embedded_font_bytes()).expect("valid font");
        let host_faces = host.families().len();

        let mut collection = FontCollection::new();
        collection.collect(&TestTemplate::without_fonts(), &[host]);

        // Host faces come first, so they win Typst's fallback tie-break.
        assert_eq!(
            collection.fonts.len(),
            embedded_only().fonts.len() + host_faces
        );
        for slot in collection.fonts.iter().take(host_faces) {
            assert!(slot.get().is_some());
        }
    }

    #[test]
    fn font_data_without_a_font_is_rejected() {
        assert!(FontSource::from_bytes(Bytes::new(b"not a font".to_vec())).is_none());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn fonts_are_read_from_disk_lazily() {
        let dir = std::env::temp_dir().join("oicana_lazy_font_test");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("font.ttf");
        fs::write(&path, embedded_font_bytes().as_slice()).unwrap();

        let source = FontSource::from_path(&path).expect("valid font file");
        assert!(!source.families().is_empty());
        assert_eq!(source.path(), Some(path.as_path()));

        let FontSource::Path(ref path_source) = source else {
            panic!("expected a path source");
        };
        assert!(path_source.data.get().is_none());

        let mut collection = FontCollection::new();
        collection.collect(
            &TestTemplate::without_fonts(),
            std::slice::from_ref(&source),
        );
        assert!(path_source.data.get().is_none());

        assert!(collection.fonts[0].get().is_some());
        assert!(path_source.data.get().is_some());

        fs::remove_file(&path).ok();
    }

    #[test]
    fn missing_families_are_reported_case_insensitively() {
        let collection = embedded_only();
        let present = collection.book.info(0).unwrap().family.clone();

        assert!(missing_font_families(&collection.book, &[present.to_uppercase()]).is_empty());
        assert_eq!(
            missing_font_families(&collection.book, &["No Such Family".to_owned()]),
            vec!["No Such Family".to_owned()]
        );
    }
}
