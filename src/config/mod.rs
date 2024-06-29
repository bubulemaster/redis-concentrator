use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Sentinels {
    pub address: Vec<String>,
    #[serde(default = "default_sentinel_check_freqency_default")]
    pub check_freqency: u64
}

/// Config structure of RedConcentrator
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Config {
    pub bind: String,
    pub group_name: String,
    #[serde(default)]
    pub sentinels: Option<Sentinels>,
    #[serde(default = "ConfigLog::default")]
    pub log: ConfigLog,
    #[serde(default = "ConfigTimeout::default")]
    pub timeout: ConfigTimeout
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ConfigLog {
    #[serde(default = "default_file")]
    pub file: String,
    #[serde(default = "default_logo")]
    pub logo: bool,
}

impl ConfigLog {
    pub fn default() -> Self {
        Self {
            file: String::from("log4rs.yml"),
            logo: true,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ConfigTimeout {
    #[serde(default = "default_timeout")]
    pub sentinels: u64
}

impl ConfigTimeout {
    pub fn default() -> Self {
        Self {
            sentinels: 5000,
        }
    }
}

// Call by serde to have default value.
fn default_file() -> String {
    String::from("log4rs.yml")
}

// Call by serde to have default value.
fn default_logo() -> bool {
    true
}

// Default value
fn default_sentinel_check_freqency_default() -> u64 {
    1000
}

// Default value
fn default_timeout() -> u64 {
    5000
}

///
/// Return config structure.
///
pub fn get_config(filename: String) -> Result<Config, Error> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    
    file.read_to_string(&mut contents)?;

    match serde_yaml2::from_str(&contents) {
        Ok(deserialized_config) => Ok(deserialized_config),
        Err(err) => Err(Error::new(
            ErrorKind::Other,
            format!("File format of config file is wrong, {}!", err),
        )),
    }
}
