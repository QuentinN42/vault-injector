use core::time;
use std::collections::BTreeMap;
use std::thread;

use log::{debug, error, trace, warn};
use semver::{Version, VersionReq};

use crate::config::parse_config;
use crate::k8s::{DeploymentSelector, K8S};
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
        loop {
            match self.run_once().await {
                Ok(_) => {}
                Err(e) => {
                    error!("Error: {}", e);
                }
            };
            sleep(10);
        }
    }

    async fn run_once(&self) -> Result<(), Box<dyn std::error::Error>> {
        trace!("Starting annotation collection.");

        let annotations = self.k8s.get_annotations().await?;

        for (key, value) in annotations.iter() {
            trace!("Found deployment {}", key);
            for (k, v) in value.iter() {
                trace!("  Found annotation {}={}", k, v);
            }

            let env = self.generate_env(key, value).await?;

            if env.is_empty() {
                trace!("No env to inject.");
                continue;
            }

            for (k, v) in env.iter() {
                trace!("  Generated env {}={}", k, v);
            }

            self.k8s.set_env(&key, &env).await?;
        }

        Ok(())
    }

    async fn generate_env(
        &self,
        deploment: &DeploymentSelector,
        annotations: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
        let config = match self.get_config_if_available(deploment, annotations) {
            Some(c) => c,
            None => return Ok(BTreeMap::new()),
        };

        match self.vault.resolve_env(&config).await {
            Ok(env) => Ok(env),
            Err(e) => {
                warn!("Unable to resolve env of deployment {}: {}", deploment, e);
                Ok(BTreeMap::new())
            }
        }
    }

    fn get_config_if_available(
        &self,
        deploment: &DeploymentSelector,
        annotations: &BTreeMap<String, String>,
    ) -> Option<crate::config::Config> {
        if !annotations.contains_key("vault-injector.io/version") {
            debug!("No vault-injector.io/version annotation found.");
            return None;
        }

        let version = match VersionReq::parse(&annotations["vault-injector.io/version"]) {
            Ok(v) => v,
            Err(e) => {
                warn!("Unable to parse version of deployment {}: {}", deploment, e);
                return None;
            }
        };

        if !version.matches(&self.version) {
            debug!(
                "Version of deployment {} does not match current executor: {} - {}",
                deploment, self.version, version
            );
            return None;
        }

        if !annotations.contains_key("vault-injector.io/config") {
            debug!("No vault-injector.io/config annotation found.");
            return None;
        }

        match parse_config(&annotations["vault-injector.io/config"]) {
            Ok(c) => Some(c),
            Err(e) => {
                warn!("Unable to parse config of deployment {}: {}", deploment, e);
                None
            }
        }
    }
}

fn sleep(seconds: u64) {
    thread::sleep(time::Duration::from_secs(seconds));
}
