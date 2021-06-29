use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::time::Duration;
use toml;

lazy_static! {
    static ref CONFIG: Arc<Config> = match Config::new() {
        Ok(val) => Arc::new(val),
        Err(e) => panic!("Config initialization error: {}", e.to_string()),
    };
}

#[derive(Serialize, Deserialize)]
pub struct Package {
    pub package: Version,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Version {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Exec {
    pub timeout: u64,
    pub timeout_duration: Duration,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Config {
    pub version: Version,
    pub exec: Exec,
}

impl Config {
    pub fn new() -> Result<Config, String> {
        let mut file = match File::open("Cargo.toml") {
            Ok(file) => file,
            Err(e) => return Err(format!("Could not open Cargo.toml. Err: {}", e.to_string())),
        };

        let mut manifest_str = String::new();
        match file.read_to_string(&mut manifest_str) {
            Ok(_) => (),
            Err(e) => return Err(format!("Could not read Cargo.toml. Err: {}", e.to_string())),
        };

        let mut manifest: Package = match toml::from_str(&manifest_str) {
            Ok(data) => data,
            Err(e) => {
                return Err(format!(
                    "Something wrong with manifest formatting. Err: {}",
                    e.to_string()
                ))
            }
        };

        let timeout: u64 = match std::env::var_os("TIMEOUT_S") {
            Some(val) => val.to_str().unwrap().parse::<u64>().unwrap(),
            None => 1,
        };

        let exec = Exec {
            timeout: timeout,
            timeout_duration: Duration::from_secs(timeout),
        };

        manifest.package.version = match std::env::var_os("TAG") {
            Some(val) => String::from(val.to_str().unwrap()),
            None => manifest.package.version,
        };

        let version: Version = Version {
            name: manifest.package.name,
            version: manifest.package.version,
        };

        let config: Config = Config {
            version: version,
            exec: exec,
        };

        Ok(config)
    }

    pub fn get_current<'a>() -> &'a Config {
        &CONFIG
    }
}
