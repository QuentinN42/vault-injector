use chrono;
use k8s_openapi::{apimachinery::pkg::apis::meta::v1::Time, NamespaceResourceScope};
use kube::{api::Api, Client, Resource, ResourceExt};
use serde::de::DeserializeOwned;

#[derive(Debug, Clone, std::hash::Hash)]
pub struct Selector<T: Resource>
where
    <T as kube::Resource>::DynamicType: Default,
    T: Resource<Scope = NamespaceResourceScope>,
    T: Clone + DeserializeOwned + std::fmt::Debug,
{
    name: String,
    namespace: Option<String>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Resource> Selector<T>
where
    <T as kube::Resource>::DynamicType: Default,
    T: Resource<Scope = NamespaceResourceScope>,
    T: Clone + DeserializeOwned + std::fmt::Debug,
{
    pub fn new(object: &T) -> Self {
        Selector {
            name: object.name_any(),
            namespace: object.namespace(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn namespace(&self) -> Option<String> {
        self.namespace.clone()
    }

    pub fn get_api(&self, client: &Client) -> Api<T>
    where
        <T as kube::Resource>::DynamicType: Default,
    {
        match &self.namespace {
            Some(namespace) => Api::namespaced(client.clone(), namespace),
            None => Api::default_namespaced(client.clone()),
        }
    }

    pub async fn get(&self, client: &Client) -> Result<T, kube::Error> {
        let api = self.get_api(client);
        api.get(&self.name).await
    }

    pub async fn get_last_update(&self, client: &Client) -> Result<Time, kube::Error> {
        let obj = self.get(client).await?;
        let managed = obj.managed_fields();
        let mut last_update = Time(chrono::Utc::now() - chrono::Duration::days(365));
        for field in managed {
            let maybe_time = field.time.as_ref();
            if maybe_time.is_none() {
                continue;
            }
            let time = maybe_time.unwrap().to_owned();
            if time > last_update {
                last_update = time;
            }
        }
        Ok(last_update)
    }
}

impl<T: Resource> std::fmt::Display for Selector<T>
where
    <T as kube::Resource>::DynamicType: Default,
    T: Resource<Scope = NamespaceResourceScope>,
    T: Clone + DeserializeOwned + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dt = <T as kube::Resource>::DynamicType::default();
        let resource_name = T::kind(&dt);
        match &self.namespace {
            Some(namespace) => write!(f, "{} {}/{}", resource_name, self.name, namespace),
            None => write!(f, "{} {}", resource_name, self.name),
        }
    }
}

impl<T: Resource> Ord for Selector<T>
where
    <T as kube::Resource>::DynamicType: Default,
    T: Resource<Scope = NamespaceResourceScope>,
    T: Clone + DeserializeOwned + std::fmt::Debug,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let ns_comp = self.namespace.cmp(&other.namespace);
        if ns_comp != std::cmp::Ordering::Equal {
            return ns_comp;
        }
        let name_comp = self.name.cmp(&other.name);
        name_comp
    }
}

impl<T: Resource> PartialOrd for Selector<T>
where
    <T as kube::Resource>::DynamicType: Default,
    T: Resource<Scope = NamespaceResourceScope>,
    T: Clone + DeserializeOwned + std::fmt::Debug,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Resource> PartialEq for Selector<T>
where
    <T as kube::Resource>::DynamicType: Default,
    T: Resource<Scope = NamespaceResourceScope>,
    T: Clone + DeserializeOwned + std::fmt::Debug,
{
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl<T: Resource> Eq for Selector<T>
where
    <T as kube::Resource>::DynamicType: Default,
    T: Resource<Scope = NamespaceResourceScope>,
    T: Clone + DeserializeOwned + std::fmt::Debug,
{
}
