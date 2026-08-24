use clap::Args;
use log::{debug, info};
use oicana::fonts::{font_files_at, FontSource};
use std::path::PathBuf;

/// Environment variable listing font directories, separated like `PATH`.
const FONT_PATH_ENV: &str = "OICANA_FONT_PATHS";

#[derive(Debug, Args, Default, Clone)]
pub struct FontArgs {
    #[arg(
        long,
        help = "Font file to make available as if a host had registered it. Repeat for several files",
        value_name = "FILE"
    )]
    font_file: Vec<PathBuf>,
    #[arg(
        long,
        help = "Directory to search for fonts to make available as if a host had registered them. Repeat for several directories",
        value_name = "DIR"
    )]
    font_path: Vec<PathBuf>,
}

impl FontArgs {
    /// Load every font given on the command line or via `OICANA_FONT_PATHS`.
    ///
    /// Unreadable files and files that hold no font are skipped with a warning,
    /// so pointing at a system font directory is safe.
    pub fn load(&self) -> Vec<FontSource> {
        let mut files = self.font_file.clone();
        for dir in self.font_path.iter().cloned().chain(env_font_paths()) {
            files.extend(font_files_at(&dir));
        }

        let sources: Vec<FontSource> = files.iter().filter_map(FontSource::from_path).collect();
        if !sources.is_empty() {
            let faces: usize = sources.iter().map(|source| source.families().len()).sum();
            info!(
                "Providing {faces} host font face(s) from {} file(s). These are not packed with the template.",
                sources.len()
            );
            for source in &sources {
                debug!(
                    "Host font {:?}: {}",
                    source.path().unwrap_or(std::path::Path::new("<memory>")),
                    source.families().join(", ")
                );
            }
        }
        sources
    }
}

fn env_font_paths() -> Vec<PathBuf> {
    let Ok(paths) = std::env::var(FONT_PATH_ENV) else {
        return vec![];
    };
    std::env::split_paths(&paths).collect()
}
