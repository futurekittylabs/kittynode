// Update checker: checks GitHub releases for newer CLI versions
//
// Strategy:
// - Cache successful checks for 24 hours to avoid excessive API calls
// - Timeout after 2 seconds to avoid blocking CLI startup
// - Show banner on stderr when update available (doesn't corrupt stdout)
//
// Return types (Result<Option<String>>):
// - Ok(Some(version)) → Found a CLI release, cache it
// - Ok(None) → No CLI release found (valid response), cache it
// - Err(e) → Network/API/parse error, don't cache, retry next time
//
// Error handling:
// - eprintln!() for expected errors (update available, timeout)
// - error!() for unexpected errors (API failures, filesystem issues)

use eyre::{Result, eyre};
use kittynode_core::api::kittynode_path;
use semver::Version;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::error;

const CACHE_FILE: &str = "cli-version.json";
const CHECK_INTERVAL: u64 = 86400; // 24 hours
const GITHUB_RELEASES_URL: &str = "https://api.github.com/repos/futurekittylabs/kittynode/releases";

#[derive(serde::Serialize, Deserialize)]
struct Cache {
    version: String,
    checked: u64,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
}

fn cache_path() -> Option<PathBuf> {
    kittynode_path().ok().map(|path| path.join(CACHE_FILE))
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn is_newer(latest: &str, current: &str) -> bool {
    let parse_version = |s: &str| {
        Version::parse(
            s.trim_start_matches("kittynode-cli-")
                .trim_start_matches('v'),
        )
        .ok()
    };
    parse_version(latest) > parse_version(current)
}

async fn fetch_latest() -> Result<Option<String>> {
    let client = reqwest::Client::builder()
        .user_agent("kittynode-cli")
        .build()
        .map_err(|e| eyre!("Failed to build HTTP client: {}", e))?;

    let response = client
        .get(GITHUB_RELEASES_URL)
        .send()
        .await
        .map_err(|e| eyre!("Failed to fetch releases from GitHub: {}", e))?;

    let releases = response
        .json::<Vec<Release>>()
        .await
        .map_err(|e| eyre!("Failed to parse GitHub releases response: {}", e))?;

    let result = releases.into_iter().find_map(|r| {
        r.tag_name
            .starts_with("kittynode-cli-")
            .then_some(r.tag_name)
    });

    Ok(result)
}

fn print_banner() {
    eprintln!("✨ Update available, run `kittynode update` to upgrade ✨\n");
}

async fn write_cache(path: &PathBuf, version: String) {
    let cache = Cache {
        version,
        checked: now(),
    };
    if let Some(parent) = path.parent()
        && let Err(e) = tokio::fs::create_dir_all(parent).await
    {
        error!("Failed to create cache directory: {}", e);
    }
    match serde_json::to_string(&cache) {
        Ok(json) => {
            if let Err(e) = tokio::fs::write(path, json).await {
                error!("Failed to write version cache file: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to serialize cache data: {}", e);
        }
    }
}

pub async fn check_and_print_update() {
    let current = env!("CARGO_PKG_VERSION");
    let Some(path) = cache_path() else {
        error!("Failed to resolve kittynode cache path");
        return;
    };

    if let Ok(content) = tokio::fs::read_to_string(&path).await
        && let Ok(cache) = serde_json::from_str::<Cache>(&content)
    {
        if is_newer(&cache.version, current) {
            print_banner();
        }
        if now() - cache.checked < CHECK_INTERVAL {
            return;
        }
    }

    let fetch_result = tokio::time::timeout(Duration::from_secs(2), fetch_latest()).await;

    match fetch_result {
        Ok(Ok(Some(latest))) => {
            // Successfully fetched and found a CLI release, cache it
            write_cache(&path, latest.clone()).await;
            if is_newer(&latest, current) {
                print_banner();
            }
        }
        Ok(Ok(None)) => {
            // Successfully checked but no CLI release found - this is valid, cache it
            write_cache(&path, current.to_string()).await;
        }
        Ok(Err(e)) => {
            // API/network/parse error - log and don't cache
            error!("{:#}", e);
        }
        Err(_) => {
            // Expected: network timeout is a common occurrence, show user-facing message
            eprintln!("Update check timed out. Run `kittynode update` to check manually.\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_comparison_with_newer_version() {
        assert!(is_newer("0.11.1", "0.11.0"));
        assert!(is_newer("kittynode-cli-1.0.0", "kittynode-cli-0.99.0"));
    }

    #[test]
    fn version_comparison_with_older_version() {
        assert!(!is_newer("0.11.0", "0.11.1"));
        assert!(!is_newer("kittynode-cli-0.30.0", "kittynode-cli-0.31.0"));
    }

    #[test]
    fn version_comparison_strips_prefixes() {
        assert!(is_newer("v0.11.1", "0.11.0"));
        assert!(is_newer("kittynode-cli-0.31.0", "0.30.0"));
    }

    #[test]
    fn version_comparison_handles_prerelease() {
        assert!(!is_newer("0.11.0-beta.1", "0.11.0"));
    }

    #[test]
    fn version_comparison_with_equal_versions() {
        assert!(!is_newer("0.11.0", "0.11.0"));
        assert!(!is_newer("kittynode-cli-0.31.0", "kittynode-cli-0.31.0"));
    }

    #[test]
    fn cache_serialization_roundtrip() {
        let cache = Cache {
            version: "kittynode-cli-0.31.0".to_string(),
            checked: 1234567890,
        };

        let json = serde_json::to_string(&cache).expect("serialization should succeed");
        let deserialized: Cache =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert_eq!(cache.version, deserialized.version);
        assert_eq!(cache.checked, deserialized.checked);
    }
}
