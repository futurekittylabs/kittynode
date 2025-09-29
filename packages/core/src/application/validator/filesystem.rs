use super::ports::ValidatorFilesystem;
use eyre::{Context, Result};
use std::path::Path;

pub(super) fn ensure_parent_secure<F: ValidatorFilesystem>(
    path: &Path,
    filesystem: &F,
    context_label: &str,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        filesystem
            .ensure_secure_directory(parent)
            .with_context(|| format!("{context_label}: {}", parent.display()))?;
    }
    Ok(())
}
