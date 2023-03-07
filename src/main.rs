use log::{debug, error, info, trace};
use std::{thread, time};

mod k8s;
use k8s::{get_annotations, set_env};

mod config;
use config::parse_config;

mod vault;
use vault::Vault;

static SLEEP_TIME: time::Duration = time::Duration::from_secs(10);

#[tokio::main]
async fn main() {
    env_logger::init();
    let vault = match Vault::new().await {
        Ok(v) => v,
        Err(e) => {
            error!("Unable to log into vault : {}", e);
            std::process::exit(1);
        }
    };

    loop {
        trace!("Running loop.");
        match run(&vault).await {
            Ok(_) => { },
            Err(e) => {
                error!("Error: {}", e);
            }
        };
        thread::sleep(SLEEP_TIME);
    }
}


async fn run(vault: &Vault) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Starting annotation collection.");

    let annotations = get_annotations().await?;

    if ! annotations.contains_key("vault-injector.io/config") {
        debug!("No vault-injector.io/config annotation found.");
        return Ok(());
    }
    let config_str = annotations.get("vault-injector.io/config").unwrap();

    let config = parse_config(config_str)?;

    debug!("Config: {:?}", config);

    let resolved = vault.resolve_env(&config).await?;

    info!("Resolved {} variables to inject.", resolved.len());

    set_env(&resolved).await?;

    Ok(())
}
