use crate::domain::package::PackageConfig;
use crate::infra::home::Home;
use eyre::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct PackageConfigStore;

impl PackageConfigStore {
    // path helper

    fn config_file_path_with_base(base: &Path, package_name: &str) -> PathBuf {
        let mut base = PathBuf::from(base);
        base.push("packages");
        base.push(package_name);
        base.push("config.toml");
        base
    }

    pub fn load_from(home: &Home, package_name: &str) -> Result<PackageConfig> {
        let config_path = Self::config_file_path_with_base(home.base(), package_name);
        if !config_path.exists() {
            return Ok(PackageConfig::default());
        }
        let toml_str = fs::read_to_string(config_path)?;
        let config = toml::from_str(&toml_str)?;
        Ok(config)
    }

    pub fn save_to(home: &Home, package_name: &str, config: &PackageConfig) -> Result<()> {
        let config_path = Self::config_file_path_with_base(home.base(), package_name);
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml_str = toml::to_string_pretty(config)?;
        fs::write(config_path, toml_str)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_default_and_roundtrip_save() {
        let dir = tempdir().unwrap();
        let home = Home::from_base(dir.path().join(".kittynode"));

        let name = "ethereum";

        // Missing returns default
        let cfg = PackageConfigStore::load_from(&home, name).expect("load default");
        assert!(cfg.values.is_empty());

        // Save with some values
        let mut to_save = PackageConfig::new();
        to_save.values.insert("network".into(), "holesky".into());
        PackageConfigStore::save_to(&home, name, &to_save).expect("save ok");

        // Load back
        let loaded = PackageConfigStore::load_from(&home, name).expect("load saved");
        assert_eq!(loaded.values.get("network").unwrap(), "holesky");
    }
}
