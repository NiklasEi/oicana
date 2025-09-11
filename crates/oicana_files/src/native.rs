/// Part of this code and the submodules is from the Typst CLI implementation
/// for file access in a Typst World. Used under its MIT License.
use crate::TemplateFiles;
use download::PrintDownload;
use log::debug;
use std::collections::HashMap;
use std::fs::{create_dir_all, ReadDir};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::{fs, io, mem};
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
    if fs::metadata(path)
        .map_err(|_| FileError::NotFound(path.clone()))?
        .is_dir()
    {
        Err(FileError::IsDirectory)
    } else {
        Ok(fs::read(path).unwrap())
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
                append_font_ids(
                    fonts,
                    fs::read_dir(path).expect("Failed to read fonts sub dir"),
                    project_root,
                );
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
        root = prepare_package(spec, files)?;
    }

    // Join the path to the root. If it tries to escape, deny
    // access. Note: It can still escape via symlinks.
    id.vpath().resolve(&root).ok_or(FileError::AccessDenied)
}

/// Make a package available in the template dependencies directory.
pub fn prepare_package(spec: &PackageSpec, files: &NativeTemplate) -> Result<PathBuf, FileError> {
    let subdir = format!(
        ".dependencies/{}/{}/{}",
        spec.namespace, spec.name, spec.version
    );

    let package_dir = files.root.join(&subdir);
    if package_dir.exists() {
        return Ok(package_dir);
    }

    if spec.namespace == DEFAULT_NAMESPACE {
        // Download preview package from network if it doesn't exist yet.
        let cached_package = files
            .package_storage
            .prepare_package(spec, &mut PrintDownload(&spec))
            .map_err(FileError::Package)?;
        debug!("Copying {spec} from {cached_package:?}.");
        copy_directory(cached_package, package_dir.clone())
            .map_err(|io_error| PackageError::Other(Some(io_error.to_string().into())))?;
        return Ok(package_dir);
    }

    let local_package = files
        .packages
        .join(format!("{}/{}/{}", spec.namespace, spec.name, spec.version));
    if local_package.is_dir() {
        debug!("Copying {spec} from {local_package:?}.");
        copy_directory(local_package, package_dir.clone())
            .map_err(|io_error| PackageError::Other(Some(io_error.to_string().into())))?;
        return Ok(package_dir);
    }

    Err(FileError::Package(PackageError::NotFound(spec.clone())))
}

fn copy_directory(in_dir: PathBuf, out_dir: PathBuf) -> io::Result<()> {
    create_dir_all(&out_dir)?;
    for file in fs::read_dir(in_dir)? {
        let file = file?;
        let path = file.path();
        if path.is_dir() {
            copy_directory(path.clone(), out_dir.join(path.file_name().unwrap()))?;
        } else if path.is_file() {
            fs::copy(path.clone(), out_dir.join(path.file_name().unwrap()))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::native::copy_directory;
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn copy_dir() {
        let in_dir = tempdir().unwrap();
        create_dir_all(in_dir.path().join("test").join("src")).unwrap();
        {
            let file_path = in_dir.path().join("my-temporary-note.txt");
            let mut tmp_file = File::create(file_path).unwrap();
            tmp_file
                .write_all("Brian was here. Briefly.".as_bytes())
                .unwrap();
        }
        {
            let file_path = in_dir
                .path()
                .join("test")
                .join("my-temporary-other-note.txt");
            let mut tmp_file = File::create(file_path).unwrap();
            tmp_file
                .write_all("Brian was here. Briefly.".as_bytes())
                .unwrap();
        }

        let out_dir = tempdir().unwrap();
        let out_dir = out_dir.into_path();

        copy_directory(in_dir.into_path(), out_dir.clone()).unwrap();

        assert!(out_dir
            .join("test")
            .join("my-temporary-other-note.txt")
            .exists());
        assert!(out_dir.join("test").join("src").exists());
        assert!(out_dir.join("my-temporary-note.txt").exists());
    }
}
