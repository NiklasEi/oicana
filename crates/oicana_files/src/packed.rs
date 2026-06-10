use crate::TemplateFiles;
use log::warn;
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::str::FromStr;
use std::sync::Mutex;
use std::{str, vec};
use thiserror::Error;
use typst::diag::{FileError, FileResult};
use typst::foundations::Bytes;
use typst::syntax::package::{PackageSpec, PackageVersion};
use typst::syntax::{FileId, Source, VirtualPath};
use zip::read::ZipFile;
use zip::ZipArchive;

/// An error that occurred while reading a packed template.
#[derive(Error, Debug)]
pub enum PackedTemplateError {
    /// The zip archive could not be read.
    #[error("Failed to read archive: {0}")]
    InvalidArchive(#[from] zip::result::ZipError),
}

/// A packed template.
///
/// All source and byte entries are always loaded in memory.
pub struct PackedTemplate {
    source: Mutex<HashMap<FileId, Source>>,
    bytes: Mutex<HashMap<FileId, Bytes>>,
    fonts: Vec<FileId>,
}

impl PackedTemplate {
    /// Create a new packed template from a reader of a zip file.
    pub fn new<R: Read + Seek>(reader: R) -> Result<Self, PackedTemplateError> {
        let mut archive = ZipArchive::new(reader).map_err(PackedTemplateError::InvalidArchive)?;

        let mut source = HashMap::new();
        let mut bytes = HashMap::new();
        let mut fonts = vec![];
        let paths: Vec<String> = archive.file_names().map(|path| path.to_owned()).collect();
        for path in &paths {
            let file_result = archive.by_name(path);
            let content = match file_result {
                Ok(content) => content,
                Err(error) => {
                    warn!("Failed to read zip path {path}: {error}");
                    continue;
                }
            };
            if !content.is_file() {
                continue;
            }

            if let Some((dir, path)) = path.split_once("/") {
                if dir == ".dependencies" {
                    let Some((namespace, path)) = path.split_once("/") else {
                        warn!("No namespace for dependency path {path}");
                        continue;
                    };
                    let Some((package, path)) = path.split_once("/") else {
                        warn!("No package for dependency path {path}");
                        continue;
                    };
                    let Some((version, path)) = path.split_once("/") else {
                        warn!("No version for dependency path {path}");
                        continue;
                    };
                    let version = match PackageVersion::from_str(version) {
                        Ok(version) => version,
                        Err(error) => {
                            warn!(
                                "Skipping package file {path}, because version cannot be parsed: {error}"
                            );
                            continue;
                        }
                    };
                    let id = FileId::new(
                        Some(PackageSpec {
                            version,
                            name: package.into(),
                            namespace: namespace.into(),
                        }),
                        VirtualPath::new(path),
                    );
                    if is_font(path) {
                        fonts.push(id);
                    }
                    read_zip_file_content(&mut source, &mut bytes, content, id);
                    continue;
                }
            };

            let id = FileId::new(None, VirtualPath::new(path));
            if is_font(path) {
                fonts.push(id);
            }
            read_zip_file_content(&mut source, &mut bytes, content, id);
        }

        Ok(PackedTemplate {
            source: Mutex::new(source),
            bytes: Mutex::new(bytes),
            fonts,
        })
    }
}

fn read_zip_file_content<R: Read + Seek>(
    source: &mut HashMap<FileId, Source>,
    bytes: &mut HashMap<FileId, Bytes>,
    mut content: ZipFile<R>,
    id: FileId,
) {
    let mut buffer = Vec::with_capacity(8192);
    if let Err(error) = content.read_to_end(&mut buffer) {
        warn!(
            "Failed to read zip file content for {}: {error}",
            id.vpath().as_rooted_path().display()
        );
        return;
    }
    if let Ok(string_content) = str::from_utf8(&buffer) {
        source.insert(id, Source::new(id, string_content.to_owned()));
    }
    bytes.insert(id, Bytes::new(buffer));
}

fn is_font(path: &str) -> bool {
    let path = path.to_lowercase();
    path.ends_with(".ttf")
        || path.ends_with(".ttc")
        || path.ends_with(".otf")
        || path.ends_with(".otc")
}

impl TemplateFiles for PackedTemplate {
    fn source(&self, id: FileId) -> FileResult<Source> {
        let mut map = self.source.lock().unwrap();
        Ok(map
            .get_mut(&id)
            .ok_or(FileError::NotFound(
                id.vpath().as_rooted_path().to_path_buf(),
            ))?
            .clone())
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let mut map = self.bytes.lock().unwrap();
        Ok(map
            .get_mut(&id)
            .ok_or(FileError::NotFound(
                id.vpath().as_rooted_path().to_path_buf(),
            ))?
            .clone())
    }

