use serde::{Deserialize, Serialize};
use std::{error::Error, fs};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Empty for now, can be extended later
}

impl Default for Config {
    fn default() -> Self {
        Config {}
    }
}

impl Config {
    pub fn load() -> Result<Config, Box<dyn Error>> {
        let config_path = dirs::home_dir()
            .ok_or("Could not find home directory")?
            .join(".logview.yml");

        if !config_path.exists() {
            let default_config = Config::default();
            let yaml = serde_yaml::to_string(&default_config)?;
            fs::write(&config_path, yaml)?;
            return Ok(default_config);
        }

        let contents = fs::read_to_string(&config_path)?;
        match serde_yaml::from_str::<Config>(&contents) {
            Ok(config) => Ok(config),
            Err(_) => {
                let default_config = Config::default();
                let yaml = serde_yaml::to_string(&default_config)?;
                fs::write(&config_path, yaml)?;
                Ok(default_config)
            }
        }
    }
}