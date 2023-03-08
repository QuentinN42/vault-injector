use serde::{de::Error, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Env {
    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub field: String,

    pub engine: String,

    pub secret: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub env: Vec<Env>,
}

/// # Parse the config
///
/// This function parses the config string and returns a Config struct.
/// Config is a list of Env structs.
/// Each env struct must define an engine and a secret to be looked up in your vault instance.
/// You can omit the name or the field, but not both, it will so been infered from the other value.
pub fn parse_config(config_str: &str) -> Result<Config, serde_json::Error> {
    let config: Config = serde_json::from_str(config_str)?;

    let mut env: Vec<Env> = vec![];
    for e in config.env {
        if e.name.is_empty() && e.field.is_empty() {
            return Err(serde_json::Error::custom(
                "Either name or field must be set for env",
            ));
        }
        env.push(Env {
            name: if e.name.is_empty() {
                e.field.clone()
            } else {
                e.name.clone()
            },
            field: if e.field.is_empty() {
                e.name.clone()
            } else {
                e.field.clone()
            },
            engine: e.engine,
            secret: e.secret,
        });
    }

    Ok(Config { env })
}
