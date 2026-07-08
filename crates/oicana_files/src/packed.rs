use crate::TemplateFiles;
use log::warn;
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::str::FromStr;
use std::sync::{Mutex, PoisonError};
use std::{str, vec};
use thiserror::Error;
use typst::diag::{FileError, FileResult};
use typst::foundations::Bytes;
use typst::syntax::package::{PackageSpec, PackageVersion};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use zip::read::ZipFile;
use zip::ZipArchive;

/// An error that occurred while reading a packed template.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum PackedTemplateError {
    /// The zip archive could not be read.
    #[error("Failed to read archive: {0}")]
    InvalidArchive(#[from] zip::result::ZipError),
    /// The template archive contains more entries than allowed by [`ZipLimits`].
    #[error("Template archive has {count} entries, exceeding the limit of {limit}")]
    TooManyEntries {
        /// The number of entries in the archive.
        count: usize,
        /// The configured entry limit.
        limit: usize,
    },
    /// The decompressed template archive content exceeds the size allowed by [`ZipLimits`].
    #[error("Decompressed template archive content exceeds the limit of {limit} bytes")]
    TooLarge {
        /// The configured limit for the total decompressed size in bytes.
        limit: u64,
    },
}

/// Limits applied while reading a packed template archive.
#[derive(Debug, Clone, Copy)]
pub struct ZipLimits {
    /// Maximum number of entries in the archive.
    pub max_entries: usize,
    /// Maximum total decompressed size of all entries in bytes.
    pub max_total_decompressed_bytes: u64,
}

impl Default for ZipLimits {
    fn default() -> Self {
        ZipLimits {
            max_entries: 10_000,
            max_total_decompressed_bytes: 512 * 1024 * 1024,
        }
    }
}

impl ZipLimits {
    /// Check the sizes declared in an archive's central directory against these limits.
    pub fn check_declared<R: Read + Seek>(&self, reader: R) -> Result<(), PackedTemplateError> {
        let mut archive = ZipArchive::new(reader).map_err(PackedTemplateError::InvalidArchive)?;
        if archive.len() > self.max_entries {
            return Err(PackedTemplateError::TooManyEntries {
                count: archive.len(),
                limit: self.max_entries,
            });
        }
        let mut total_bytes: u64 = 0;
        for index in 0..archive.len() {
            let entry = archive
                .by_index_raw(index)
                .map_err(PackedTemplateError::InvalidArchive)?;
            total_bytes = total_bytes.saturating_add(entry.size());
        }
        if total_bytes > self.max_total_decompressed_bytes {
            return Err(PackedTemplateError::TooLarge {
                limit: self.max_total_decompressed_bytes,
            });
        }
        Ok(())
    }
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
        Self::new_with_limits(reader, ZipLimits::default())
    }

    /// Create a new packed template from a reader of a zip file, enforcing the given limits.
    pub fn new_with_limits<R: Read + Seek>(
        reader: R,
        limits: ZipLimits,
    ) -> Result<Self, PackedTemplateError> {
        let mut archive = ZipArchive::new(reader).map_err(PackedTemplateError::InvalidArchive)?;
        if archive.len() > limits.max_entries {
            return Err(PackedTemplateError::TooManyEntries {
                count: archive.len(),
                limit: limits.max_entries,
            });
        }
        let mut remaining_bytes = limits.max_total_decompressed_bytes;

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
                    let vpath = match VirtualPath::new(path) {
                        Ok(vpath) => vpath,
                        Err(error) => {
                            warn!("Skipping package file with invalid path {path}: {error}");
                            continue;
                        }
                    };
                    let id = FileId::new(RootedPath::new(
                        VirtualRoot::Package(PackageSpec {
                            version,
                            name: package.into(),
                            namespace: namespace.into(),
                        }),
                        vpath,
                    ));
                    if is_font(path) {
                        fonts.push(id);
                    }
                    read_zip_file_content(
                        &mut source,
                        &mut bytes,
                        content,
                        id,
                        &mut remaining_bytes,
                        &limits,
                    )?;
                    continue;
                }
            };

            let vpath = match VirtualPath::new(path) {
                Ok(vpath) => vpath,
                Err(error) => {
                    warn!("Skipping file with invalid path {path}: {error}");
                    continue;
                }
            };
            let id = FileId::new(RootedPath::new(VirtualRoot::Project, vpath));
            if is_font(path) {
                fonts.push(id);
            }
            read_zip_file_content(
                &mut source,
                &mut bytes,
                content,
                id,
                &mut remaining_bytes,
                &limits,
            )?;
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
    content: ZipFile<R>,
    id: FileId,
    remaining_bytes: &mut u64,
    limits: &ZipLimits,
) -> Result<(), PackedTemplateError> {
    let mut buffer = Vec::with_capacity(8192);
    let mut limited = content.take(remaining_bytes.saturating_add(1));
    if let Err(error) = limited.read_to_end(&mut buffer) {
        warn!(
            "Failed to read zip file content for {}: {error}",
            id.vpath().get_with_slash()
        );
        return Ok(());
    }
    if buffer.len() as u64 > *remaining_bytes {
        return Err(PackedTemplateError::TooLarge {
            limit: limits.max_total_decompressed_bytes,
        });
    }
    *remaining_bytes -= buffer.len() as u64;
    if let Ok(string_content) = str::from_utf8(&buffer) {
        source.insert(id, Source::new(id, string_content.to_owned()));
    }
    bytes.insert(id, Bytes::new(buffer));
    Ok(())
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
        // The maps are never structurally modified after construction, so a
        // poisoned lock cannot mean inconsistent data.
        let mut map = self.source.lock().unwrap_or_else(PoisonError::into_inner);
        Ok(map
            .get_mut(&id)
            .ok_or(FileError::NotFound(id.vpath().get_with_slash().into()))?
            .clone())
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let mut map = self.bytes.lock().unwrap_or_else(PoisonError::into_inner);
        Ok(map
            .get_mut(&id)
            .ok_or(FileError::NotFound(id.vpath().get_with_slash().into()))?
            .clone())
    }

    fn font_files(&self) -> &Vec<FileId> {
        &self.fonts
    }
}

