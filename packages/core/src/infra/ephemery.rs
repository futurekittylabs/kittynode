use crate::infra::file::kittynode_path;
use eyre::{Context, ContextCompat, Result, eyre};
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::{
    fs,
    io::copy,
    path::{Path, PathBuf},
};
use tar::Archive;
use tracing::{info, warn};

pub const EPHEMERY_NETWORK_NAME: &str = "ephemery";
pub const EPHEMERY_CHECKPOINT_URLS: &[&str] = &[
    "https://checkpoint-sync.ephemery.ethpandaops.io/",
    "https://checkpointz.bordel.wtf/",
    "https://ephemery.beaconstate.ethstaker.cc/",
];

const RELEASE_API_URL: &str =
    "https://api.github.com/repos/ephemery-testnet/ephemery-genesis/releases/latest";
const NETWORK_ARCHIVE_NAME: &str = "network-config.tar.gz";
const USER_AGENT: &str = "kittynode";

#[derive(Clone)]
pub struct EphemeryConfig {
    pub tag: String,
    pub metadata_dir: PathBuf,
    pub execution_bootnodes: Vec<String>,
    pub consensus_bootnodes: Vec<String>,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub fn ensure_ephemery_config() -> Result<EphemeryConfig> {
    let base_dir = kittynode_path()?
        .join("networks")
        .join(EPHEMERY_NETWORK_NAME);
    fs::create_dir_all(&base_dir).wrap_err("Failed to prepare Ephemery network directory")?;

    let current_dir = base_dir.join("current");
    let tag_file = base_dir.join("current_tag");
    let mut active_tag = fs::read_to_string(&tag_file)
        .ok()
        .map(|s| s.trim().to_string());

    let latest_release = fetch_latest_release();

    match latest_release {
        Ok((latest_tag, archive_url)) => {
            let needs_fetch = active_tag
                .as_ref()
                .map(|tag| tag != &latest_tag)
                .unwrap_or(true)
                || !current_dir.exists();
            if needs_fetch {
                info!("Updating Ephemery network configuration to {latest_tag}");
                download_and_install(&base_dir, &archive_url)?;
                fs::write(&tag_file, &latest_tag)
                    .wrap_err("Failed to record Ephemery release tag")?;
            }
            active_tag = Some(latest_tag);
        }
        Err(error) => {
            if active_tag.is_some() && current_dir.exists() {
                warn!(
                    "Failed to check for Ephemery updates, continuing with cached config: {error}"
                );
            } else {
                return Err(error.wrap_err(
                    "Unable to download Ephemery configuration and no cached copy is available",
                ));
            }
        }
    }

    let tag = active_tag.context("Ephemery configuration tag is missing")?;
    let metadata_dir = current_dir.join("metadata");
    if !metadata_dir.exists() {
        return Err(eyre!(
            "Ephemery metadata directory missing at {}",
            metadata_dir.display()
        ));
    }

    let execution_bootnodes = read_lines(metadata_dir.join("enodes.txt"))
        .wrap_err("Failed to load execution bootnodes")?;
    let consensus_bootnodes = read_lines(metadata_dir.join("bootstrap_nodes.txt"))
        .wrap_err("Failed to load consensus bootnodes")?;

    Ok(EphemeryConfig {
        tag,
        metadata_dir,
        execution_bootnodes,
        consensus_bootnodes,
    })
}

fn fetch_latest_release() -> Result<(String, String)> {
    let response = ureq::get(RELEASE_API_URL)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|error| eyre!("Failed to fetch Ephemery release metadata: {error}"))?;

    let release: Release = response
        .into_json()
        .map_err(|error| eyre!("Failed to decode Ephemery release metadata: {error}"))?;

    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name == NETWORK_ARCHIVE_NAME)
        .context("Ephemery release missing network archive")?;

    Ok((release.tag_name, asset.browser_download_url))
}

fn download_and_install(base_dir: &Path, archive_url: &str) -> Result<()> {
    let staging_path = base_dir.join("current.staging");
    if staging_path.exists() {
        fs::remove_dir_all(&staging_path).wrap_err("Failed to reset Ephemery staging directory")?;
    }
    fs::create_dir_all(&staging_path).wrap_err("Failed to create Ephemery staging directory")?;
    let archive_path = staging_path.join(NETWORK_ARCHIVE_NAME);

    download_archive(archive_url, &archive_path)?;

    let archive_file =
        fs::File::open(&archive_path).wrap_err("Failed to open downloaded Ephemery archive")?;
    let tar = GzDecoder::new(archive_file);
    let mut archive = Archive::new(tar);
    archive
        .unpack(&staging_path)
        .wrap_err("Failed to unpack Ephemery archive")?;
    let _ = fs::remove_file(&archive_path);

    let new_dir = base_dir.join("current.new");
    if new_dir.exists() {
        fs::remove_dir_all(&new_dir)
            .wrap_err("Failed to remove existing Ephemery staging directory")?;
    }
    fs::rename(&staging_path, &new_dir).wrap_err("Failed to stage Ephemery configuration")?;

    let metadata_dir = new_dir.join("metadata");
    if !metadata_dir.exists() {
        return Err(eyre!(
            "Downloaded Ephemery archive missing metadata directory at {}",
            metadata_dir.display()
        ));
    }

    let current_dir = base_dir.join("current");
    let backup_dir = base_dir.join("current.backup");
    if backup_dir.exists() {
        fs::remove_dir_all(&backup_dir)
            .wrap_err("Failed to remove previous Ephemery backup directory")?;
    }
    if current_dir.exists() {
        fs::rename(&current_dir, &backup_dir)
            .wrap_err("Failed to move previous Ephemery configuration to backup")?;
    }
    if let Err(error) = fs::rename(&new_dir, &current_dir) {
        // Attempt to restore the backup before returning an error.
        if backup_dir.exists() {
            let _ = fs::rename(&backup_dir, &current_dir);
        }
        return Err(error).wrap_err("Failed to promote Ephemery configuration");
    }
    if backup_dir.exists() {
        let _ = fs::remove_dir_all(&backup_dir);
    }

    Ok(())
}

fn download_archive(url: &str, destination: &Path) -> Result<()> {
    let response = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|error| eyre!("Failed to download Ephemery archive: {error}"))?;

    let mut reader = response.into_reader();
    let mut file = fs::File::create(destination).wrap_err("Failed to create archive file")?;
    copy(&mut reader, &mut file).wrap_err("Failed to save Ephemery archive")?;
    Ok(())
}

fn read_lines(path: PathBuf) -> Result<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content =
        fs::read_to_string(&path).wrap_err_with(|| format!("Failed to read {}", path.display()))?;
    Ok(content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect())
}
