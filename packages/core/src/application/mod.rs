pub mod add_capability;
pub mod delete_kittynode;
pub mod delete_package;
pub mod get_capabilities;
pub mod get_config;
pub mod get_container_logs;
pub mod get_installed_packages;
pub mod get_onboarding_completed;
pub mod get_operational_state;
pub mod get_package_config;
pub mod get_package_runtime_state;
pub mod get_packages;
pub mod get_packages_runtime_state;
pub mod get_server_url;
pub mod get_system_info;
pub mod init_kittynode;
pub mod install_package;
pub mod is_docker_running;
pub mod remove_capability;
pub mod resume_package;
pub mod set_auto_start_docker;
pub mod set_onboarding_completed;
pub mod set_server_url;
pub mod start_docker;
pub mod start_docker_if_needed;
pub mod stop_package;
pub mod update_package_config;
pub mod validator;
pub mod web_service;

pub use add_capability::add_capability;
pub use delete_kittynode::delete_kittynode;
pub use delete_package::delete_package;
pub use get_capabilities::get_capabilities;
pub use get_config::get_config;
pub use get_container_logs::get_container_logs;
pub use get_installed_packages::get_installed_packages;
pub use get_onboarding_completed::get_onboarding_completed;
pub use get_operational_state::get_operational_state;
pub use get_package_config::get_package_config;
pub use get_package_runtime_state::get_package_runtime_state;
pub use get_packages::get_packages;
pub use get_packages_runtime_state::get_packages_runtime_state;
pub use get_server_url::get_server_url;
pub use get_system_info::get_system_info;
pub use init_kittynode::init_kittynode;
pub use install_package::install_package;
pub use is_docker_running::is_docker_running;
pub use remove_capability::remove_capability;
pub use resume_package::resume_package;
pub use set_auto_start_docker::set_auto_start_docker;
pub use set_onboarding_completed::set_onboarding_completed;
pub use set_server_url::set_server_url;
pub use start_docker::start_docker;
pub use start_docker_if_needed::{DockerStartStatus, start_docker_if_needed};
pub use stop_package::stop_package;
pub use update_package_config::update_package_config;
pub use web_service::{
    get_web_service_log_path, get_web_service_status, start_web_service, stop_web_service,
};

#[cfg(test)]
pub(crate) mod test_support {
    use std::ffi::{OsStr, OsString};
    use std::sync::{LazyLock, Mutex, MutexGuard};

    use tempfile::TempDir;

    /// Serialises config-mutating tests across crates to avoid clobbering the
    /// shared config override slot.
    pub(crate) static CONFIG_ENV_GUARD: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    pub(crate) struct ConfigEnvOverride {
        _guard: MutexGuard<'static, ()>,
        previous: Option<OsString>,
    }

    fn set_env(value: Option<&OsStr>) {
        unsafe {
            match value {
                Some(val) => std::env::set_var("KITTYNODE_HOME", val),
                None => std::env::remove_var("KITTYNODE_HOME"),
            }
        }
    }

    pub(crate) fn override_kittnode_home_for_tests(path: &std::path::Path) -> ConfigEnvOverride {
        override_kittnode_home_raw_for_tests(Some(path.as_os_str()))
    }

    pub(crate) fn override_kittnode_home_raw_for_tests(value: Option<&OsStr>) -> ConfigEnvOverride {
        let guard = CONFIG_ENV_GUARD
            .lock()
            .expect("config override mutex poisoned");
        let previous = std::env::var_os("KITTYNODE_HOME");
        set_env(value);
        ConfigEnvOverride {
            _guard: guard,
            previous,
        }
    }

    impl Drop for ConfigEnvOverride {
        fn drop(&mut self) {
            let value = self.previous.take();
            set_env(value.as_deref());
        }
    }

    pub(crate) struct ConfigSandbox {
        _override: ConfigEnvOverride,
        _temp: TempDir,
    }

    impl ConfigSandbox {
        pub(crate) fn new() -> Self {
            let temp = tempfile::tempdir().expect("failed to create temporary directory");
            let override_guard = override_kittnode_home_for_tests(temp.path());

            Self {
                _override: override_guard,
                _temp: temp,
            }
        }
    }
}
