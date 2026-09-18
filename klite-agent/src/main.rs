use bollard::Docker;

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
                        Ok(status) => {
                            println!("Pod {} is in state {:?}", &pod.uid, status);
                        }
                        Err(e) => eprintln!("Failed to get container status: {e}"),
                    }
                }
            }
            Err(e) => eprintln!("Failed to fetch pods: {e}"),
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
