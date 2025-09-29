use crate::infra::config::ConfigStore;
use eyre::Result;
use tracing::info;

/// Enables or disables automatic Docker startup at application launch
pub fn set_auto_start_docker(enabled: bool) -> Result<()> {
    let mut config = ConfigStore::load()?;
    config.auto_start_docker = enabled;
    ConfigStore::save_normalized(&mut config)?;
    info!("Set auto start docker to: {}", enabled);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::config::ConfigStore;
    use std::env;
    use tempfile::tempdir;

    fn with_temp_home<F, R>(f: F) -> R
    where
        F: FnOnce(&std::path::Path) -> R,
    {
        let _lock = crate::ENV_GUARD.lock().expect("lock poisoned");
        let original_home = env::var_os("HOME");
        let temp = tempdir().expect("tempdir failed");
        // Newer toolchains mark environment mutation unsafe; scope it with a guard.
        unsafe {
            env::set_var("HOME", temp.path());
        }
        let result = f(temp.path());
        match original_home {
            Some(value) => unsafe {
                env::set_var("HOME", value);
            },
            None => unsafe {
                env::remove_var("HOME");
            },
        }
        result
    }

    #[test]
    fn persists_auto_start_toggle() {
        with_temp_home(|_| {
            set_auto_start_docker(true).expect("toggle on");
            let config = ConfigStore::load().expect("load config");
            assert!(config.auto_start_docker);
            assert!(!config.onboarding_completed);

            set_auto_start_docker(false).expect("toggle off");
            let config = ConfigStore::load().expect("reload config");
            assert!(!config.auto_start_docker);
            assert!(!config.onboarding_completed);
        });
    }
}
