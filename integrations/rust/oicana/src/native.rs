use std::path::{Path, PathBuf};

use oicana_files::native::{package_data_dir, NativeTemplate};
use oicana_input::TemplateInputs;
use oicana_template::manifest::TemplateManifest;
use oicana_world::{fonts::FontSource, manifest::OicanaWorldFiles, world::OicanaWorld};

use crate::{Template, TemplateInitializationError};

impl Template<NativeTemplate> {
    /// Initialize the given template
    pub fn init(path: &Path) -> Result<Self, TemplateInitializationError> {
        Self::init_with_fonts(path, &[])
    }

    /// Initialize the given template with additional fonts.
    ///
    /// The fonts are available to the template on top of the ones it packs
    /// itself, but do not become part of it: a template relying on them only
    /// renders where an equivalent font is provided. Declare the families under
    /// `tool.oicana.fonts.require` in the manifest to have that checked here.
    pub fn init_with_fonts(
        path: &Path,
        fonts: &[FontSource],
    ) -> Result<Self, TemplateInitializationError> {
        let files = NativeTemplate::new(
            path,
            package_data_dir().ok_or(TemplateInitializationError::PackageDirectoryNotFound)?,
        );
        let manifest = files.manifest()?;

        let world = OicanaWorld::new_with_fonts(files, TemplateInputs::new(), manifest, fonts)?;

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
        Self::from_with_fonts(template_root, packages, manifest, &[])
    }

    /// Create a native template from all required parts, with additional fonts
    pub fn from_with_fonts(
        template_root: &Path,
        packages: &Path,
        manifest: TemplateManifest,
        fonts: &[FontSource],
    ) -> Result<Self, TemplateInitializationError> {
        let files = NativeTemplate::new(template_root, packages.to_path_buf());
        let world = OicanaWorld::new_with_fonts(files, TemplateInputs::new(), manifest, fonts)?;

        Ok(Template { world })
    }
}
