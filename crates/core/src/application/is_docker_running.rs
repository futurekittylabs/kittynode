use crate::infra::docker::get_docker_instance;
use tracing::warn;

pub async fn is_docker_running() -> bool {
    match get_docker_instance().await {
        Ok(_) => true,
        Err(err) => {
            warn!("Docker not reachable: {}", err);
            false
        }
    }
}
