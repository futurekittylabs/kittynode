use crate::domain::config::Config;
use crate::infra::config::ConfigStore;
use eyre::{Result, eyre};
use url::Url;

fn validate_server_url(endpoint: &str) -> Result<()> {
    if endpoint.is_empty() {
        return Ok(());
    }

    let parsed = Url::parse(endpoint).map_err(|e| eyre!("invalid server URL '{endpoint}': {e}"))?;

    match parsed.scheme() {
        "http" | "https" => {}
        other => {
            return Err(eyre!(
                "invalid server URL '{endpoint}': unsupported scheme '{other}' (expected http or https)"
            ));
        }
    }

    if parsed.host_str().is_none() {
        return Err(eyre!("invalid server URL '{endpoint}': missing host"));
    }

    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(eyre!(
            "invalid server URL '{endpoint}': credentials are not supported"
        ));
    }

    Ok(())
}

fn apply_server_url(config: &mut Config, endpoint: &str) -> Result<()> {
    let trimmed = endpoint.trim();

    validate_server_url(trimmed)?;

    if trimmed.is_empty() {
        config.server_url.clear();
        config.has_remote_server = false;
    } else {
        let normalized = trimmed.to_string();
        config.server_url = normalized.clone();
        config.last_server_url = normalized;
        config.has_remote_server = true;
    }

    Ok(())
}

pub fn set_server_url(endpoint: String) -> Result<()> {
    let mut config = ConfigStore::load()?;
    apply_server_url(&mut config, &endpoint)?;
    ConfigStore::save_normalized(&mut config)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{apply_server_url, set_server_url, validate_server_url};
    use crate::domain::config::Config;
    use crate::infra::{config::ConfigStore, file::with_kittynode_path_override};
    use tempfile::tempdir;

    #[test]
    fn validate_allows_empty_endpoint() {
        assert!(validate_server_url("").is_ok());
    }

    #[test]
    fn validate_rejects_invalid_scheme() {
        let err =
            validate_server_url("ftp://example.com").expect_err("expected validation failure");
        assert!(err.to_string().contains("unsupported scheme"));
    }

    #[test]
    fn apply_sets_server_url_and_last() {
        let mut config = Config::default();
        apply_server_url(&mut config, " http://example.com ").expect("apply should succeed");

        assert_eq!(config.server_url, "http://example.com");
        assert_eq!(config.last_server_url, "http://example.com");
        assert!(config.has_remote_server);
    }

    #[test]
    fn apply_clears_server_but_preserves_last() {
        let mut config = Config::default();
        apply_server_url(&mut config, "http://example.com").expect("initial apply should succeed");
        apply_server_url(&mut config, "").expect("clearing should succeed");

        assert_eq!(config.server_url, "");
        assert_eq!(config.last_server_url, "http://example.com");
        assert!(!config.has_remote_server);
    }

    #[test]
    fn apply_preserves_trailing_slash() {
        let mut config = Config::default();
        apply_server_url(&mut config, "https://example.com/ ").expect("apply should succeed");

        assert_eq!(config.server_url, "https://example.com/");
        assert_eq!(config.last_server_url, "https://example.com/");
        assert!(config.has_remote_server);
    }

    #[test]
    fn apply_does_not_mutate_on_validation_error() {
        let mut config = Config::default();
        let _err =
            apply_server_url(&mut config, "not a url").expect_err("expected validation failure");

        assert_eq!(config.server_url, "");
        assert_eq!(config.last_server_url, "");
        assert!(!config.has_remote_server);
    }

    #[test]
    fn set_server_url_persists_normalized_endpoint() {
        let temp = tempdir().unwrap();

        with_kittynode_path_override(temp.path(), || {
            set_server_url(" https://example.com ".into()).expect("setting url should succeed");

            let persisted = ConfigStore::load().expect("config should load");
            assert_eq!(persisted.server_url, "https://example.com");
            assert_eq!(persisted.last_server_url, "https://example.com");
            assert!(persisted.has_remote_server);
        });
    }

    #[test]
    fn set_server_url_allows_clearing_endpoint() {
        let temp = tempdir().unwrap();

        with_kittynode_path_override(temp.path(), || {
            set_server_url("https://example.com".into()).expect("initial set should succeed");
            set_server_url("".into()).expect("clearing should succeed");

            let persisted = ConfigStore::load().expect("config should load");
            assert_eq!(persisted.server_url, "");
            assert_eq!(persisted.last_server_url, "https://example.com");
            assert!(!persisted.has_remote_server);
        });
    }

    #[test]
    fn set_server_url_does_not_persist_on_failure() {
        let temp = tempdir().unwrap();

        with_kittynode_path_override(temp.path(), || {
            let mut existing = Config {
                server_url: "https://example.com".into(),
                last_server_url: "https://example.com".into(),
                has_remote_server: true,
                ..Config::default()
            };
            ConfigStore::save_normalized(&mut existing).expect("initial save should succeed");

            let err = set_server_url("notaurl".into()).expect_err("should fail to set url");
            assert!(err.to_string().contains("invalid server URL"));

            let persisted = ConfigStore::load().expect("config should load");
            assert_eq!(persisted.server_url, "https://example.com");
            assert_eq!(persisted.last_server_url, "https://example.com");
            assert!(persisted.has_remote_server);
        });
    }
}
