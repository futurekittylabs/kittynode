#[cfg(test)]
use std::sync::Mutex;

#[cfg(test)]
pub(crate) static ENV_GUARD: Mutex<()> = Mutex::new(());

// Public modules
pub mod api;

// Internal modules
mod application;
mod domain;
mod infra;
mod manifests;
