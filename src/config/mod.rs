extern crate serde_yaml;

use std::io::{Error, ErrorKind, Read};
use std::fs::File;

/// Config structure of Redis Concentrator
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Config {
    pub bind: String,
    pub port: u16,
    pub group_name: String,
    pub sentinels: Option<Vec<String>>,
    #[serde(default = "ConfigLog::default")]
    pub log: ConfigLog
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ConfigLog {
    #[serde(alias = "type")]
    #[serde(default = "default_log_type")]
    pub log_type: String,
    #[serde(default = "default_level")]
    pub level: String,
    pub file: Option<String>,
    #[serde(default = "default_header")]
    pub header: bool
}

impl ConfigLog {
    pub fn default() -> ConfigLog {
        ConfigLog {
            log_type: String::from("console"),
            level: String::from("info"),
            file: None,
            header: true
        }
    }
}

// Call by serde to have default value.
fn default_log_type() -> String {
    String::from("console")
}

// Call by serde to have default value.
fn default_level() -> String {
    String::from("info")
}

// Call by serde to have default value.
fn default_header() -> bool {
    false
}

///
/// Return config structure.
///
pub fn get_config(filename: String) -> Result<Config, Error> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    // let deserialized_config: Config = serde_yaml::from_str(&data).unwrap();
    //
    // Ok(deserialized_config)

    match serde_yaml::from_str(&contents) {
        Ok(deserialized_config) => Ok(deserialized_config),
        Err(err) => Err(Error::new(
            ErrorKind::Other,
            format!("File format of config file is wrong, {}!", err),
        )),
    }
}