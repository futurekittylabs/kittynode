use crate::infra::file::kittynode_path;
use eyre::Result;
use std::{fs, io::ErrorKind};
use tracing::info;

/// Deletes the Kittynode config directory
pub fn delete_kittynode() -> Result<()> {
    if let Err(e) = fs::remove_dir_all(kittynode_path()?)
        && e.kind() != ErrorKind::NotFound
    {
        return Err(e.into());
    }
    info!("Successfully deleted Kittynode.");
    Ok(())
}
