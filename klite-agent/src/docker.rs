use bollard::{Docker, container::CreateCheckpointOptions, errors, plugin::Config, query_parameters::StartContainerOptions};
use klite_core::PodSpec;



#[derive(Debug)]
pub enum ContainerState {
    Missing,
    Stopped,
    Running,
}

pub async fn container_status(docker: &Docker, container_name: &str) -> anyhow::Result<ContainerState> {
    match docker.inspect_container(container_name, None).await {
        Ok(response) => {
            let running = response.state.and_then(|state| state.running).unwrap_or(false);
            Ok(if running {ContainerState::Running} else {ContainerState::Stopped})
        }
        Err(errors::Error::DockerResponseServerError { status_code: 404, .. }) => {
            Ok(ContainerState::Missing)
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn create_and_start_container(docker: &Docker, container_name: &str, spec: &PodSpec) -> anyhow::Result<()> {
    let env_vars: Vec<String> =spec.env.iter().map(|(k,v)| format!("{}={}", k, v)).collect(); 

    let config= Config { 
        image: Some(spec.image.clone()),
        cmd: spec.command.clone(),
        env: SOme(env)
        ..Default::default()
    };   

    docker.create_container(Some(CreateCheckpointOptions {name: container_name, platform: None}), config).await?;
    docker.start_container(container_name, None::<StartContainerOptions<String>>).await?;
    Ok(())
}
    

