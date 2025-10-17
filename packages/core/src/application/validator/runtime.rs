use crate::domain::package::PackageDefinition;
use crate::domain::validator::ValidatorRuntimeStatus;
use crate::infra::docker::{container_is_running, find_container, get_docker_instance};
use crate::infra::package_config::PackageConfigStore;
use crate::manifests::ethereum::{Ethereum, LIGHTHOUSE_VALIDATOR_CONTAINER_NAME};
use eyre::Result;

pub async fn get_validator_runtime_status() -> Result<ValidatorRuntimeStatus> {
    let config = PackageConfigStore::load(Ethereum::NAME)?;
    let validator_enabled = config
        .values
        .get("validator_enabled")
        .map(|value| value == "true")
        .unwrap_or(false);

    if !validator_enabled {
        return Ok(ValidatorRuntimeStatus::Disabled);
    }

    let docker = get_docker_instance().await?;
    let summaries = find_container(&docker, LIGHTHOUSE_VALIDATOR_CONTAINER_NAME).await?;
    let container_exists = !summaries.is_empty();
    let container_running = summaries.iter().any(container_is_running);

    Ok(ValidatorRuntimeStatus::classify(
        validator_enabled,
        container_exists,
        container_running,
    ))
}
