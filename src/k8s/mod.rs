use std::collections::BTreeMap;

use gethostname::gethostname;
use k8s_openapi::api::{
    apps::v1::{Deployment, ReplicaSet},
    core::v1::Pod,
};
use kube::{
    api::{Api, Patch, PatchParams},
    Client, ResourceExt,
};
use log::debug;

pub struct K8S {
    pod_api: Api<Pod>,
    replicaset_api: Api<ReplicaSet>,
    deployment_api: Api<Deployment>,
    hostname: String,
}

impl K8S {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(K8S {
            pod_api: Api::all(Client::try_default().await?),
            replicaset_api: Api::all(Client::try_default().await?),
            deployment_api: Api::all(Client::try_default().await?),
            hostname: gethostname().into_string().unwrap(),
        })
    }

    async fn get_deployment(&self) -> Result<Deployment, Box<dyn std::error::Error>> {
        let pod = self.pod_api.get(&self.hostname).await?;

        let repliacset = pod.metadata.owner_references.unwrap()[0].clone();

        let rs = self.replicaset_api.get(&repliacset.name).await?;

        let deployment = rs.metadata.owner_references.unwrap()[0].clone();

        let dep = self.deployment_api.get(&deployment.name).await?;

        Ok(dep)
    }

    pub async fn get_annotations(
        &self,
    ) -> Result<BTreeMap<String, BTreeMap<String, String>>, Box<dyn std::error::Error>> {
        let dep = self.get_deployment().await?;

        let mut annotations = BTreeMap::new();
        annotations.insert("one".to_string(), dep.metadata.annotations.unwrap());

        Ok(annotations)
        // match dep.metadata.annotations {
        //     Some(annotations) => Ok(annotations),
        //     None => Ok(BTreeMap::new()),
        // }
    }

    pub async fn set_env(
        &self,
        envs: &BTreeMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dep = self.get_deployment().await?;
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
                                "env": envs.iter().map(|(k, v)| {
                                    serde_json::json!({
                                        "name": k,
                                        "value": v,
                                    })
                                }).collect::<Vec<_>>(),
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
}
