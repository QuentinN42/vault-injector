mod k8s;
use k8s::get_pod_annotations;
use log::{debug, error, info};


#[tokio::main]
async fn main() {
    env_logger::init();

    debug!("Starting annotation collection.\n");

    let annotations = match get_pod_annotations().await {
        Ok(data) => data,
        Err(e) => {
            error!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if ! annotations.contains_key("vault-injector.io/config") {
        info!("No vault-injector.io/config annotation found.\n");
        debug!("Exiting.\n");
        std::process::exit(0);
    }

    dbg!(annotations);
}
