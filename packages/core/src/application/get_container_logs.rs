use crate::infra::docker::get_docker_instance;
use eyre::Result;

pub async fn get_container_logs(
    container_name: &str,
    tail_lines: Option<usize>,
) -> Result<Vec<String>> {
    let docker = get_docker_instance().await?;
    crate::infra::docker::get_container_logs(&docker, container_name, tail_lines).await
}
