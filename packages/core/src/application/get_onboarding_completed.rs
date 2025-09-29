use crate::infra::config::ConfigStore;
use eyre::Result;
use tracing::info;

pub fn get_onboarding_completed() -> Result<bool> {
    let config = ConfigStore::load()?;
    info!(
        "Retrieved onboarding completed status: {}",
        config.onboarding_completed
    );
    Ok(config.onboarding_completed)
}

#[cfg(test)]
mod tests {
    use super::get_onboarding_completed;
    use crate::application::set_onboarding_completed;
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
    fn returns_false_when_config_missing() {
        with_temp_home(|_| {
            let status = get_onboarding_completed().expect("should load default");
            assert!(!status);
        });
    }

    #[test]
    fn reads_persisted_onboarding_flag() {
        with_temp_home(|_| {
            set_onboarding_completed(true).expect("set flag");
            let status = get_onboarding_completed().expect("should read flag");
            assert!(status);
        });
    }
}
