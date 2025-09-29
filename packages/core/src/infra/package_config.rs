use crate::domain::package::PackageConfig;
use crate::infra::file::kittynode_path;
use eyre::Result;
use std::{fs, path::PathBuf};

pub struct PackageConfigStore;

impl PackageConfigStore {
    pub fn load(package_name: &str) -> Result<PackageConfig> {
        let config_path = Self::config_file_path(package_name)?;
        if !config_path.exists() {
            return Ok(PackageConfig::default());
        }
        let toml_str = fs::read_to_string(config_path)?;
        let config = toml::from_str(&toml_str)?;
        Ok(config)
    }

    pub fn save(package_name: &str, config: &PackageConfig) -> Result<()> {
        let config_path = Self::config_file_path(package_name)?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml_str = toml::to_string_pretty(config)?;
        fs::write(config_path, toml_str)?;
        Ok(())
    }

    fn config_file_path(package_name: &str) -> Result<PathBuf> {
        let path = format!(
            "{}/packages/{}/config.toml",
            kittynode_path()?.display(),
            package_name
        );
        Ok(PathBuf::from(path))
    }
}

#[cfg(test)]
mod tests {
    use super::PackageConfigStore;
    use crate::domain::package::PackageConfig;
    use std::env;
    use std::ffi::OsString;
    use std::fs;
    use std::path::Path;
    use std::sync::{LazyLock, Mutex};
    use tempfile::tempdir;

    static HOME_GUARD: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    struct HomeEnvGuard {
        original: Option<OsString>,
    }

    impl Drop for HomeEnvGuard {
        fn drop(&mut self) {
            if let Some(original) = self.original.take() {
                // SAFETY: Restoring HOME to its previous value for test isolation.
                unsafe { env::set_var("HOME", original) };
            } else {
                // SAFETY: Tests previously set HOME, so removing it returns to the prior state.
                unsafe { env::remove_var("HOME") };
            }
        }
    }

    fn with_temp_home<F: FnOnce(&Path)>(test: F) {
        let _guard = HOME_GUARD.lock().expect("home guard poisoned");
        let temp_dir = tempdir().expect("create temp home");
        let home_dir = temp_dir.path().to_path_buf();
        let original_home = env::var_os("HOME");

        // SAFETY: HOME is set to a test directory and restored after the test completes.
        unsafe { env::set_var("HOME", &home_dir) };
        let _restore = HomeEnvGuard {
            original: original_home,
        };

        test(&home_dir);
    }

    #[test]
    fn load_returns_default_when_config_missing() {
        with_temp_home(|_| {
            let config =
                PackageConfigStore::load("missing-package").expect("load missing package config");

            assert!(
                config.values.is_empty(),
                "expected default config to have no overrides"
            );
        });
    }

    #[test]
    fn save_and_load_round_trip_preserves_values() {
        with_temp_home(|home| {
            let mut config = PackageConfig::new();
            config
                .values
                .insert("rpcUrl".to_string(), "https://example.com".to_string());

            PackageConfigStore::save("sample", &config).expect("save config");

            let config_path = home.join(".kittynode/packages/sample/config.toml");
            let serialized = fs::read_to_string(&config_path).expect("read serialized config");
            assert!(serialized.contains("rpcUrl = \"https://example.com\""));

            let loaded = PackageConfigStore::load("sample").expect("load saved config");
            assert_eq!(
                loaded.values.get("rpcUrl").map(String::as_str),
                Some("https://example.com")
            );
        });
    }
}
