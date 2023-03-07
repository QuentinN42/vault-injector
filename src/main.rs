use log::{debug, error, info};


mod k8s;
use k8s::get_pod_annotations;

mod config;
use config::parse_config;


#[tokio::main]
async fn main() {
    env_logger::init();

    debug!("Starting annotation collection.");

    let annotations = match get_pod_annotations().await {
        Ok(data) => data,
        Err(e) => {
            error!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if ! annotations.contains_key("vault-injector.io/config") {
        info!("No vault-injector.io/config annotation found.");
        debug!("Exiting.");
        std::process::exit(0);
    }
    let config_str = annotations.get("vault-injector.io/config").unwrap();

    let config = match parse_config(config_str) {
        Ok(data) => data,
        Err(e) => {
            error!("Error: {}", e);
            std::process::exit(1);
        }
    };

    dbg!(config);
}
