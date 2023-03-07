use std::error::Error;

use vaultrs::{client::{VaultClient, VaultClientSettingsBuilder}, auth::userpass, api::{kv2::requests::{ReadSecretRequest}, self}};
use log::{debug, trace, warn};

use crate::config::Config;

pub struct Vault {
    client: VaultClient,
}


#[derive(Debug)]
pub struct Env {
    pub name: String,
    pub value: String,
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
        let auth = userpass::login(
            &tmp,
            "userpass",
            &user,
            &pass,
        ).await.unwrap();

        // then set the token as current context
        let settings = VaultClientSettingsBuilder::default()
            .address(endpoint)
            .token(auth.client_token)
            .build()
            .unwrap();
        let client = VaultClient::new(settings)?;

        Ok(Vault { client })
    }


    pub async fn resolve_env(&self, _config: &Config) -> Result<Vec<Env>, Box<dyn Error>> {
        trace!("Resolving env variables.");
        let mut env: Vec<Env> = vec![];
        let mut seen_names: Vec<String> = vec![];

        for e in &_config.env {
            debug!("Resolving env variable: {}", e.name);
            env.push(Env {
                name: e.name.clone(),
                value: self.get_secret(&e.engine, &e.secret, &e.field).await.unwrap(),
            });
            if seen_names.contains(&e.name) {
                warn!("Duplicate env variable name: {}", e.name);
            }
            seen_names.push(e.name.clone());
        }

        Ok(env)
    }

    pub async fn get_secret(&self, engine: &str, secret: &str, field: &str) -> Result<String, Box<dyn Error>> {
        let endpoint = ReadSecretRequest::builder()
            .mount(engine)
            .path(secret)
            .build()
            .unwrap();
        let res = api::exec_with_result(&self.client, endpoint).await?;
        Ok(res.data.get(field).unwrap().to_string())
    }
}
