use bollard::{
    Docker, errors, models::ContainerCreateBody, query_parameters::{CreateContainerOptions, StartContainerOptions},
};
use klite_core::PodSpec;

#[derive(Debug)]
pub enum ContainerState {
    Missing,
    Stopped,
    Running,
}

pub async fn container_status(
    docker: &Docker,
    container_name: &str,
) -> anyhow::Result<ContainerState> {
    match docker.inspect_container(container_name, None).await {
        Ok(response) => {
            let running = response
                .state
                .and_then(|state| state.running)
                .unwrap_or(false);
            Ok(if running {
                ContainerState::Running
            } else {
                ContainerState::Stopped
            })
        }
        Err(errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) => Ok(ContainerState::Missing),
        Err(e) => Err(e.into()),
    }
}

pub async fn create_and_start_container(
    docker: &Docker,
    container_name: &str,
    spec: &PodSpec,
) -> anyhow::Result<()> {
    let env_vars: Vec<String> = spec
        .env
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();

    let config = ContainerCreateBody {
        image: Some(spec.image.clone()),
        cmd: spec.command.clone(),
        env: Some(env_vars),
        ..Default::default()
    };

    docker
        .create_container(
            Some(CreateContainerOptions {
                name: Some(container_name.to_string()),
                platform: String::new(),
            }),
            config,
        )
        .await?;
    docker
        .start_container(container_name, None::<StartContainerOptions>)
        .await?;
    Ok(())
}

pub async fn start_existing_container(docker: &Docker, container_name: &str) -> anyhow::Result<()> {
    docker
        .start_container(container_name, None::<StartContainerOptions>)
        .await?;
    Ok(())
}
