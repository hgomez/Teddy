use thiserror::Error;
use log::{info, warn};
use serde::Deserialize;
use serde_json::from_reader;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("configuration file not found")]
    FileNotFound,

    #[error("error parsing configuration file")]
    ParsingError {
        #[from]
        source: serde_json::Error, // On utilise directement l'erreur de serde_json
    },
}

#[derive(Deserialize, Clone)]
pub struct Configuration {
    #[serde(default = "Configuration::default_host")]
    pub host: String,
    #[serde(default = "Configuration::default_port")]
    pub port: u16,
    pub user: String,
    pub password: String,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            host: Configuration::default_host(),
            port: Configuration::default_port(),
            user: String::from("teddy"),
            password: String::from("rocks"),
        }
    }
}

impl Configuration {
    fn default_host() -> String {
        String::from("0.0.0.0")
    }

    fn default_port() -> u16 {
        3000
    }
}

const CONFIG_PATH: &'static str = "config.json";

pub fn load_config() -> Configuration {
    match File::open(CONFIG_PATH)
        .map_err(|_| ConfigurationError::FileNotFound)
        .and_then(|file| {
            // Ici, parse_configuration renvoie déjà une ConfigurationError
            parse_configuration(file)
        }) {
        Ok(configuration) => configuration,
        Err(error) => {
            warn!("Error loading configuration : {}", error);
            info!("Loading default configuration");
            Configuration::default()
        }
    }
}

// Changement ici : on retourne ConfigurationError au lieu du type inexistant Error
fn parse_configuration(file: File) -> Result<Configuration, ConfigurationError> {
    let reader = BufReader::new(file);
    // L'opérateur ? convertit automatiquement serde_json::Error vers ConfigurationError::ParsingError
    let result: Configuration = from_reader(reader)?;
    Ok(result)
}

pub fn get_address(config: &Configuration) -> String {
    format!("{}:{}", config.host, config.port)
}