#[cfg(test)]
mod tests {
    use crate::packed::{PackedTemplate, PackedTemplateError, ZipLimits};
    use crate::TemplateFiles;
    use std::fs::read;
    use std::io::{Cursor, Write};
    use typst::diag::EcoString;
    use typst::syntax::package::{PackageManifest, PackageSpec};
    use typst::syntax::{FileId, RootedPath, VirtualPath, VirtualRoot};
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    fn project_file(path: &str) -> FileId {
        FileId::new(RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new(path).unwrap(),
        ))
    }

    fn package_file(spec: PackageSpec, path: &str) -> FileId {
        FileId::new(RootedPath::new(
            VirtualRoot::Package(spec),
            VirtualPath::new(path).unwrap(),
        ))
    }

    #[test]
    fn test_zip() {
        let template =
            read("../../assets/templates/table-0.1.0.zip").expect("Failed to read template zip");
        let files =
            PackedTemplate::new(Cursor::new(template)).expect("Failed to parse template zip");
        assert!(files.source(project_file("/main.typ")).is_ok());
    }

    #[test]
    fn recovers_from_poisoned_file_maps() {
        let template =
            read("../../assets/templates/table-0.1.0.zip").expect("Failed to read template zip");
        let files =
            PackedTemplate::new(Cursor::new(template)).expect("Failed to parse template zip");

        let _ = std::panic::catch_unwind(|| {
            let _source = files.source.lock().unwrap();
            let _bytes = files.bytes.lock().unwrap();
            panic!("poison the file maps");
        });
        assert!(files.source.is_poisoned());
        assert!(files.bytes.is_poisoned());

        assert!(files.source(project_file("/main.typ")).is_ok());
        assert!(files
            .file(package_file(
                "@preview/oicana:0.1.1".parse().unwrap(),
                "/typst.toml"
            ))
            .is_ok());
    }

    #[test]
    fn can_read_manifest() {
        let template =
            read("../../assets/templates/table-0.1.0.zip").expect("Failed to read template zip");
        let files =
            PackedTemplate::new(Cursor::new(template)).expect("Failed to parse template zip");
        let manifest = files
            .source(project_file("/typst.toml"))
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
            .file(package_file(
                "@preview/oicana:0.1.1".parse().unwrap(),
                "/typst.toml"
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
                &VirtualPath::new("/fonts/NotoSansArabic-VariableFont_wdth,wght.ttf").unwrap(),
                &VirtualPath::new("/fonts/InriaSerif-Regular.ttf").unwrap()
            ]
        )
    }

    fn zip_with_entries(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Zstd);
        for (name, content) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn rejects_archive_with_too_many_entries() {
        let archive = zip_with_entries(&[
            ("a.typ", b"a".as_slice()),
            ("b.typ", b"b".as_slice()),
            ("c.typ", b"c".as_slice()),
        ]);

        let result = PackedTemplate::new_with_limits(
            Cursor::new(archive),
            ZipLimits {
                max_entries: 2,
                ..ZipLimits::default()
            },
        );

        assert!(matches!(
            result,
            Err(PackedTemplateError::TooManyEntries { count: 3, limit: 2 })
        ));
    }

    #[test]
    fn rejects_archive_exceeding_the_decompressed_size_limit() {
        // Highly compressible content: the zip stays tiny, only the
        // decompressed size trips the limit (a zip bomb in miniature).
        let archive = zip_with_entries(&[("main.typ", vec![b' '; 1024 * 1024].as_slice())]);
        assert!(archive.len() < 16 * 1024);

        let result = PackedTemplate::new_with_limits(
            Cursor::new(archive),
            ZipLimits {
                max_total_decompressed_bytes: 64 * 1024,
                ..ZipLimits::default()
            },
        );

        assert!(matches!(
            result,
            Err(PackedTemplateError::TooLarge { limit }) if limit == 64 * 1024
        ));
    }

    #[test]
    fn size_limit_applies_to_the_sum_of_all_entries() {
        let content = vec![b' '; 40 * 1024];
        let archive =
            zip_with_entries(&[("a.typ", content.as_slice()), ("b.typ", content.as_slice())]);

        let result = PackedTemplate::new_with_limits(
            Cursor::new(archive),
            ZipLimits {
                max_total_decompressed_bytes: 64 * 1024,
                ..ZipLimits::default()
            },
        );

        assert!(matches!(result, Err(PackedTemplateError::TooLarge { .. })));
    }

    #[test]
    fn reads_archive_that_exactly_fits_the_limits() {
        let content = vec![b' '; 32 * 1024];
        let archive =
            zip_with_entries(&[("a.typ", content.as_slice()), ("b.typ", content.as_slice())]);

        let files = PackedTemplate::new_with_limits(
            Cursor::new(archive),
            ZipLimits {
                max_entries: 2,
                max_total_decompressed_bytes: 64 * 1024,
            },
        )
        .expect("archive within the limits should load");

        assert!(files.source(project_file("/a.typ")).is_ok());
        assert!(files.source(project_file("/b.typ")).is_ok());
    }

    #[test]
    fn check_declared_rejects_too_many_entries() {
        let archive = zip_with_entries(&[
            ("a.typ", b"a".as_slice()),
            ("b.typ", b"b".as_slice()),
            ("c.typ", b"c".as_slice()),
        ]);

        let result = ZipLimits {
            max_entries: 2,
            ..ZipLimits::default()
        }
        .check_declared(Cursor::new(archive));

        assert!(matches!(
            result,
            Err(PackedTemplateError::TooManyEntries { count: 3, limit: 2 })
        ));
    }

    #[test]
    fn check_declared_rejects_oversized_content() {
        let content = vec![b' '; 40 * 1024];
        let archive =
            zip_with_entries(&[("a.typ", content.as_slice()), ("b.typ", content.as_slice())]);

        let result = ZipLimits {
            max_total_decompressed_bytes: 64 * 1024,
            ..ZipLimits::default()
        }
        .check_declared(Cursor::new(archive));

        assert!(matches!(
            result,
            Err(PackedTemplateError::TooLarge { limit }) if limit == 64 * 1024
        ));
    }

    #[test]
    fn check_declared_accepts_archive_within_limits() {
        let template =
            read("../../assets/templates/table-0.1.0.zip").expect("Failed to read template zip");

        assert!(ZipLimits::default()
            .check_declared(Cursor::new(template))
            .is_ok());
    }

    #[test]
    fn cannot_access_files_outside_zip() {
        let template =
            read("../../assets/templates/table-0.1.0.zip").expect("Failed to read template zip");
        let files =
            PackedTemplate::new(Cursor::new(template)).expect("Failed to parse template zip");

        // Paths escaping the root cannot even be constructed
        assert!(VirtualPath::new("/../../etc/passwd").is_err());
        // Attempting to access a path that doesn't exist in the zip returns NotFound
        assert!(files.source(project_file("/nonexistent.typ")).is_err());
    }
}
