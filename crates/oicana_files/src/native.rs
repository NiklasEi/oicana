/// Part of this code and the submodules is from the Typst CLI implementation
/// for file access in a Typst World. Used under its MIT License.
use crate::TemplateFiles;
use download::PrintDownload;
use log::debug;
use std::collections::HashMap;
use std::fs::ReadDir;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::{fs, mem};
use typst::diag::{FileError, FileResult, PackageError};
use typst::foundations::Bytes;
use typst::syntax::package::PackageSpec;
use typst::syntax::{FileId, Source, VirtualPath};
use typst_kit::download::Downloader;
use typst_kit::package::{PackageStorage, DEFAULT_NAMESPACE};

mod download;
mod terminal;

/// An Oicana template in a native file system.
///
/// This is mostly used for testing and for template development.
/// This template will load files and dependencies on demand.
pub struct NativeTemplate {
    /// Lazily loaded file content of this template.
    pub slots: Mutex<HashMap<FileId, FileSlot>>,
    root: PathBuf,
    fonts: Vec<FileId>,
    package_storage: PackageStorage,
    packages: PathBuf,
}

impl NativeTemplate {
    /// Create a new template at the given path.
    pub fn new(root: &Path, packages: PathBuf) -> Self {
        debug!("Packages are globally stored at {packages:?}.");

        NativeTemplate {
            root: Path::new(root).to_owned(),
            slots: Mutex::new(HashMap::new()),
            fonts: find_fonts(root),
            package_storage: PackageStorage::new(
                Some(packages.clone()),
                Some(packages.clone()),
                downloader(),
            ),
            packages,
        }
    }

    fn slot<F, T>(&self, id: FileId, f: F) -> T
    where
        F: FnOnce(&mut FileSlot) -> T,
    {
        let mut map = self.slots.lock().unwrap();
        f(map.entry(id).or_insert_with(|| FileSlot::new(id)))
    }

    /// Reset the access tracking on all file slots in preparation for a new compilation.
    pub fn reset(&self) {
        for slot in self.slots.lock().unwrap().values_mut() {
            slot.reset();
        }
    }

    /// Resolve a package to its source directory.
    ///
    /// For `@preview` packages, this uses the global Typst package cache (downloading if needed).
    /// For local packages, this resolves from the local package registry.
    pub fn package_dir(&self, spec: &PackageSpec) -> Result<PathBuf, FileError> {
        if spec.namespace == DEFAULT_NAMESPACE {
            self.package_storage
                .prepare_package(spec, &mut PrintDownload(&spec))
                .map_err(FileError::Package)
        } else {
            let local_package = self
                .packages
                .join(format!("{}/{}/{}", spec.namespace, spec.name, spec.version));
            if local_package.is_dir() {
                Ok(local_package)
            } else {
                Err(FileError::Package(PackageError::NotFound(spec.clone())))
            }
        }
    }

    /// Return the system paths of all files that were accessed during the last compilation.
    pub fn dependencies(&self) -> Vec<PathBuf> {
        self.slots
            .lock()
            .unwrap()
            .values()
            .filter(|slot| slot.accessed())
            .filter_map(|slot| system_path(slot.id, self).ok())
            .collect()
    }
}

/// The path to Typsts package data directory on the current system
pub fn package_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|data| data.join("typst").join("packages"))
}

/// Returns a new downloader.
fn downloader() -> Downloader {
    let user_agent = concat!("oicana/", env!("CARGO_PKG_VERSION"));
    Downloader::new(user_agent)
}

impl TemplateFiles for NativeTemplate {
    fn source(&self, id: FileId) -> FileResult<Source> {
        self.slot(id, |slot| slot.source(self))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.slot(id, |slot| slot.file(self))
    }

    fn font_files(&self) -> &Vec<FileId> {
        &self.fonts
    }
}