    fn font_files(&self) -> &Vec<FileId> {
        &self.fonts
    }
}

#[cfg(test)]
mod tests {
    use crate::packed::PackedTemplate;
    use crate::TemplateFiles;
    use std::fs::read;
    use std::io::Cursor;
    use typst::diag::EcoString;
    use typst::syntax::package::PackageManifest;
    use typst::syntax::{FileId, VirtualPath};

    #[test]
    fn test_zip() {
        let template =
            read("../../assets/templates/table-0.1.0.zip").expect("Failed to read template zip");
        let files =
            PackedTemplate::new(Cursor::new(template)).expect("Failed to parse template zip");
        assert!(files
            .source(FileId::new(None, VirtualPath::new("/main.typ")))
            .is_ok());
    }

    #[test]
    fn can_read_manifest() {
        let template =
            read("../../assets/templates/table-0.1.0.zip").expect("Failed to read template zip");
        let files =
            PackedTemplate::new(Cursor::new(template)).expect("Failed to parse template zip");
        let manifest = files
            .source(FileId::new(None, VirtualPath::new("/typst.toml")))
            .expect("Failed to find typst.toml");

        let manifest: PackageManifest =
            toml::from_str(manifest.text()).expect("Failed to parse the manifest");
        assert!(manifest
            .tool
            .sections
            .contains_key(&EcoString::from("oicana")));
    }

    #[test]
    fn can_find_dependency() {
        let template =
            read("../../assets/templates/table-0.1.0.zip").expect("Failed to read template zip");
        let files =
            PackedTemplate::new(Cursor::new(template)).expect("Failed to parse template zip");

        assert!(files
            .file(FileId::new(
                Some("@preview/oicana:0.1.1".parse().unwrap()),
                VirtualPath::new("/typst.toml")
            ))
            .is_ok());
    }

    #[test]
    fn finds_fonts() {
        let template =
            read("../../assets/templates/fonts-0.1.0.zip").expect("Failed to read template zip");
        let files =
            PackedTemplate::new(Cursor::new(template)).expect("Failed to parse template zip");

        assert_eq!(
            files.fonts.iter().map(|id| id.vpath()).collect::<Vec<_>>(),
            vec![
                &VirtualPath::new("/fonts/NotoSansArabic-VariableFont_wdth,wght.ttf"),
                &VirtualPath::new("/fonts/InriaSerif-Regular.ttf")
            ]
        )
    }

    #[test]
    fn cannot_access_files_outside_zip() {
        let template =
            read("../../assets/templates/table-0.1.0.zip").expect("Failed to read template zip");
        let files =
            PackedTemplate::new(Cursor::new(template)).expect("Failed to parse template zip");

        // Attempting to access a path that doesn't exist in the zip returns NotFound
        assert!(files
            .file(FileId::new(None, VirtualPath::new("/../../etc/passwd")))
            .is_err());
        assert!(files
            .source(FileId::new(None, VirtualPath::new("/nonexistent.typ")))
            .is_err());
    }
}
