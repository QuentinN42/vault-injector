use std::error::Error;

use log::{debug, trace, warn};

use crate::config::Config;

#[derive(Debug)]
pub struct Env {
    pub name: String,
    pub value: String,
}

pub fn resolve_env(_config: &Config) -> Result<Vec<Env>, Box<dyn Error>> {
    trace!("Resolving env variables.");
    let mut env: Vec<Env> = vec![];
    let mut seen_names: Vec<String> = vec![];

    for e in &_config.env {
        debug!("Resolving env variable: {}", e.name);
        env.push(Env {
            name: e.name.clone(),
            value: get_secret(&e.engine, &e.secret, &e.field).unwrap(),
        });
        if seen_names.contains(&e.name) {
            warn!("Duplicate env variable name: {}", e.name);
        }
        seen_names.push(e.name.clone());
    }

    Ok(env)
}

fn get_secret(_engine: &str, _secret: &str, _field: &str) -> Result<String, Box<dyn Error>> {
    Ok(String::from("test"))
}