/// Holds the processed data for a file ID.
///
/// Both fields can be populated if the file is both imported and `read()`.
pub struct FileSlot {
    /// The slot's file id.
    id: FileId,
    /// The lazily loaded and incrementally updated source file.
    source: SlotCell<Source>,
    /// The lazily loaded raw byte buffer.
    pub file: SlotCell<Bytes>,
}

impl FileSlot {
    /// Create a new path slot.
    pub fn new(id: FileId) -> Self {
        Self {
            id,
            file: SlotCell::new(),
            source: SlotCell::new(),
        }
    }

    /// Whether the file was accessed during the current compilation.
    pub fn accessed(&self) -> bool {
        self.source.accessed || self.file.accessed
    }

    /// Reset the access tracking for the next compilation.
    pub fn reset(&mut self) {
        self.source.accessed = false;
        self.file.accessed = false;
    }

    /// Retrieve the source for this file.
    pub(crate) fn source(&mut self, files: &NativeTemplate) -> Result<Source, FileError> {
        self.source.get_or_init(
            || read(self.id, files),
            |data, prev| {
                let text = decode_utf8(&data)?;
                if let Some(mut prev) = prev {
                    prev.replace(text);
                    Ok(prev)
                } else {
                    Ok(Source::new(self.id, text.into()))
                }
            },
        )
    }

    /// Retrieve the file's bytes.
    pub(crate) fn file(&mut self, files: &NativeTemplate) -> Result<Bytes, FileError> {
        self.file
            .get_or_init(|| read(self.id, files), |data, _| Ok(Bytes::new(data)))
    }
}

/// Lazily processes data for a file.
pub struct SlotCell<T> {
    /// The processed data.
    pub data: Option<Result<T, FileError>>,
    /// A hash of the raw file contents / access error.
    fingerprint: u128,
    /// Whether the slot has been accessed in the current compilation.
    pub accessed: bool,
}

impl<T: Clone> SlotCell<T> {
    /// Creates a new, empty cell.
    fn new() -> Self {
        Self {
            data: None,
            fingerprint: 0,
            accessed: false,
        }
    }

    /// Gets the contents of the cell or initialize them.
    fn get_or_init(
        &mut self,
        load: impl FnOnce() -> Result<Vec<u8>, FileError>,
        f: impl FnOnce(Vec<u8>, Option<T>) -> Result<T, FileError>,
    ) -> Result<T, FileError> {
        // If we accessed the file already in this compilation, retrieve it.
        if mem::replace(&mut self.accessed, true) {
            if let Some(data) = &self.data {
                return data.clone();
            }
        }

        // Read and hash the file.
        let result = load();
        let fingerprint = typst_utils::hash128(&result);

        // If the file contents didn't change, yield the old processed data.
        if mem::replace(&mut self.fingerprint, fingerprint) == fingerprint {
            if let Some(data) = &self.data {
                return data.clone();
            }
        }

        let prev = self.data.take().and_then(Result::ok);
        let value = result.and_then(|data| f(data, prev));
        self.data = Some(value.clone());

        value
    }
}

/// Reads a file from a `FileId`.
///
/// If the ID represents stdin it will read from standard input,
/// otherwise it gets the file path of the ID and reads the file from disk.
fn read(id: FileId, files: &NativeTemplate) -> Result<Vec<u8>, FileError> {
    let path = &system_path(id, files)?;
    let f = |error| FileError::from_io(error, path);
    if fs::metadata(path).map_err(f)?.is_dir() {
        Err(FileError::IsDirectory)
    } else {
        fs::read(path).map_err(f)
    }
}

