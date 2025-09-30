use crate::infra::config::ConfigStore;
use eyre::Result;
use tracing::info;

pub fn set_onboarding_completed(completed: bool) -> Result<()> {
    let mut config = ConfigStore::load()?;
    config.onboarding_completed = completed;
    ConfigStore::save_normalized(&mut config)?;
    info!("Set onboarding completed to: {}", completed);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::set_onboarding_completed;
    use crate::application::get_onboarding_completed;
    use crate::application::test_support::ConfigSandbox;

    #[test]
    fn set_onboarding_completed_roundtrip() {
        let _sandbox = ConfigSandbox::new();

        set_onboarding_completed(true).expect("enabling onboarding flag should succeed");
        assert!(
            get_onboarding_completed().expect("reading onboarding flag should succeed"),
            "onboarding flag should be true after enabling"
        );

        set_onboarding_completed(false).expect("disabling onboarding flag should succeed");
        assert!(
            !get_onboarding_completed().expect("reading onboarding flag should succeed"),
            "onboarding flag should be false after disabling"
        );
    }
}
