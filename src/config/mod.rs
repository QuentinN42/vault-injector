use serde::{Deserialize, Serialize, de::Error};


#[derive(Serialize, Deserialize, Debug)]
struct Env {

    #[serde(default)]
    name: String,

    #[serde(default)]
    field: String,

    engine: String,

    secret: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    version: String,
    env: Vec<Env>,
}


pub fn parse_config(config_str: &str) -> Result<Config, serde_json::Error> {
    let config: Config = serde_json::from_str(config_str)?;

    let mut env: Vec<Env> = vec![];
    for e in config.env {
        if e.name.is_empty() && e.field.is_empty(){
            return Err(serde_json::Error::custom("Either name or field must be set for env"));
        }
        env.push(Env {
            name: if e.name.is_empty() { e.field.clone() } else { e.name.clone() },
            field: if e.field.is_empty() { e.name.clone() } else { e.field.clone() },
            engine: e.engine,
            secret: e.secret,
        });
    }

    Ok(Config {
        version: config.version,
        env: env,
    })
}
