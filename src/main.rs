mod k8s;
use k8s::get_pod_annotations;


#[tokio::main]
async fn main() {
    let annotations = match get_pod_annotations().await {
        Ok(data) => data,
        Err(e) => {
            println!("Error: {}", e);
            std::process::exit(1);
        }
    };

    dbg!(annotations);
}
