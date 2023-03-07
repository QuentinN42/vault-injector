use log::{debug, error, info, trace};

mod k8s;
use k8s::{get_annotations, set_env};

mod config;
use config::parse_config;

mod vault;
use vault::resolve_env;

#[tokio::main]
async fn main() {
    env_logger::init();

    loop {
        trace!("Running loop.");
        match run().await {
            Ok(_) => { },
            Err(e) => {
                error!("Error: {}", e);
            }
        };
    }
}


async fn run() -> Result<(), Box<dyn std::error::Error>> {
    debug!("Starting annotation collection.");

    let annotations = get_annotations().await?;

    if ! annotations.contains_key("vault-injector.io/config") {
        debug!("No vault-injector.io/config annotation found.");
        return Ok(());
    }
    let config_str = annotations.get("vault-injector.io/config").unwrap();

    let config = parse_config(config_str)?;

    debug!("Config: {:?}", config);

    let resolved = resolve_env(&config)?;

    info!("Resolved {} variables to inject.", resolved.len());

    set_env(&resolved).await?;

    Ok(())
}