fn find_fonts(project_root: &Path) -> Vec<FileId> {
    let fonts_dir = match fs::read_dir(project_root) {
        Ok(dir) => dir,
        Err(_) => return vec![],
    };

    fn append_font_ids(fonts: &mut Vec<FileId>, fonts_dir: ReadDir, project_root: &Path) {
        for entry in fonts_dir.flatten() {
            let path = entry.path();
            if path.is_file() {
                match path.extension().and_then(|e| e.to_str()) {
                    Some("ttf") | Some("ttc") | Some("TTF") | Some("TTC") | Some("otf")
                    | Some("otc") | Some("OTF") | Some("OTC") => fonts.push(FileId::new(
                        None,
                        VirtualPath::new(path.strip_prefix(project_root).unwrap()),
                    )),
                    _ => {}
                }
            } else if path.is_dir() {
                match fs::read_dir(&path) {
                    Ok(dir) => append_font_ids(fonts, dir, project_root),
                    Err(error) => debug!(
                        "Skipping directory {:?} while looking for font files: {}",
                        path, error
                    ),
                };
            }
        }
    }
    let mut fonts = vec![];
    append_font_ids(&mut fonts, fonts_dir, project_root);

    fonts
}

/// Decode UTF-8 with an optional BOM.
fn decode_utf8(buf: &[u8]) -> Result<&str, FileError> {
    // Remove UTF-8 BOM.
    Ok(std::str::from_utf8(
        buf.strip_prefix(b"\xef\xbb\xbf").unwrap_or(buf),
    )?)
}

/// Resolves the path of a file id on the system, downloading a package if
/// necessary.
fn system_path(id: FileId, files: &NativeTemplate) -> Result<PathBuf, FileError> {
    // Determine the root path relative to which the file path
    // will be resolved.
    let mut root = files.root.to_owned();
    if let Some(spec) = id.package() {
        root = files.package_dir(spec)?;
    }

    // Join the path to the root. If it tries to escape, deny
    // access. Note: It can still escape via symlinks, but native
    // templates are only used during development, not at runtime.
    id.vpath().resolve(&root).ok_or(FileError::AccessDenied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn template(root: &Path) -> NativeTemplate {
        NativeTemplate::new(root, root.join(".packages"))
    }

    fn symlink_file(src: &Path, dst: &Path) {
        #[cfg(unix)]
        std::os::unix::fs::symlink(src, dst).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(src, dst).unwrap();
    }

    #[test]
    fn reads_source_and_bytes_of_a_file() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.typ"), "Hello").unwrap();
        let files = template(dir.path());

        let id = FileId::new(None, VirtualPath::new("main.typ"));
        assert_eq!(files.source(id).unwrap().text(), "Hello");
        assert_eq!(files.file(id).unwrap().as_slice(), b"Hello");
    }

    #[test]
    fn reading_a_missing_file_returns_not_found() {
        let dir = TempDir::new().unwrap();
        let files = template(dir.path());

        let id = FileId::new(None, VirtualPath::new("missing.typ"));
        assert!(matches!(files.file(id), Err(FileError::NotFound(_))));
        assert!(matches!(files.source(id), Err(FileError::NotFound(_))));
    }

    #[test]
    fn reading_follows_a_symlink_to_its_target() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("real.typ");
        fs::write(&target, "from target").unwrap();
        symlink_file(&target, &dir.path().join("link.typ"));
        let files = template(dir.path());

        let id = FileId::new(None, VirtualPath::new("link.typ"));
        assert_eq!(files.source(id).unwrap().text(), "from target");
        assert_eq!(files.file(id).unwrap().as_slice(), b"from target");
    }

    #[test]
    fn reading_a_directory_returns_is_directory() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        let files = template(dir.path());

        let id = FileId::new(None, VirtualPath::new("sub"));
        assert!(matches!(files.file(id), Err(FileError::IsDirectory)));
    }

    #[test]
    fn path_escaping_the_root_is_denied() {
        let dir = TempDir::new().unwrap();
        let files = template(dir.path());

        let id = FileId::new(None, VirtualPath::new("../../etc/passwd"));
        assert!(matches!(files.file(id), Err(FileError::AccessDenied)));
    }

    #[test]
    fn decode_utf8_strips_bom() {
        assert_eq!(decode_utf8(b"\xef\xbb\xbfhello").unwrap(), "hello");
        assert_eq!(decode_utf8(b"hello").unwrap(), "hello");
    }
}
