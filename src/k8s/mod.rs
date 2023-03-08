use std::{collections::BTreeMap, fmt::Display, hash};

use k8s_openapi::api::apps::v1::Deployment;
use kube::{
    api::{Api, Patch, PatchParams},
    Client, ResourceExt,
};
use log::debug;

pub struct K8S {
    deployment_api: Api<Deployment>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, hash::Hash)]
pub struct DeploymentSelector {
    name: String,
    namespace: String,
}

impl DeploymentSelector {
    pub fn new(dep: &Deployment, default_namespace: &String) -> Self {
        DeploymentSelector {
            name: dep.name_any(),
            namespace: dep.namespace().unwrap_or(default_namespace.clone()),
        }
    }

    pub async fn get_deployment(&self) -> Result<Deployment, Box<dyn std::error::Error>> {
        let dep = self.get_api().await?.get(&self.name).await?;
        Ok(dep)
    }

    pub async fn get_api(&self) -> Result<Api<Deployment>, Box<dyn std::error::Error>> {
        let deps: Api<Deployment> = Api::namespaced(Client::try_default().await?, &self.namespace);
        Ok(deps)
    }
}

impl Display for DeploymentSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.name, self.namespace)
    }
}

impl K8S {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(K8S {
            deployment_api: Api::all(Client::try_default().await?),
        })
    }

    pub async fn get_annotations(
        &self,
    ) -> Result<BTreeMap<DeploymentSelector, BTreeMap<String, String>>, Box<dyn std::error::Error>>
    {
        let mut annotations = BTreeMap::new();
        let deps = self.deployment_api.list(&Default::default()).await?;
        let default_namespace = "default".to_string();

        for dep in deps {
            let key = DeploymentSelector::new(&dep, &default_namespace);
            let value = match dep.metadata.annotations {
                Some(x) => x,
                None => BTreeMap::new(),
            };
            annotations.insert(key, value);
        }

        Ok(annotations)
    }

    pub async fn set_env(
        &self,
        deployment: &DeploymentSelector,
        envs: &BTreeMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dep = deployment.get_deployment().await?;

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

        let o_patched = deployment
            .get_api()
            .await?
            .patch(&deployment.name, &params, &patch)
            .await?;
        debug!("Patched deployment : {:?}", o_patched);
        Ok(())
    }
}
