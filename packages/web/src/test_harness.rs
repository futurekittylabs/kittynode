use std::collections::{HashMap, VecDeque};
use std::sync::{
    Mutex, MutexGuard, OnceLock,
    atomic::{AtomicBool, Ordering},
};

use eyre::{Result, eyre};
use kittynode_core::api as core_api;
use kittynode_core::api::DockerStartStatus;
use kittynode_core::api::types::{
    Config, OperationalState, Package, PackageConfig, PackageRuntimeState, SystemInfo,
};

pub struct Harness {
    pub add_capability: ResponseQueue<()>,
    pub remove_capability: ResponseQueue<()>,
    pub get_capabilities: ResponseQueue<Vec<String>>,
    pub get_packages: ResponseQueue<HashMap<String, Package>>,
    pub get_config: ResponseQueue<Config>,
    pub install_package: ResponseQueue<()>,
    pub delete_package: ResponseQueue<()>,
    pub stop_package: ResponseQueue<()>,
    pub resume_package: ResponseQueue<()>,
    pub get_package_runtime_state: ResponseQueue<PackageRuntimeState>,
    pub get_packages_runtime_state: ResponseQueue<HashMap<String, PackageRuntimeState>>,
    pub get_installed_packages: ResponseQueue<Vec<Package>>,
    pub is_docker_running: ResponseQueue<bool>,
    pub init_kittynode: ResponseQueue<()>,
    pub delete_kittynode: ResponseQueue<()>,
    pub get_system_info: ResponseQueue<SystemInfo>,
    pub get_container_logs: ResponseQueue<Vec<String>>,
    pub get_package_config: ResponseQueue<PackageConfig>,
    pub update_package_config: ResponseQueue<()>,
    pub start_docker_if_needed: ResponseQueue<DockerStartStatus>,
    pub get_operational_state: ResponseQueue<OperationalState>,
    pub validate_web_port: ResponseQueue<u16>,
    pub delete_package_calls: Vec<(String, bool)>,
    pub update_package_config_calls: Vec<(String, PackageConfig)>,
    pub validate_web_port_calls: Vec<u16>,
    pub skip_server_start: bool,
}

impl Default for Harness {
    fn default() -> Self {
        Self {
            add_capability: ResponseQueue::new("add_capability"),
            remove_capability: ResponseQueue::new("remove_capability"),
            get_capabilities: ResponseQueue::new("get_capabilities"),
            get_packages: ResponseQueue::new("get_packages"),
            get_config: ResponseQueue::new("get_config"),
            install_package: ResponseQueue::new("install_package"),
            delete_package: ResponseQueue::new("delete_package"),
            stop_package: ResponseQueue::new("stop_package"),
            resume_package: ResponseQueue::new("resume_package"),
            get_package_runtime_state: ResponseQueue::new("get_package_runtime_state"),
            get_packages_runtime_state: ResponseQueue::new("get_packages_runtime_state"),
            get_installed_packages: ResponseQueue::new("get_installed_packages"),
            is_docker_running: ResponseQueue::new("is_docker_running"),
            init_kittynode: ResponseQueue::new("init_kittynode"),
            delete_kittynode: ResponseQueue::new("delete_kittynode"),
            get_system_info: ResponseQueue::new("get_system_info"),
            get_container_logs: ResponseQueue::new("get_container_logs"),
            get_package_config: ResponseQueue::new("get_package_config"),
            update_package_config: ResponseQueue::new("update_package_config"),
            start_docker_if_needed: ResponseQueue::new("start_docker_if_needed"),
            get_operational_state: ResponseQueue::new("get_operational_state"),
            validate_web_port: ResponseQueue::new("validate_web_port"),
            delete_package_calls: Vec::new(),
            update_package_config_calls: Vec::new(),
            validate_web_port_calls: Vec::new(),
            skip_server_start: false,
        }
    }
}

pub struct ResponseQueue<T> {
    label: &'static str,
    responses: VecDeque<Result<T>>,
}

impl<T> ResponseQueue<T> {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            responses: VecDeque::new(),
        }
    }

    pub fn push_ok(&mut self, value: T) {
        self.responses.push_back(Ok(value));
    }

    pub fn push_err(&mut self, message: impl Into<String>) {
        let message = message.into();
        self.responses.push_back(Err(eyre!("{}", message)));
    }

    pub fn push_result(&mut self, result: Result<T>) {
        self.responses.push_back(result);
    }

    pub fn next(&mut self) -> Result<T> {
        self.responses
            .pop_front()
            .unwrap_or_else(|| Err(eyre!("No response registered for {}", self.label)))
    }
}

static HARNESS: OnceLock<Mutex<Harness>> = OnceLock::new();
static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static HARNESS_ACTIVE: AtomicBool = AtomicBool::new(false);

fn harness() -> &'static Mutex<Harness> {
    HARNESS.get_or_init(|| Mutex::new(Harness::default()))
}

fn test_lock() -> &'static Mutex<()> {
    TEST_LOCK.get_or_init(|| Mutex::new(()))
}

fn lock_state() -> MutexGuard<'static, Harness> {
    harness()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

fn lock_test() -> MutexGuard<'static, ()> {
    test_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

fn is_active() -> bool {
    HARNESS_ACTIVE.load(Ordering::SeqCst)
}

pub fn reset() {
    let mut state = lock_state();
    *state = Harness::default();
    HARNESS_ACTIVE.store(false, Ordering::SeqCst);
}

