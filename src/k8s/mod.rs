use std::collections::BTreeMap;

use base64::{engine::general_purpose, Engine};
use k8s_openapi::api::{apps::v1::Deployment, core::v1::Secret};
use kube::{
    api::{Api, Patch, PatchParams},
    Client, ResourceExt,
};

pub mod selector;
use log::{debug, info};
pub use selector::Selector;

pub struct K8S {
    client: Client,
}

impl K8S {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(K8S {
            client: Client::try_default().await?,
        })
    }

    pub fn secret_api(&self) -> Api<Secret> {
        Api::all(self.client.clone())
    }

    pub async fn get_annotations(
        &self,
    ) -> Result<BTreeMap<Selector, BTreeMap<String, String>>, Box<dyn std::error::Error>> {
        let mut annotations = BTreeMap::new();

        for object in self.secret_api().list(&Default::default()).await? {
            annotations.insert(
                Selector::new(&object),
                match object.metadata.annotations {
                    Some(annotations) => annotations,
                    None => BTreeMap::new(),
                },
            );
        }

        Ok(annotations)
    }

    pub async fn set_env(
        &self,
        object: &Selector,
        envs: &BTreeMap<String, String>,
    ) -> Result<Secret, kube::Error> {
        let patch = serde_json::json!({
            "apiVersion": "v1",
            "kind": "Secret",
            "data": envs.iter().map(|(k, v)| (k.to_owned(), encode(v))).collect::<BTreeMap<String, String>>(),
        });

        let params = PatchParams::apply("vault-injector").force();
        let patch = Patch::Apply(&patch);

        object
            .get_api(&self.client)
            .patch(&object.name, &params, &patch)
            .await
    }

    pub async fn set_env_and_restart_services(
        &self,
        object: &Selector,
        envs: &BTreeMap<String, String>,
    ) -> Result<(), kube::Error> {
        debug!("Setting env for object {:}", object);

        let nb_modified_before = object.get_nb_of_modifications(&self.client).await?;
        self.set_env(object, envs).await.unwrap();
        let nb_modified_after = object.get_nb_of_modifications(&self.client).await?;

        if nb_modified_before != nb_modified_after {
            info!("Secret {} changed, searching linked objects.", object);
            self.restart_linked_to(object).await?;
        }

        Ok(())
    }

    pub async fn restart_linked_to(&self, object: &Selector) -> Result<(), kube::Error> {
        let api = object.get_deployment_api(&self.client);
        let deployments = api.list(&Default::default()).await?;
        for deployment in deployments {
            if need_restart_deployment(&deployment, &object.name) {
                api.restart(&deployment.name_any()).await.unwrap();
                info!("Restarted deployment {}.", deployment.name_any());
            }
        }

        Ok(())
    }
}

fn encode(txt: &str) -> String {
    general_purpose::STANDARD.encode(txt.as_bytes())
}

fn need_restart_deployment(deployment: &Deployment, secret_name: &String) -> bool {
    let dep_spec = deployment.spec.as_ref();
    if dep_spec.is_none() {
        return false;
    }

    let pod_spec = dep_spec.unwrap().template.spec.as_ref();
    if pod_spec.is_none() {
        return false;
    }

    let cts = &pod_spec.unwrap().containers;
    for ct in cts {
        if let Some(envs) = ct.env.as_ref() {
            for env in envs {
                if env.value_from.is_none() {
                    continue;
                }
                let env_from = env.value_from.as_ref().unwrap();
                if env_from.secret_key_ref.is_none() {
                    continue;
                }
                let secret_key_ref = env_from.secret_key_ref.as_ref().unwrap();
                if secret_key_ref.name.is_some()
                    && secret_key_ref.name.as_ref().unwrap() == secret_name.as_str()
                {
                    return true;
                }
            }
        }
        if let Some(envs) = ct.env_from.as_ref() {
            for env in envs {
                if env.secret_ref.is_none() {
                    continue;
                }
                let source = env.secret_ref.as_ref().unwrap();
                if source.name.is_none() {
                    continue;
                }
                let secret_key_ref = source.name.as_ref();
                if secret_key_ref.is_some() && secret_key_ref.unwrap() == secret_name.as_str() {
                    return true;
                }
            }
        }
    }
    false
}
