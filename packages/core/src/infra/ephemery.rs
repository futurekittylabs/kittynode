use crate::infra::file::kittynode_path;
use eyre::{Context, ContextCompat, Result, eyre};
use flate2::read::GzDecoder;
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

const RELEASE_LATEST_PAGE_URL: &str =
    "https://github.com/ephemery-testnet/ephemery-genesis/releases/latest";
const NETWORK_ARCHIVE_NAME: &str = "network-config.tar.gz";
const USER_AGENT: &str = "kittynode";

#[derive(Clone)]
pub struct EphemeryConfig {
    pub tag: String,
    pub metadata_dir: PathBuf,
    pub execution_bootnodes: Vec<String>,
    pub consensus_bootnodes: Vec<String>,
}

pub fn ensure_ephemery_config() -> Result<EphemeryConfig> {
    let base_dir = ephemery_dir(&kittynode_path()?);
    ensure_ephemery_config_with(&base_dir, fetch_latest_release, download_and_install)
}

fn ephemery_dir(config_root: &Path) -> PathBuf {
    config_root
        .join("packages")
        .join("ethereum")
        .join("networks")
        .join(EPHEMERY_NETWORK_NAME)
}

fn ensure_ephemery_config_with(
    base_dir: &Path,
    fetch_latest_release: fn() -> Result<(String, String)>,
    download_and_install: fn(&Path, &str) -> Result<()>,
) -> Result<EphemeryConfig> {
    fs::create_dir_all(base_dir).wrap_err("Failed to prepare Ephemery network directory")?;

    let current_dir = base_dir.join("current");
    let metadata_dir = current_dir.join("metadata");
    let tag_file = base_dir.join("current_tag");
    let mut active_tag = fs::read_to_string(&tag_file)
        .ok()
        .map(|s| s.trim().to_string());
    let has_cached_layout = current_dir.exists() && metadata_dir.exists();

    let latest_release = fetch_latest_release();

    match latest_release {
        Ok((latest_tag, archive_url)) => {
            // Refresh when the cached tag is missing, outdated, or the on-disk layout is incomplete.
            let needs_fetch = active_tag
                .as_ref()
                .map(|tag| tag != &latest_tag)
                .unwrap_or(!has_cached_layout)
                || !current_dir.exists()
                || !metadata_dir.exists();
            if needs_fetch {
                info!("Updating Ephemery network configuration to {latest_tag}");
                download_and_install(base_dir, &archive_url)?;
                fs::write(&tag_file, &latest_tag)
                    .wrap_err("Failed to record Ephemery release tag")?;
            }
            active_tag = Some(latest_tag);
        }
        Err(error) => {
            // Allow offline operation when we already have a complete cache on disk.
            if has_cached_layout {
                warn!(
                    "Failed to check for Ephemery updates, continuing with cached config: {error}"
                );
                if active_tag.is_none() {
                    active_tag = Some("cached-offline".to_string());
                }
            } else {
                return Err(error.wrap_err(
                    "Unable to download Ephemery configuration and no cached copy is available",
                ));
            }
        }
    }

    let tag = active_tag.context("Ephemery configuration tag is missing")?;
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
    let response = ureq::get(RELEASE_LATEST_PAGE_URL)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|error| eyre!("Failed to resolve Ephemery latest release page: {error}"))?;

    let location = response.get_url();
    let tag = parse_tag_from_location(location)?;
    let download_url = format!(
        "https://github.com/ephemery-testnet/ephemery-genesis/releases/download/{tag}/{NETWORK_ARCHIVE_NAME}"
    );
    Ok((tag, download_url))
}

fn parse_tag_from_location(location: &str) -> Result<String> {
    let after_tag = location
        .split("/tag/")
        .nth(1)
        .context("Unable to locate /tag/ segment in Ephemery release redirect")?;
    let trimmed = after_tag.trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(eyre!(
            "Ephemery release redirect contained empty tag segment: {location}"
        ));
    }
    Ok(trimmed.to_string())
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
        // Restore the previous configuration if promotion fails midflight.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn parse_tag_from_location_extracts_tag() {
        let tag = parse_tag_from_location(
            "https://github.com/ephemery-testnet/ephemery-genesis/releases/tag/v1.2.3",
        )
        .expect("tag should parse");
        assert_eq!(tag, "v1.2.3");
    }

    #[test]
    fn parse_tag_from_location_rejects_missing_segment() {
        let err =
            parse_tag_from_location("https://example.com/no-tag-here").expect_err("expected error");
        assert!(err.to_string().contains("/tag/"));
    }

    #[test]
    fn ensure_ephemery_config_works_offline_with_cached_layout() {
        let temp = tempdir().expect("temp dir");
        let base_dir = temp.path().join("ephemery");
        let metadata_dir = base_dir.join("current").join("metadata");
        fs::create_dir_all(&metadata_dir).expect("create metadata dir");
        fs::write(
            metadata_dir.join("enodes.txt"),
            "enode://abc\n\nenode://def\n",
        )
        .expect("write enodes");
        fs::write(metadata_dir.join("bootstrap_nodes.txt"), "node1\nnode2\n")
            .expect("write bootstrap nodes");

        let fetch_fails = || Err(eyre!("offline"));

        let config = ensure_ephemery_config_with(&base_dir, fetch_fails, download_and_install)
            .expect("should succeed offline with cached layout");

        assert_eq!(config.tag, "cached-offline");
        assert_eq!(
            config.execution_bootnodes,
            vec!["enode://abc", "enode://def"]
        );
        assert_eq!(config.consensus_bootnodes, vec!["node1", "node2"]);
    }

    #[test]
    fn ensure_ephemery_config_refreshes_when_latest_tag_differs() {
        let temp = tempdir().expect("temp dir");
        let base_dir = temp.path().join("ephemery");

        fn fetch_latest() -> Result<(String, String)> {
            Ok((
                "v9.9.9".to_string(),
                "https://example.invalid/archive.tar.gz".to_string(),
            ))
        }

        fn install_stub(base_dir: &Path, _url: &str) -> Result<()> {
            let metadata_dir = base_dir.join("current").join("metadata");
            fs::create_dir_all(&metadata_dir)?;
            fs::write(metadata_dir.join("enodes.txt"), "enode://aaa\n")?;
            fs::write(metadata_dir.join("bootstrap_nodes.txt"), "bbb\n")?;
            Ok(())
        }

        let config = ensure_ephemery_config_with(&base_dir, fetch_latest, install_stub)
            .expect("should refresh");

        assert_eq!(config.tag, "v9.9.9");
        assert_eq!(config.execution_bootnodes, vec!["enode://aaa"]);
        assert_eq!(config.consensus_bootnodes, vec!["bbb"]);
        let tag_file = base_dir.join("current_tag");
        assert!(tag_file.exists(), "tag file should be written");
        assert_eq!(
            fs::read_to_string(tag_file).expect("tag readable").trim(),
            "v9.9.9"
        );
    }
}
