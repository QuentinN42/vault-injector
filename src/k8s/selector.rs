use k8s_openapi::api::{apps::v1::Deployment, core::v1::Secret};
use kube::{api::Api, Client, ResourceExt};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, std::hash::Hash)]
pub struct Selector {
    pub name: String,
    namespace: Option<String>,
}

impl Selector {
    pub fn new(object: &Secret) -> Self {
        Selector {
            name: object.name_any(),
            namespace: object.namespace(),
        }
    }

    pub async fn get(&self, client: &Client) -> Result<Secret, kube::Error> {
        self.get_api(client).get(&self.name).await
    }

    pub fn get_api(&self, client: &Client) -> Api<Secret> {
        match &self.namespace {
            Some(namespace) => Api::namespaced(client.clone(), namespace),
            None => Api::default_namespaced(client.clone()),
        }
    }

    pub fn get_deployment_api(&self, client: &Client) -> Api<Deployment> {
        match &self.namespace {
            Some(namespace) => Api::namespaced(client.clone(), namespace),
            None => Api::default_namespaced(client.clone()),
        }
    }

    pub async fn get_nb_of_modifications(&self, client: &Client) -> Result<usize, kube::Error> {
        match &self.get(client).await?.metadata.managed_fields {
            Some(managed_fields) => Ok(managed_fields.len()),
            None => Ok(0),
        }
    }
}

impl std::fmt::Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.namespace {
            Some(namespace) => write!(f, "{}/{}", self.name, namespace),
            None => write!(f, "{}", self.name),
        }
    }
}
