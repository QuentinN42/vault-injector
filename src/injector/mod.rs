use std::collections::BTreeMap;

use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::Secret;
use log::{debug, error, trace, warn};
use semver::{Version, VersionReq};

use crate::config::parse_config;
use crate::k8s::restart_manager::RestartManager;
use crate::k8s::{Selector, K8S};
use crate::vault::Vault;

pub struct Injector {
    version: Version,
    vault: Vault,
    k8s: K8S,
}

impl Injector {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let injector = Injector {
            version: Version::parse(clap::crate_version!())?,
            vault: Vault::new().await?,
            k8s: K8S::new().await?,
        };
        trace!("Initialized vault and k8s clients.");
        Ok(injector)
    }

    pub async fn run(&self) {
        match self.run_once().await {
            Ok(_) => {}
            Err(e) => {
                error!("Error: {}", e);
            }
        };
    }

    async fn run_once(&self) -> Result<(), Box<dyn std::error::Error>> {
        trace!("Starting annotation collection.");

        let annotations = self.k8s.get_annotations().await?;
        let mut restart_manager = RestartManager::<Deployment>::new(&self.k8s.client());

        for (key, value) in annotations.iter() {
            let env = self.generate_env(key, value).await?;

            if env.is_empty() {
                continue;
            }

            let need_restart = self.k8s.set_env(&key, &env).await?;
            if need_restart {
                restart_manager.add_linked_services(&key).await;
            }
        }

        restart_manager.restart().await;
        Ok(())
    }

    async fn generate_env(
        &self,
        object: &Selector<Secret>,
        annotations: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
        let config = match self.get_config_if_available(object, annotations) {
            Some(c) => c,
            None => return Ok(BTreeMap::new()),
        };

        match self.vault.resolve_env(&config).await {
            Ok(env) => Ok(env),
            Err(e) => {
                warn!("Unable to resolve env of {}: {}", object, e);
                Ok(BTreeMap::new())
            }
        }
    }

    fn get_config_if_available(
        &self,
        object: &Selector<Secret>,
        annotations: &BTreeMap<String, String>,
    ) -> Option<crate::config::Config> {
        if !annotations.contains_key("vault-injector.io/version") {
            return None;
        }

        let version = match VersionReq::parse(&annotations["vault-injector.io/version"]) {
            Ok(v) => v,
            Err(e) => {
                warn!("Unable to parse version of {}: {}", object, e);
                return None;
            }
        };

        if !version.matches(&self.version) {
            debug!(
                "Version of {} does not match current executor: {} - {}",
                object, self.version, version
            );
            return None;
        }

        if !annotations.contains_key("vault-injector.io/config") {
            return None;
        }

        match parse_config(&annotations["vault-injector.io/config"]) {
            Ok(c) => Some(c),
            Err(e) => {
                warn!("Unable to parse config of {}: {}", object, e);
                None
            }
        }
    }
}
