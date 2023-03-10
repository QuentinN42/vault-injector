use std::collections::BTreeMap;

use base64::{engine::general_purpose, Engine};
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{Api, Patch, PatchParams},
    Client,
};

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
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Setting env for object {:}", object);
        for e in envs.iter() {
            debug!("  {}={}", e.0, e.1);
        }

        let patch = serde_json::json!({
            "apiVersion": "v1",
            "kind": "Secret",
            "data": envs.iter().map(|(k, v)| (k.to_owned(), encode(v))).collect::<BTreeMap<String, String>>(),
        });

        let params = PatchParams::apply("vault-injector");
        let patch = Patch::Apply(&patch);

        let o_patched = object
            .get_api(&self.client)
            .patch(&object.name, &params, &patch)
            .await?;
        debug!("Patched deployment : {:?}", o_patched);
        Ok(())
    }
}

fn encode(txt: &str) -> String {
    general_purpose::STANDARD.encode(txt.as_bytes())
}
