use config::{Config, File, FileFormat};

#[derive(serde::Deserialize, Debug, Default)]
pub struct Configuration {
    pub application: ApplicationConfiguration,
    pub auth: Option<AuthConfiguration>,
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct ApplicationConfiguration {
    pub port: Option<u16>,
    pub host: Option<String>,
    pub log_lvl: Option<String>,
    pub timeout: Option<u64>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct AuthConfiguration {
    pub username: Option<String>,
    pub password: Option<String>,
}

pub fn get_configuration(config_folder: &str) -> Result<Configuration, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join(config_folder);

    let settings = Config::builder()
        // Start off by merging in the "base" configuration file
        .add_source(
            File::new(
                configuration_directory
                    .join("config")
                    .as_os_str()
                    .to_str()
                    .unwrap(),
                FileFormat::Yaml,
            )
            .required(false),
        )
        // Add in settings from environment variables (with a '__' as separator)
        // E.g. `APPLICATION__PORT=5001 would set `Settings.application.port`
        .add_source(config::Environment::default().separator("__"))
        .build()?;

    // If no configuration is found, return a default configuration
    settings.try_deserialize().or_else(|_| Ok(Configuration::default()))
}
