use std::{collections::BTreeMap, error::Error};

use log::{debug, trace, warn};
use vaultrs::{
    api::{self, kv2::requests::ReadSecretRequest},
    auth::userpass,
    client::{VaultClient, VaultClientSettingsBuilder},
};

use crate::config::Config;

pub struct Vault {
    client: VaultClient,
}

impl Vault {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let endpoint = std::env::var("VAULT_ADDR")?;
        let user = std::env::var("VAULT_USER")?;
        let pass = std::env::var("VAULT_PASS")?;

        // login to retrieve token
        let settings = VaultClientSettingsBuilder::default()
            .address(endpoint.clone())
            .build()
            .unwrap();
        let tmp = VaultClient::new(settings)?;
        let auth = userpass::login(&tmp, "userpass", &user, &pass)
            .await
            .unwrap();

        // then set the token as current context
        let settings = VaultClientSettingsBuilder::default()
            .address(endpoint)
            .token(auth.client_token)
            .build()
            .unwrap();
        let client = VaultClient::new(settings)?;

        Ok(Vault { client })
    }

    pub async fn resolve_env(
        &self,
        _config: &Config,
    ) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
        trace!("Resolving env variables.");
        let mut env = BTreeMap::<String, String>::new();

        for e in &_config.env {
            debug!("Resolving env variable: {}", e.name);
            if env.contains_key(&e.name) {
                warn!("Duplicate env variable name: {}", e.name);
            }
            env.insert(
                e.name.clone(),
                self.get_secret(&e.engine, &e.secret, &e.field)
                    .await
                    .unwrap(),
            );
        }

        Ok(env)
    }

    pub async fn get_secret(
        &self,
        engine: &str,
        secret: &str,
        field: &str,
    ) -> Result<String, Box<dyn Error>> {
        let endpoint = ReadSecretRequest::builder()
            .mount(engine)
            .path(secret)
            .build()
            .unwrap();
        let res = api::exec_with_result(&self.client, endpoint).await?;
        Ok(res.data.get(field).unwrap().to_string())
    }
}
