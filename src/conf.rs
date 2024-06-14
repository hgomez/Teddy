use failure::Error;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Fail)]
enum ConfigurationError {
    #[fail(display = "configuration file not found")]
    FileNotFound,
    #[fail(display = "error parsing configuration file")]
    ParsingError {
        #[cause]
        cause: Error,
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
        .map_err(|_| ConfigurationError::FileNotFound.into())
        .and_then(|file| {
            parse_configuration(file).map_err(|cause| ConfigurationError::ParsingError {
                cause: cause.into(),
            })
        }) {
        Ok(configuration) => configuration,
        Err(error) => {
            warn!("Error loading configuration : {}", error);
            info!("Loading default configuration");
            Configuration::default()
        }
    }
}

fn parse_configuration(file: File) -> Result<Configuration, Error> {
    let reader = BufReader::new(file);
    let result: Configuration = serde_json::from_reader(reader)?;
    Ok(result)
}

pub fn get_address(config: &Configuration) -> String {
    format!("{}:{}", config.host, config.port)
}
