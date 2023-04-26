use serde::{Deserialize, Serialize};

use crate::{
  api::traits::LightCollection,
  devices::{LightGroup, Remote, Sensor},
};

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
    Room {
      name,
      lights: LightGroup::new("Main".to_string()),
      sensors: vec![],
      remotes: vec![],
      icon: String::from("square.split.bottomrightquarter.fill"),
    }
  }
}

impl LightCollection for Room {
  fn flatten_lights(&self) -> Vec<&crate::devices::Light> {
    self.lights.flatten_lights()
  }

  fn flatten_lights_mut(&mut self) -> Vec<&mut crate::devices::Light> {
    self.lights.flatten_lights_mut()
  }
}
