use crate::domain::container::{Binding, Container};
use bollard::{
    Docker,
    models::{ContainerCreateBody, EndpointSettings, NetworkConnectRequest, NetworkCreateRequest},
    query_parameters::{
        CreateContainerOptionsBuilder, CreateImageOptionsBuilder, ListContainersOptionsBuilder,
        LogsOptionsBuilder,
    },
    secret::{ContainerSummary, HostConfig},
};
use eyre::{Report, Result};
use std::collections::HashMap;
use tokio_stream::StreamExt;
use tracing::{error, info};

pub(crate) fn get_docker_instance() -> Result<Docker> {
    Docker::connect_with_local_defaults().map_err(Report::from)
}

pub(crate) async fn create_or_recreate_network(docker: &Docker, network_name: &str) -> Result<()> {
    // Check if network already exists
    let network_exists = docker
        .list_networks(None::<bollard::query_parameters::ListNetworksOptions>)
        .await?
        .iter()
        .any(|n| n.name.as_deref() == Some(network_name));

    // Remove network if it already exists
    if network_exists {
        docker.remove_network(network_name).await?;
        info!("Removed existing network: '{}'", network_name);
    }

    // Create new network
    let network_config = NetworkCreateRequest {
        name: network_name.to_string(),
        driver: Some("bridge".to_string()),
        ..Default::default()
    };
    docker.create_network(network_config).await?;
    info!("Created new network: '{}'", network_name);

    Ok(())
}

pub(crate) async fn find_container(docker: &Docker, name: &str) -> Result<Vec<ContainerSummary>> {
    let filters = HashMap::from([("name".to_string(), vec![name.to_string()])]);

    let options = ListContainersOptionsBuilder::default()
        .all(true)
        .filters(&filters)
        .build();

    Ok(docker.list_containers(Some(options)).await?)
}

pub(crate) async fn remove_container(docker: &Docker, name: &str) -> Result<()> {
    for container in find_container(docker, name).await? {
        let id = container
            .id
            .ok_or_else(|| eyre::eyre!("Container ID was None"))?;
        docker
            .stop_container(&id, None::<bollard::query_parameters::StopContainerOptions>)
            .await
            .ok(); // Ignore stop errors
        docker
            .remove_container(
                &id,
                None::<bollard::query_parameters::RemoveContainerOptions>,
            )
            .await?;
    }

    Ok(())
}

pub(crate) async fn pull_and_start_container(
    docker: &Docker,
    container: &Container,
    network_name: &str,
) -> Result<()> {
    let options = Some(
        CreateImageOptionsBuilder::default()
            .from_image(container.image.as_str())
            .tag("latest")
            .build(),
    );

    let mut stream = docker.create_image(options, None, None);
    while let Some(item) = stream.next().await {
        match item {
            Ok(info) => info!("Pulling image info: {:?}", info),
            Err(e) => error!("Error pulling image: {:?}", e),
        }
    }

    let port_bindings = container
        .port_bindings
        .iter()
        .map(|(k, v)| (k.to_string(), Some(v.clone())))
        .collect();

    let bindings = container
        .volume_bindings
        .iter()
        .chain(&container.file_bindings)
        .map(create_binding_string)
        .collect();

    let host_config = HostConfig {
        binds: Some(bindings),
        port_bindings: Some(port_bindings),
        ..Default::default()
    };

    let config = ContainerCreateBody {
        image: Some(container.image.to_string()),
        cmd: Some(container.cmd.clone()),
        host_config: Some(host_config),
        ..Default::default()
    };

    let options = Some(
        CreateContainerOptionsBuilder::default()
            .name(container.name.as_str())
            .build(),
    );

    let created_container = docker.create_container(options, config).await?;
    info!("Container {} created successfully", container.name);

    docker
        .start_container(
            &created_container.id,
            None::<bollard::query_parameters::StartContainerOptions>,
        )
        .await?;
    info!("Container {} started successfully", container.name);

    let connect_options = NetworkConnectRequest {
        container: Some(container.name.to_string()),
        endpoint_config: Some(EndpointSettings::default()),
    };

    docker
        .connect_network(network_name, connect_options)
        .await?;
    info!(
        "Container {} connected to network '{}'",
        container.name, network_name
    );

    Ok(())
}

pub async fn get_container_logs(
    docker: &Docker,
    container_name: &str,
    tail_lines: Option<usize>,
) -> Result<Vec<String>> {
    let tail = tail_lines.map_or_else(|| "all".to_string(), |n| n.to_string());

    let options = LogsOptionsBuilder::default()
        .stdout(true)
        .stderr(true)
        .follow(false)
        .timestamps(true)
        .tail(&tail)
        .build();

    let mut stream = docker.logs(container_name, Some(options));
    let mut log_strings = Vec::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(output) => match output {
                bollard::container::LogOutput::StdOut { message }
                | bollard::container::LogOutput::StdErr { message } => {
                    log_strings.push(String::from_utf8_lossy(&message).to_string());
                }
                _ => {}
            },
            Err(e) => return Err(e.into()),
        }
    }

    Ok(log_strings)
}

fn create_binding_string(binding: &Binding) -> String {
    match &binding.options {
        Some(options) => format!("{}:{}:{}", binding.source, binding.destination, options),
        None => format!("{}:{}", binding.source, binding.destination),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_string_with_options() {
        let b = Binding {
            source: "/src".into(),
            destination: "/dst".into(),
            options: Some("ro".into()),
        };
        assert_eq!(super::create_binding_string(&b), "/src:/dst:ro");
    }

    #[test]
    fn binding_string_without_options() {
        let b = Binding {
            source: "/src".into(),
            destination: "/dst".into(),
            options: None,
        };
        assert_eq!(super::create_binding_string(&b), "/src:/dst");
    }
}
