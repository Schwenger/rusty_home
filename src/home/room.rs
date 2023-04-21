use serde::{Deserialize, Serialize};

use crate::devices::{Light, Sensor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
  name: String,
  lights: Vec<Light>,
  sensors: Vec<Sensor>,
}

impl Room {
  pub fn new(name: String) -> Self {
    Room { name, lights: vec![], sensors: vec![] }
  }
}
