use anyhow::{bail, Context, Result};

pub fn update() -> Result<()> {
    // receipts get installed by cargo-dist at ~/.config/oicana_cli/.
    let mut updater = axoupdater::AxoUpdater::new_for("oicana_cli");

    // Oicana is in alpha, so opt in to prereleases for now.
    updater.configure_version_specifier(axoupdater::UpdateRequest::LatestMaybePrerelease);

    updater.load_receipt().context(
        "No install receipt found. \
         `oicana update` only works when oicana was installed via the official \
         shell or PowerShell installer. Use your package manager to update instead.",
    )?;

    if !updater
        .check_receipt_is_for_this_executable()
        .context("failed to verify install receipt")?
    {
        bail!(
            "The install receipt does not match this binary. \
             This oicana was likely installed via a different method \
             (package manager, `cargo install`, etc.). Update it the same way."
        );
    }

    match updater.run_sync().context("failed to run self-update")? {
        Some(result) => {
            println!("Updated Oicana to {}.", result.new_version);
        }
        None => {
            println!("Oicana is already up to date.");
        }
    }

    Ok(())
}
