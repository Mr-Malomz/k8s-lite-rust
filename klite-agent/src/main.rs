
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

    loop {
        match fetch_pods(&client).await {
            Ok(pods) => println!("{:#?}", pods),
            Err(e) => eprintln!("Failed to fetch pods: {e}"),
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
