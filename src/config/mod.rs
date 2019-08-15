extern crate serde_yaml;

use std::io::{Error, ErrorKind, Read};
use std::fs::File;

/// Config structure of D-SH
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Config {
    pub bind: String,
    pub port: u16,
    pub group_name: String,
    pub sentinels: Option<Vec<String>>
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