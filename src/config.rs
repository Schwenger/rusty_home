use serde::{Deserialize, Serialize};

use crate::{controller::Controller, Error, Result};

#[derive(Serialize, Deserialize, Debug)]
pub struct GlobalConfig {
  pub mosquitto: MosquittoConfig,
  pub log: LogConfig,
  pub home: HomeConfig,
}

impl GlobalConfig {
  pub fn read() -> Result<GlobalConfig> {
    let content = std::fs::read_to_string("config/global.yml")?;
    let cfg: GlobalConfig = serde_yaml::from_str(&content).unwrap();
    println!("{:?}", cfg);
    Ok(cfg)
  }
}

impl TryInto<Controller> for GlobalConfig {
  type Error = Error;
  fn try_into(self) -> Result<Controller> {
    Controller::new(self)
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MosquittoConfig {
  pub ip: String,
  pub port: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogConfig {
  pub dir: String,
  pub format: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HomeConfig {
  pub dir: String,
}
