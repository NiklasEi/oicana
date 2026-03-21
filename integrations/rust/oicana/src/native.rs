use std::path::{Path, PathBuf};

use oicana_files::native::{package_data_dir, NativeTemplate};
use oicana_input::TemplateInputs;
use oicana_template::manifest::TemplateManifest;
use oicana_world::{manifest::OicanaWorldFiles, world::OicanaWorld};

use crate::{Template, TemplateInitializationError};

impl Template<NativeTemplate> {
    /// Initialize the given template
    pub fn init(path: &Path) -> Result<Self, TemplateInitializationError> {
        let files = NativeTemplate::new(
            path,
            package_data_dir().ok_or(TemplateInitializationError::PackageDirectoryNotFound)?,
        );
        let manifest = files.manifest()?;

        let world = OicanaWorld::new(files, TemplateInputs::new(), manifest)?;

        Ok(Template { world })
    }

    /// Reset the file access tracking in preparation for a new compilation.
    ///
    /// After calling this, all files will be re-read from disk on next access.
    /// Files whose content hasn't changed (same fingerprint) will reuse their
    /// processed data, enabling efficient incremental recompilation.
    pub fn reset(&mut self) {
        self.world.files.reset();
        self.world.reset_time();
    }

    /// Return the system paths of all files accessed during the last compilation.
    pub fn dependencies(&self) -> Vec<PathBuf> {
        self.world.files.dependencies()
    }

    /// Create a native template from all required parts
    pub fn from(
        template_root: &Path,
        packages: &Path,
        manifest: TemplateManifest,
    ) -> Result<Self, TemplateInitializationError> {
        let files = NativeTemplate::new(template_root, packages.to_path_buf());
        let world = OicanaWorld::new(files, TemplateInputs::new(), manifest)?;

        Ok(Template { world })
    }
}
