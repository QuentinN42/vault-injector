use std::collections::BTreeMap;

use base64::{engine::general_purpose, Engine};
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{Api, Patch, PatchParams},
    Client,
};

pub mod restart_manager;
pub mod selector;
use log::debug;
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

    pub fn client(&self) -> Client {
        self.client.clone()
    }

    pub async fn get_annotations(
        &self,
    ) -> Result<BTreeMap<Selector<Secret>, BTreeMap<String, String>>, Box<dyn std::error::Error>>
    {
        let mut annotations = BTreeMap::new();
        let secret_api: Api<Secret> = Api::all(self.client.clone());

        for object in secret_api.list(&Default::default()).await? {
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

    async fn _set_env(
        &self,
        object: &Selector<Secret>,
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
            .patch(&object.name(), &params, &patch)
            .await
    }

    pub async fn set_env(
        &self,
        object: &Selector<Secret>,
        envs: &BTreeMap<String, String>,
    ) -> Result<bool, kube::Error> {
        debug!("Setting env for object {:}", object);

        let last_modified_before = object.get_last_update(&self.client).await?;
        self._set_env(object, envs).await.unwrap();
        let last_modified_after = object.get_last_update(&self.client).await?;

        Ok(last_modified_before != last_modified_after)
    }
}

fn encode(txt: &str) -> String {
    general_purpose::STANDARD.encode(txt.as_bytes())
}
