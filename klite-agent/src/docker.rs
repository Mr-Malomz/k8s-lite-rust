use bollard::{
    Docker, errors,
    models::ContainerCreateBody,
    query_parameters::{CreateContainerOptions, CreateImageOptionsBuilder, StartContainerOptions},
};
use futures_util::TryStreamExt;
use klite_core::PodSpec;

#[derive(Debug)]
pub enum ContainerState {
    Missing,
    Stopped { id: String, exit_code: Option<i64> },
    Running(String),
}

pub async fn container_status(
    docker: &Docker,
    container_name: &str,
) -> anyhow::Result<ContainerState> {
    match docker.inspect_container(container_name, None).await {
        Ok(response) => {
            let state = response.state;
            let running = state
                .as_ref()
                .and_then(|state| state.running)
                .unwrap_or(false);
            let exit_code = state.as_ref().and_then(|state| state.exit_code);
            let id = response.id.clone().unwrap_or_default();
            Ok(if running {
                ContainerState::Running(id)
            } else {
                ContainerState::Stopped { id, exit_code }
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
) -> anyhow::Result<String> {
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

    let response = match docker
        .create_container(
            Some(CreateContainerOptions {
                name: Some(container_name.to_string()),
                platform: String::new(),
            }),
            config.clone(),
        )
        .await {
            Ok(response) => response,
            Err(errors::Error::DockerResponseServerError {
                status_code: 404,
                ..
            }) => {
                pull_image(docker, &spec.image).await?;
                docker.create_container(
                    Some(CreateContainerOptions {
                        name: Some(container_name.to_string()),
                        platform: String::new(),
                    }),
                    config,
                )
                .await?
            }
            Err(e) => return Err(e.into()),
        };
    docker
        .start_container(container_name, None::<StartContainerOptions>)
        .await?;
    Ok(response.id)
}

pub async fn start_existing_container(docker: &Docker, container_name: &str) -> anyhow::Result<()> {
    docker
        .start_container(container_name, None::<StartContainerOptions>)
        .await?;
    Ok(())
}

pub async fn pull_image(docker: &Docker, image: &str) -> anyhow::Result<()> {
    println!("pulling image {}", image);
    docker
        .create_image(
            Some(CreateImageOptionsBuilder::default().from_image(image).build()),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await?;
    Ok(())
}
