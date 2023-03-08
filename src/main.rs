use log::{debug, error, trace};

mod config;
mod injector;
mod k8s;
mod vault;

#[tokio::main]
async fn main() {
    env_logger::init();

    trace!("Logger init.");
    debug!("Running with version {}", clap::crate_version!());

    let injector = match injector::Injector::new().await {
        Ok(v) => v,
        Err(e) => {
            error!("Unable to init the injector : {}", e);
            std::process::exit(1);
        }
    };

    injector.run().await;
}
