use kube::{api::Api, Client};
use k8s_openapi::api::core::v1::Pod;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::try_default().await?;
    let pods: Api<Pod> = Api::namespaced(client, "default");

    let pod = pods.get("podname").await?;
    dbg!(pod);

    Ok(())
}
