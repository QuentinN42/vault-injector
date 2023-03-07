use std::{collections::BTreeMap};

use kube::{api::Api, Client};
use k8s_openapi::api::core::v1::Pod;
use gethostname::gethostname;

pub async fn get_pod_annotations() -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>>  {
    let client = Client::try_default().await?;
    let pods: Api<Pod> = Api::default_namespaced(client);

    let hostname = gethostname().into_string().unwrap();
    let pod = pods.get(&hostname).await?;

    match pod.metadata.annotations {
        Some(annotations) => Ok(annotations),
        None => Ok(BTreeMap::new()),
    }
}
