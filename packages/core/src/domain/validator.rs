use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ValidatorRuntimeStatus {
    Disabled,
    NotInstalled,
    Stopped,
    Running,
}

impl ValidatorRuntimeStatus {
    pub(crate) fn classify(
        validator_enabled: bool,
        container_exists: bool,
        container_running: bool,
    ) -> Self {
        if !validator_enabled {
            return Self::Disabled;
        }
        if !container_exists {
            return Self::NotInstalled;
        }
        if container_running {
            Self::Running
        } else {
            Self::Stopped
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ValidatorRuntimeStatus;

    #[test]
    fn classification_reflects_enabled_flag() {
        let result = ValidatorRuntimeStatus::classify(false, true, true);
        assert_eq!(result, ValidatorRuntimeStatus::Disabled);
    }

    #[test]
    fn classification_tracks_missing_container() {
        let result = ValidatorRuntimeStatus::classify(true, false, false);
        assert_eq!(result, ValidatorRuntimeStatus::NotInstalled);
    }

    #[test]
    fn classification_distinguishes_running_and_stopped() {
        let running = ValidatorRuntimeStatus::classify(true, true, true);
        assert_eq!(running, ValidatorRuntimeStatus::Running);

        let stopped = ValidatorRuntimeStatus::classify(true, true, false);
        assert_eq!(stopped, ValidatorRuntimeStatus::Stopped);
    }
}
