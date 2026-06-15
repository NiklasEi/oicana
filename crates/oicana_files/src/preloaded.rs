use crate::TemplateFiles;
use log::warn;
use std::collections::HashMap;
use typst::diag::{FileError, FileResult};
use typst::foundations::Bytes;
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};

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
            let vpath = match VirtualPath::new(&path) {
                Ok(vpath) => vpath,
                Err(error) => {
                    warn!("Skipping file with invalid path {path}: {error}");
                    continue;
                }
            };
            let id = FileId::new(RootedPath::new(VirtualRoot::Project, vpath));
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
            .ok_or(FileError::NotFound(id.vpath().get_with_slash().into()))?
            .0
            .clone())
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        Ok(self
            .slots
            .get(&id)
            .ok_or(FileError::NotFound(id.vpath().get_with_slash().into()))?
            .1
            .clone())
    }

    /// Todo: preloaded currently doesn't support fonts from the template
    fn font_files(&self) -> &Vec<FileId> {
        &self.fonts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_file(path: &str) -> FileId {
        FileId::new(RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new(path).unwrap(),
        ))
    }

    #[test]
    fn creates_template_from_files() {
        let mut files = HashMap::new();
        files.insert("test.typ".to_owned(), "content".to_owned());

        let template = PreloadedTemplate::new(files);

        assert_eq!(template.slots.len(), 1);
    }

    #[test]
    fn source_returns_file_content() {
        let mut files = HashMap::new();
        files.insert("test.typ".to_owned(), "content".to_owned());
        let template = PreloadedTemplate::new(files);

        let id = project_file("test.typ");
        let source = template.source(id).unwrap();

        assert_eq!(source.text(), "content");
    }

    #[test]
    fn file_returns_bytes() {
        let mut files = HashMap::new();
        files.insert("test.typ".to_owned(), "content".to_owned());
        let template = PreloadedTemplate::new(files);

        let id = project_file("test.typ");
        let bytes = template.file(id).unwrap();

        assert_eq!(bytes.as_slice(), b"content");
    }

    #[test]
    fn source_fails_for_nonexistent_file() {
        let template = PreloadedTemplate::new(HashMap::new());
        let id = project_file("missing.typ");

        let result = template.source(id);

        assert!(result.is_err());
    }

    #[test]
    fn file_fails_for_nonexistent_file() {
        let template = PreloadedTemplate::new(HashMap::new());
        let id = project_file("missing.typ");

        let result = template.file(id);

        assert!(result.is_err());
    }

    #[test]
    fn handles_multiple_files() {
        let mut files = HashMap::new();
        files.insert("file1.typ".to_owned(), "content1".to_owned());
        files.insert("file2.typ".to_owned(), "content2".to_owned());
        let template = PreloadedTemplate::new(files);

        let id1 = project_file("file1.typ");
        let id2 = project_file("file2.typ");

        assert_eq!(template.source(id1).unwrap().text(), "content1");
        assert_eq!(template.source(id2).unwrap().text(), "content2");
    }
}
