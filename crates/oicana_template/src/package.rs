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
    let it = walk_dir
        .into_iter()
        .filter_entry(|entry| manifest.should_path_be_packed(entry.path()));

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
