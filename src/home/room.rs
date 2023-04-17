use serde::{Deserialize, Serialize};

use crate::devices::{Light, Sensor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
  pub name: String,
  pub lights: Vec<Light>,
  pub sensors: Vec<Sensor>,
}
