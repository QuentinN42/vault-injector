use std::collections::BTreeMap;

use gethostname::gethostname;
use k8s_openapi::api::{apps::v1::{Deployment, ReplicaSet}, core::v1::Pod};
use kube::{api::{Api, PatchParams, Patch}, Client, ResourceExt};
use log::debug;

use crate::vault::Env;


async fn get_deployment() -> Result<Deployment, Box<dyn std::error::Error>> {
    let pods: Api<Pod> = Api::default_namespaced(Client::try_default().await?);

    let hostname = gethostname().into_string().unwrap();
    let pod = pods.get(&hostname).await?;

    let repliacset = pod.metadata.owner_references.unwrap()[0].clone();

    let rs: Api<ReplicaSet> = Api::default_namespaced(Client::try_default().await?);
    let rs = rs.get(&repliacset.name).await?;

    let deployment = rs.metadata.owner_references.unwrap()[0].clone();

    let deps: Api<Deployment> = Api::default_namespaced(Client::try_default().await?);
    let dep = deps.get(&deployment.name).await?;

    Ok(dep)
}


pub async fn get_annotations() -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    let dep = get_deployment().await?;

    match dep.metadata.annotations {
        Some(annotations) => Ok(annotations),
        None => Ok(BTreeMap::new()),
    }
}


pub async fn set_env(envs: &Vec<Env>) -> Result<(), Box<dyn std::error::Error>> {
    let dep = get_deployment().await?;
    let dep_name = dep.name_any();

    let cts = dep.spec.unwrap().template.spec.unwrap().containers;
    let patch = serde_json::json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "spec": {
            "template": {
                "spec": {
                    "containers": cts.iter().map(|c| {
                        serde_json::json!({
                            "name": c.name,
                            "env": envs.iter().map(|e| {
                                serde_json::json!({
                                    "name": e.name,
                                    "value": e.value,
                                })
                            }).collect::<Vec<_>>()
                        })
                    }).collect::<Vec<_>>(),
                }
            }
            
        }
    });

    let params = PatchParams::apply("vault-injector").force();
    let patch = Patch::Apply(&patch);

    let deps: Api<Deployment> = Api::default_namespaced(Client::try_default().await?);
    let o_patched = deps.patch(&dep_name, &params, &patch).await?;
    debug!("Patched deployment : {:?}", o_patched);
    Ok(())
}
