use chrono::{Datelike, Timelike, Utc};
use log::trace;
use std::fs::File;
use std::io;
use std::io::{Read, Seek, Write};
use std::num::TryFromIntError;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};
use zip::result::{DateTimeRangeError, ZipError};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipWriter};

use crate::manifest::TemplateManifest;

/// Package a directory as an Oicana template.
pub fn package<T>(
    src_dir: &Path,
    writer: T,
    manifest: &TemplateManifest,
) -> Result<(), PackageError>
where
    T: Write + Seek,
{
    if !Path::new(src_dir).is_dir() {
        return Err(PackageError::SourceIsNotADirectory);
    }

    let walk_dir = WalkDir::new(src_dir);
    let it = walk_dir.into_iter().filter_entry(|entry| {
        manifest.should_path_be_packed(entry.path().strip_prefix(src_dir).unwrap())
    });

    zip_dir(
        &mut it.filter_map(|e| e.ok()),
        src_dir,
        writer,
        CompressionMethod::ZSTD,
    )?;

    Ok(())
}

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &Path,
    writer: T,
    method: CompressionMethod,
) -> Result<(), PackageError>
where
    T: Write + Seek,
{
    let mut zip = ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let prefix = Path::new(prefix);
    let mut buffer = Vec::with_capacity(4096);
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(prefix).unwrap();
        let path_as_string = name
            .to_str()
            .map(str::to_owned)
            .ok_or(PackageError::InvalidFilePath(name.to_path_buf()))?;

        // ZIP spec requires forward slashes, not backslashes (Windows uses backslashes)
        let path_as_string = path_as_string.replace('\\', "/");

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            trace!("adding file {path_as_string:?}");
            let mut f = File::open(path)?;
            zip.start_file(
                path_as_string,
                options.last_modified_time(zip_date_from_system_time(f.metadata()?.modified()?)?),
            )?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and "mapname conversion failed" error on unzip
            trace!("adding dir {path_as_string:?}");
            zip.add_directory(path_as_string, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

fn zip_date_from_system_time(time: SystemTime) -> Result<DateTime, PackageError> {
    let date_time = chrono::DateTime::<Utc>::from(time);
    Ok(DateTime::from_date_and_time(
        date_time.year().try_into()?,
        date_time.month().try_into()?,
        date_time.day().try_into()?,
        date_time.hour().try_into()?,
        date_time.minute().try_into()?,
        date_time.second().try_into()?,
    )?)
}

/// Error while packaging a template.
#[derive(Debug, Error)]
pub enum PackageError {
    /// The given source is not a directory.
    #[error("The source is not a directory")]
    SourceIsNotADirectory,
    /// A file path in the template is not valid UTF-8.
    #[error("File path {0} is not valid UTF-8")]
    InvalidFilePath(PathBuf),
    /// IO Error while packaging the template.
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),
    /// Error while compressing the template.
    #[error("zip error: {0}")]
    Zip(#[from] ZipError),
    /// Failed to convert a last modified date to a [`DateTime`].
    #[error("failed to convert last modified dates: {0}")]
    IntConversion(#[from] TryFromIntError),
    /// A last modified date is out of range.
    #[error("failed to convert last modified dates: {0}")]
    DateTimeRange(#[from] DateTimeRangeError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tempfile::TempDir;

    fn manifest() -> &'static str {
        r#"
[package]
name = "test"
version = "0.1.0"
entrypoint = "main.typ"

[tool.oicana]
manifest_version = 1
"#
    }

    fn create_simple_template() -> TempDir {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("typst.toml"), manifest()).unwrap();
        std::fs::write(dir.path().join("main.typ"), "Hello").unwrap();
        dir
    }

    #[test]
    fn packages_simple_template_and_can_unpack() {
        let dir = create_simple_template();
        let manifest = TemplateManifest::from_toml(
            &std::fs::read_to_string(dir.path().join("typst.toml")).unwrap(),
        )
        .unwrap();

        let mut buffer = Cursor::new(Vec::new());
        package(dir.path(), &mut buffer, &manifest).unwrap();

        buffer.set_position(0);
        let mut archive = zip::ZipArchive::new(buffer).unwrap();

        assert!(archive.by_name("main.typ").is_ok());
        assert!(archive.by_name("typst.toml").is_ok());

        let mut main_file = archive.by_name("main.typ").unwrap();
        let mut content = String::new();
        main_file.read_to_string(&mut content).unwrap();
        assert_eq!(content, "Hello");
    }

    #[test]
    fn packages_template_with_subdirectories() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("typst.toml"), manifest()).unwrap();
        std::fs::write(dir.path().join("main.typ"), "Hello").unwrap();
        std::fs::create_dir(dir.path().join("assets")).unwrap();
        std::fs::write(dir.path().join("assets").join("data.json"), "{}").unwrap();

        let manifest = TemplateManifest::from_toml(
            &std::fs::read_to_string(dir.path().join("typst.toml")).unwrap(),
        )
        .unwrap();

        let mut buffer = Cursor::new(Vec::new());
        package(dir.path(), &mut buffer, &manifest).unwrap();

        buffer.set_position(0);
        let mut archive = zip::ZipArchive::new(buffer).unwrap();
        assert!(archive
            .by_name(&Path::new("assets").join("data.json").to_string_lossy())
            .is_ok());
    }

    #[test]
    fn excludes_test_directory() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("typst.toml"), manifest()).unwrap();
        std::fs::write(dir.path().join("main.typ"), "Hello").unwrap();
        std::fs::create_dir(dir.path().join("tests")).unwrap();
        std::fs::write(dir.path().join("tests").join("test.toml"), "").unwrap();

        let manifest = TemplateManifest::from_toml(
            &std::fs::read_to_string(dir.path().join("typst.toml")).unwrap(),
        )
        .unwrap();

        let mut buffer = Cursor::new(Vec::new());
        package(dir.path(), &mut buffer, &manifest).unwrap();

        buffer.set_position(0);
        let archive = zip::ZipArchive::new(buffer).unwrap();
        let file_names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();

        assert!(!file_names.iter().any(|name| name.starts_with("tests")));
        assert!(file_names.contains(&"main.typ".to_string()));
    }

    #[test]
    fn fails_when_source_is_not_directory() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("not_a_dir.txt");
        std::fs::write(&file_path, "content").unwrap();

        let manifest = TemplateManifest::from_toml(manifest()).unwrap();
        let mut buffer = Cursor::new(Vec::new());
        let result = package(&file_path, &mut buffer, &manifest);

        assert!(matches!(result, Err(PackageError::SourceIsNotADirectory)));
    }

    #[test]
    fn fails_when_source_does_not_exist() {
        let manifest = TemplateManifest::from_toml(manifest()).unwrap();
        let mut buffer = Cursor::new(Vec::new());
        let result = package(Path::new("/nonexistent/path"), &mut buffer, &manifest);

        assert!(result.is_err());
    }

    #[test]
    fn packed_template_uses_forward_slashes_in_paths() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("typst.toml"), manifest()).unwrap();
        std::fs::write(dir.path().join("main.typ"), "Main content").unwrap();
        std::fs::create_dir(dir.path().join("lib")).unwrap();
        std::fs::write(dir.path().join("lib").join("utils.typ"), "Utils content").unwrap();
        std::fs::create_dir(dir.path().join("assets")).unwrap();
        std::fs::write(dir.path().join("assets").join("data.json"), "{}").unwrap();
        std::fs::create_dir(dir.path().join("assets").join("images")).unwrap();
        std::fs::write(
            dir.path().join("assets").join("images").join("logo.txt"),
            "Logo placeholder",
        )
        .unwrap();

        let manifest = TemplateManifest::from_toml(
            &std::fs::read_to_string(dir.path().join("typst.toml")).unwrap(),
        )
        .unwrap();

        let mut buffer = Cursor::new(Vec::new());
        package(dir.path(), &mut buffer, &manifest).unwrap();

        buffer.set_position(0);
        let mut archive = zip::ZipArchive::new(buffer).unwrap();

        let mut paths_with_backslashes = Vec::new();
        let mut all_paths = Vec::new();

        for i in 0..archive.len() {
            let entry = archive.by_index(i).unwrap();
            let name = entry.name().to_string();
            all_paths.push(name.clone());

            if name.contains('\\') {
                paths_with_backslashes.push(name);
            }
        }

        assert!(
            paths_with_backslashes.is_empty(),
            "ZIP paths must use forward slashes '/' not backslashes '\\'. Found paths with backslashes: {:?}",
            paths_with_backslashes
        );

        assert!(
            all_paths.iter().any(|p| p == "main.typ"),
            "Expected 'main.typ' in zip"
        );
        assert!(
            all_paths.iter().any(|p| p == "typst.toml"),
            "Expected 'typst.toml' in zip"
        );
        assert!(
            all_paths.iter().any(|p| p == "lib/utils.typ"),
            "Expected 'lib/utils.typ' with forward slash in zip"
        );
        assert!(
            all_paths.iter().any(|p| p == "assets/data.json"),
            "Expected 'assets/data.json' with forward slash in zip"
        );
        assert!(
            all_paths.iter().any(|p| p == "assets/images/logo.txt"),
            "Expected 'assets/images/logo.txt' with forward slashes in zip"
        );
    }
}
