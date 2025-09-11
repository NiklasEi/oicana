use crate::TemplateFiles;
use std::collections::HashMap;
use typst::diag::{FileError, FileResult};
use typst::foundations::Bytes;
use typst::syntax::{FileId, Source, VirtualPath};

/// A preloaded template.
///
/// This is intended for test purposes only.
#[derive(Debug)]
pub struct PreloadedTemplate {
    slots: HashMap<FileId, (Source, Bytes)>,
    fonts: Vec<FileId>,
}

impl PreloadedTemplate {
    /// Create a new preloaded template from a map.
    ///
    /// Every map entry is a file in the template.
    pub fn new(files: HashMap<String, String>) -> Self {
        let mut slots = HashMap::new();
        for (path, content) in files {
            let id = FileId::new(None, VirtualPath::new(path));
            slots.insert(
                id,
                (
                    Source::new(id, content.clone()),
                    Bytes::new(content.into_bytes()),
                ),
            );
        }

        PreloadedTemplate {
            slots,
            fonts: vec![],
        }
    }
}

impl TemplateFiles for PreloadedTemplate {
    fn source(&self, id: FileId) -> FileResult<Source> {
        Ok(self
            .slots
            .get(&id)
            .ok_or(FileError::NotFound(
                id.vpath().as_rooted_path().to_path_buf(),
            ))?
            .0
            .clone())
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        Ok(self
            .slots
            .get(&id)
            .ok_or(FileError::NotFound(
                id.vpath().as_rooted_path().to_path_buf(),
            ))?
            .1
            .clone())
    }

    /// preloaded currently doesn't support fonts from the template
    fn font_files(&self) -> &Vec<FileId> {
        &self.fonts
    }
}