pub struct HarnessGuard {
    _lock: MutexGuard<'static, ()>,
}

impl Drop for HarnessGuard {
    fn drop(&mut self) {
        reset();
    }
}

pub fn with_harness<F>(configure: F) -> HarnessGuard
where
    F: FnOnce(&mut Harness),
{
    let lock = lock_test();

    reset();
    HARNESS_ACTIVE.store(true, Ordering::SeqCst);
    let mut state = lock_state();
    configure(&mut state);

    HarnessGuard { _lock: lock }
}

pub fn read_state<F, R>(reader: F) -> R
where
    F: FnOnce(&Harness) -> R,
{
    let state = lock_state();
    reader(&state)
}

pub fn should_skip_server_start() -> bool {
    if !is_active() {
        return false;
    }
    lock_state().skip_server_start
}

pub mod api {
    use super::*;

    pub fn add_capability(name: &str) -> Result<()> {
        if !is_active() {
            return core_api::add_capability(name);
        }
        lock_state().add_capability.next()
    }

    pub fn remove_capability(name: &str) -> Result<()> {
        if !is_active() {
            return core_api::remove_capability(name);
        }
        lock_state().remove_capability.next()
    }

    pub fn get_capabilities() -> Result<Vec<String>> {
        if !is_active() {
            return core_api::get_capabilities();
        }
        lock_state().get_capabilities.next()
    }

    pub fn get_packages() -> Result<HashMap<String, Package>> {
        if !is_active() {
            return core_api::get_packages();
        }
        lock_state().get_packages.next()
    }

    pub fn get_config() -> Result<Config> {
        if !is_active() {
            return core_api::get_config();
        }
        lock_state().get_config.next()
    }

    pub async fn install_package(name: &str) -> Result<()> {
        if !is_active() {
            return core_api::install_package(name).await;
        }
        lock_state().install_package.next()
    }

    pub async fn delete_package(name: &str, include_images: bool) -> Result<()> {
        if !is_active() {
            return core_api::delete_package(name, include_images).await;
        }
        let mut state = lock_state();
        state
            .delete_package_calls
            .push((name.to_string(), include_images));
        state.delete_package.next()
    }

    pub async fn stop_package(name: &str) -> Result<()> {
        if !is_active() {
            return core_api::stop_package(name).await;
        }
        lock_state().stop_package.next()
    }

    pub async fn resume_package(name: &str) -> Result<()> {
        if !is_active() {
            return core_api::resume_package(name).await;
        }
        lock_state().resume_package.next()
    }

    pub async fn get_package_runtime_state(name: &str) -> Result<PackageRuntimeState> {
        if !is_active() {
            return core_api::get_package_runtime_state(name).await;
        }
        lock_state().get_package_runtime_state.next()
    }

    pub async fn get_packages_runtime_state(
        names: &[String],
    ) -> Result<HashMap<String, PackageRuntimeState>> {
        if !is_active() {
            return core_api::get_packages_runtime_state(names).await;
        }
        lock_state().get_packages_runtime_state.next()
    }

    pub async fn get_installed_packages() -> Result<Vec<Package>> {
        if !is_active() {
            return core_api::get_installed_packages().await;
        }
        lock_state().get_installed_packages.next()
    }

    pub async fn is_docker_running() -> bool {
        if !is_active() {
            return core_api::is_docker_running().await;
        }
        lock_state()
            .is_docker_running
            .next()
            .expect("test harness missing is_docker_running response")
    }

    pub fn init_kittynode() -> Result<()> {
        if !is_active() {
            return core_api::init_kittynode();
        }
        lock_state().init_kittynode.next()
    }

    pub fn delete_kittynode() -> Result<()> {
        if !is_active() {
            return core_api::delete_kittynode();
        }
        lock_state().delete_kittynode.next()
    }

    pub fn get_system_info() -> Result<SystemInfo> {
        if !is_active() {
            return core_api::get_system_info();
        }
        lock_state().get_system_info.next()
    }

    pub async fn get_container_logs(name: &str, tail: Option<usize>) -> Result<Vec<String>> {
        if !is_active() {
            return core_api::get_container_logs(name, tail).await;
        }
        lock_state().get_container_logs.next()
    }

    pub async fn get_package_config(name: &str) -> Result<PackageConfig> {
        if !is_active() {
            return core_api::get_package_config(name).await;
        }
        lock_state().get_package_config.next()
    }

    pub async fn update_package_config(name: &str, config: PackageConfig) -> Result<()> {
        if !is_active() {
            return core_api::update_package_config(name, config).await;
        }
        let mut state = lock_state();
        state
            .update_package_config_calls
            .push((name.to_string(), config.clone()));
        state.update_package_config.next()
    }

    pub async fn start_docker_if_needed() -> Result<DockerStartStatus> {
        if !is_active() {
            return core_api::start_docker_if_needed().await;
        }
        lock_state().start_docker_if_needed.next()
    }

    pub async fn get_operational_state() -> Result<OperationalState> {
        if !is_active() {
            return core_api::get_operational_state().await;
        }
        lock_state().get_operational_state.next()
    }
}

pub fn validate_web_port(port: u16) -> Result<u16> {
    if !is_active() {
        return core_api::validate_web_port(port);
    }
    let mut state = lock_state();
    state.validate_web_port_calls.push(port);
    state.validate_web_port.next()
}
