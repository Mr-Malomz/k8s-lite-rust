use bollard::Docker;
use klite_core::PodStatus;

mod docker;

async fn fetch_pods(client: &reqwest::Client) -> anyhow::Result<Vec<klite_core::Pod>> {
    let pods = client
        .get("http://127.0.0.1:3000/pods")
        .send()
        .await?
        .json::<Vec<klite_core::Pod>>()
        .await?;

    Ok(pods)
}

async fn report_status(
    client: &reqwest::Client,
    name: &str,
    status: &PodStatus,
) -> anyhow::Result<()> {
    client
        .patch(format!("http://127.0.0.1:3000/pods/{}/status", &name))
        .json(status)
        .send()
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let docker = Docker::connect_with_local_defaults().expect("Failed to connect to Docker");

    loop {
        match fetch_pods(&client).await {
            Ok(pods) => {
                println!("{:#?}", pods);
                for pod in pods {
                    let container_name = format!("klite-{}", &pod.uid);
                    match docker::container_status(&docker, &container_name).await {
                        Ok(status) => match status {
                            docker::ContainerState::Running(id) => {
                                println!("Pod {} is already running", id);
                            }
                            docker::ContainerState::Stopped(id) => {
                                match docker::start_existing_container(&docker, &container_name)
                                    .await
                                {
                                    Ok(_) => {
                                        let status = PodStatus {
                                            phase: klite_core::PodPhase::Running,
                                            node_name: Some("local".into()),
                                            container_id: Some(id),
                                            restart_count: pod.status.restart_count + 1,
                                            last_transition: std::time::SystemTime::now(),
                                        };
                                        match report_status(&client, &pod.spec.name, &status).await {
                                            Ok(_) => {},
                                            Err(e) => eprintln!("Failed to report pod status: {e}"),
                                        }
                                        println!("Pod {} started successfully", &pod.uid)
                                    }
                                    Err(e) => eprintln!("Failed to start pod {}: {}", &pod.uid, e),
                                };
                            }
                            docker::ContainerState::Missing => {
                                match docker::create_and_start_container(
                                    &docker,
                                    &container_name,
                                    &pod.spec,
                                )
                                .await
                                {
                                    Ok(id) => {
                                        let status = PodStatus {
                                            phase: klite_core::PodPhase::Running,
                                            node_name: Some("local".into()),
                                            container_id: Some(id),
                                            restart_count: 0,
                                            last_transition: std::time::SystemTime::now(),
                                        };
                                        match report_status(&client, &pod.spec.name, &status).await {
                                            Ok(_) => {},
                                            Err(e) => eprintln!("Failed to report pod status: {e}"),
                                        }
                                        println!(
                                            "Pod {} created and started successfully",
                                            &pod.uid
                                        )
                                    }
                                    Err(e) => eprintln!(
                                        "Failed to create and start pod {}: {}",
                                        &pod.uid, e
                                    ),
                                };
                            }
                        },
                        Err(e) => eprintln!("Failed to get container status: {e}"),
                    }
                }
            }
            Err(e) => eprintln!("Failed to fetch pods: {e}"),
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
