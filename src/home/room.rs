use serde::{Deserialize, Serialize};

use crate::devices::{Sensor, LightGroup, Remote};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
  name: String,
  lights: LightGroup,
  icon: String,
  sensors: Vec<Sensor>,
  remotes: Vec<Remote>,
}

impl Room {
  pub fn new(name: String) -> Self {
    Room { name, lights: LightGroup::new("Main".to_string()), sensors: vec![], remotes: vec![], icon: String::from("square.split.bottomrightquarter.fill") }
  }
}
