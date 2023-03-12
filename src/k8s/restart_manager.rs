use k8s_openapi::{
    api::{apps::v1::Deployment, core::v1::Secret},
    NamespaceResourceScope,
};
use kube::{core::util::Restart, Api, Resource};
use serde::de::DeserializeOwned;

use crate::k8s::Selector;

pub struct RestartManager<K>
where
    K: Restart + Resource,
    <K as kube::Resource>::DynamicType: Default,
    K: Resource<Scope = NamespaceResourceScope>,
    K: Clone + DeserializeOwned + std::fmt::Debug,
{
    client: kube::Client,
    data: Vec<Selector<K>>,
}

impl<K> RestartManager<K>
where
    K: Restart + Resource + 'static,
    <K as kube::Resource>::DynamicType: Default,
    K: Resource<Scope = NamespaceResourceScope>,
    K: Clone + DeserializeOwned + std::fmt::Debug,
{
    pub fn new(client: &kube::Client) -> Self {
        RestartManager {
            client: client.clone(),
            data: Vec::new(),
        }
    }

    pub async fn add_linked_services(&mut self, object: &Selector<Secret>) {
        let api: Api<K> = match object.namespace() {
            Some(namespace) => Api::namespaced(self.client.clone(), &namespace),
            None => Api::default_namespaced(self.client.clone()),
        };
        let objs = match api.list(&Default::default()).await {
            Ok(objs) => objs,
            Err(e) => {
                log::error!("Failed to list deployments: {}", e);
                return;
            }
        };
        for obj in objs {
            let sel = Selector::new(&obj);
            if self.data.contains(&sel) {
                continue;
            }
            if need_restart(&obj, &object.name()) {
                self.data.push(sel);
            }
        }
    }

    pub async fn restart(&self) {
        for object in &self.data {
            let api = object.get_api(&self.client.clone());
            match api.restart(&object.name()).await {
                Ok(_) => log::info!("Restarted {}", object.name()),
                Err(e) => log::error!("Failed to restart {}: {}", object.name(), e),
            };
        }
    }
}

macro_rules! need_restart_macro {
    ( $e:expr , $n:expr ) => {
        if let Some(f) = (&$e as &dyn std::any::Any).downcast_ref::<Deployment>() {
            need_restart_deployment(f, &$n)
        } else {
            log::warn!("Not implemented for object `{:?}`", $e);
            false
        }
    };
}

fn need_restart<K>(object: &K, secret_name: &String) -> bool
where
    K: Restart + Resource + 'static,
    <K as kube::Resource>::DynamicType: Default,
    K: Resource<Scope = NamespaceResourceScope>,
    K: Clone + DeserializeOwned + std::fmt::Debug,
{
    need_restart_macro!(object.to_owned(), secret_name)
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
