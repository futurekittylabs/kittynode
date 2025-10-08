use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

const CACHE_FILE: &str = "cli-version.json";

#[derive(Serialize, Deserialize)]
struct VersionCache {
    latest: String,
}

fn cache_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".kittynode").join(CACHE_FILE))
}

fn is_newer(latest: &str, current: &str) -> bool {
    Version::parse(latest).ok() > Version::parse(current).ok()
}

pub async fn check_and_print_update() {
    let current = env!("CARGO_PKG_VERSION");

    if let Some(path) = cache_path() {
        if let Ok(content) = std::fs::read_to_string(&path)
            && let Ok(cache) = serde_json::from_str::<VersionCache>(&content)
            && is_newer(&cache.latest, current)
        {
            eprintln!("A new Kittynode CLI version v{} is available", cache.latest);
            eprintln!("Run `kittynode update` to upgrade");
            eprintln!();
            return;
        }

        let check = async {
            let mut updater = axoupdater::AxoUpdater::new_for("kittynode-cli");
            updater.set_release_source(axoupdater::ReleaseSource {
                release_type: axoupdater::ReleaseSourceType::GitHub,
                owner: "futurekittylabs".into(),
                name: "kittynode".into(),
                app_name: "kittynode-cli".into(),
            });
            if let Ok(v) = axoupdater::Version::parse(current) {
                let _ = updater.set_current_version(v);
            }
            if let Ok(token) = std::env::var("KITTYNODE_GITHUB_TOKEN") {
                updater.set_github_token(&token);
            }
            match updater.query_new_version().await {
                Ok(Some(v)) => Some(v.to_string()),
                _ => None,
            }
        };

        if let Ok(Some(latest)) = tokio::time::timeout(Duration::from_secs(1), check).await {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string(&VersionCache {
                latest: latest.clone(),
            }) {
                let _ = std::fs::write(&path, json);
            }
            if is_newer(&latest, current) {
                eprintln!("A new Kittynode CLI version v{} is available", latest);
                eprintln!("Run `kittynode update` to upgrade");
                eprintln!();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_version_comparison() {
        assert!(is_newer("0.11.1", "0.11.0"));
        assert!(!is_newer("0.11.0", "0.11.1"));
        assert!(!is_newer("0.11.0-beta.1", "0.11.0"));
    }
}
