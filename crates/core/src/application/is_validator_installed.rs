use crate::infra::docker::{find_container, get_docker_instance};
use crate::manifests::ethereum::LIGHTHOUSE_VALIDATOR_CONTAINER_NAME;
use eyre::Result;

pub async fn is_validator_installed() -> Result<bool> {
    let docker = get_docker_instance().await?;
    let containers = find_container(&docker, LIGHTHOUSE_VALIDATOR_CONTAINER_NAME).await?;
    Ok(!containers.is_empty())
}
