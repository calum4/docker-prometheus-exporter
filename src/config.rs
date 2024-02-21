use lazy_static::lazy_static;
use serde::{Deserialize};
use std::{env, fs};

#[derive(Debug)]
pub struct ConfigEnv {
    pub config_path: Option<String>,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Config {
}

lazy_static! {
    pub static ref CONFIG_ENV: ConfigEnv = get_config_env();
    pub static ref CONFIG: Config = get_config();
}

fn get_config() -> Config {
    let config_file = CONFIG_ENV.config_path.clone().unwrap_or(String::from("./config.yaml"));

    let reader = match fs::read_to_string(&config_file) {
        Ok(reader) => reader,
        Err(error) => {
            panic!("Unable to read config at '{config_file}', received error: {error}");
        }
    };

    match serde_yaml::from_str(reader.as_str()) {
        Err(error) => {
            panic!("Unable to parse config at '{config_file}', received error: {error}");
        }
        Ok(config) => config,
    }
}

fn get_config_env() -> ConfigEnv {
    ConfigEnv {
        config_path: env::var("CONFIG_PATH").ok(),
    }
}

fn load_env_var(name: &str) -> String {
    match env::var(name) {
        Ok(var) => var,
        Err(error) => {
            panic!("Failed to load environment variable '{name}'. {error}");
        }
    }
}
