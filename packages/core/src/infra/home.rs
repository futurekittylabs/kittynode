use crate::infra::file::kittynode_path;
use eyre::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Home {
    base: PathBuf,
}

impl Home {
    pub fn try_default() -> Result<Self> {
        Ok(Self {
            base: kittynode_path()?,
        })
    }

    pub fn from_base<P: Into<PathBuf>>(base: P) -> Self {
        Self { base: base.into() }
    }

    pub fn base(&self) -> &Path {
        &self.base
    }

    pub fn config_path(&self) -> PathBuf {
        let mut p = self.base.clone();
        p.push("config.toml");
        p
    }

    pub fn package_config_path(&self, package_name: &str) -> PathBuf {
        let mut p = self.base.clone();
        p.push("packages");
        p.push(package_name);
        p.push("config.toml");
        p
    }

    pub fn jwt_path(&self) -> PathBuf {
        let mut p = self.base.clone();
        p.push("jwt.hex");
        p
    }

    pub fn lighthouse_dir(&self) -> PathBuf {
        let mut p = self.base.clone();
        p.push(".lighthouse");
        p
    }
}
