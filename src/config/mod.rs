use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use serde::{Serialize, Deserialize};

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
    pub timeout: ConfigTimeout,
    #[serde(default = "ConfigWorker::default")]
    pub workers: ConfigWorker
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Sentinels {
    pub address: Vec<String>,
    #[serde(default = "default_sentinel_check_freqency_default")]
    pub check_freqency: u64
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
    pub sentinels: u64,
    #[serde(default = "default_timeout")]
    pub worker_idle_timeout: u64    
}

impl ConfigTimeout {
    pub fn default() -> Self {
        Self {
            sentinels: default_timeout(),
            worker_idle_timeout: default_timeout()
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ConfigWorker {
    #[serde(default = "ConfigWorkerPool::default")]
    pub pool: ConfigWorkerPool
}

impl ConfigWorker {
    pub fn default() -> Self {
        Self {
            pool: ConfigWorkerPool::default(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ConfigWorkerPool {
    #[serde(default = "default_pool_size_min")]
    pub min: u8,
    #[serde(default = "default_pool_size_max")]
    pub max: u8,
}

impl ConfigWorkerPool {
    pub fn default() -> Self {
        Self {
            min: 5,
            max: 10
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

// Default value
fn default_pool_size_min() -> u8 {
    5
}
// Default value
fn default_pool_size_max() -> u8 {
    10
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
