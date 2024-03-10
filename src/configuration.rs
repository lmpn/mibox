use config::Config;
use serde_aux::field_attributes::deserialize_number_from_string;
use std::str::FromStr;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub app_name: String,
    pub log_level: String,
}

impl ApplicationSettings {
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Add configuration values from a file named `configuration`. // It will look for any top-level file with an extension
    // that `config` knows how to parse: yaml, json, etc.
    let base_path = std::env::current_dir().expect("unable to find current dir");
    let configuration_path = base_path.join("configuration");

    let builder = Config::builder()
        .add_source(config::File::from(configuration_path.join("base")).required(true));

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".to_string())
        .parse()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let builder = builder
        .add_source(
            config::File::from(configuration_path.join(environment.as_str())).required(true),
        )
        .add_source(config::Environment::with_prefix("app").separator("__"));
    // Try to convert the configuration values it read into
    // our Settings type
    let settings = builder.build()?;
    settings.try_deserialize()
}

enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
