use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OperationalMode {
    Local,
    Remote,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationalState {
    pub mode: OperationalMode,
    pub docker_running: bool,
    pub can_install: bool,
    pub can_manage: bool,
    pub diagnostics: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operational_mode_serializes_to_camel_case() {
        let local = OperationalMode::Local;
        let remote = OperationalMode::Remote;

        let local_json = serde_json::to_string(&local).unwrap();
        let remote_json = serde_json::to_string(&remote).unwrap();

        assert_eq!(local_json, r#""local""#);
        assert_eq!(remote_json, r#""remote""#);
    }

    #[test]
    fn operational_mode_deserializes_from_camel_case() {
        let local: OperationalMode = serde_json::from_str(r#""local""#).unwrap();
        let remote: OperationalMode = serde_json::from_str(r#""remote""#).unwrap();

        assert_eq!(local, OperationalMode::Local);
        assert_eq!(remote, OperationalMode::Remote);
    }

    #[test]
    fn operational_state_serializes_with_camel_case_fields() {
        let state = OperationalState {
            mode: OperationalMode::Local,
            docker_running: true,
            can_install: true,
            can_manage: false,
            diagnostics: vec!["test diagnostic".to_string()],
        };

        let json = serde_json::to_value(&state).unwrap();

        assert_eq!(json["mode"], "local");
        assert_eq!(json["dockerRunning"], true);
        assert_eq!(json["canInstall"], true);
        assert_eq!(json["canManage"], false);
        assert_eq!(json["diagnostics"][0], "test diagnostic");
    }

    #[test]
    fn operational_state_deserializes_from_camel_case_json() {
        let json = r#"{
            "mode": "remote",
            "dockerRunning": false,
            "canInstall": false,
            "canManage": true,
            "diagnostics": ["error 1", "error 2"]
        }"#;

        let state: OperationalState = serde_json::from_str(json).unwrap();

        assert_eq!(state.mode, OperationalMode::Remote);
        assert!(!state.docker_running);
        assert!(!state.can_install);
        assert!(state.can_manage);
        assert_eq!(state.diagnostics.len(), 2);
        assert_eq!(state.diagnostics[0], "error 1");
        assert_eq!(state.diagnostics[1], "error 2");
    }

    #[test]
    fn operational_state_roundtrips_through_json() {
        let original = OperationalState {
            mode: OperationalMode::Local,
            docker_running: true,
            can_install: false,
            can_manage: true,
            diagnostics: vec![],
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: OperationalState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.mode, original.mode);
        assert_eq!(deserialized.docker_running, original.docker_running);
        assert_eq!(deserialized.can_install, original.can_install);
        assert_eq!(deserialized.can_manage, original.can_manage);
        assert_eq!(deserialized.diagnostics.len(), 0);
    }
}
