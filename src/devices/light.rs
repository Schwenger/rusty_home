use serde::{Deserialize, Serialize};

use super::DeviceModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
  name: String,
  model: DeviceModel,
  icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightGroup {
  name: String,
  atomics: Vec<Light>,
  subgroups: Vec<LightGroup>,
}

impl LightGroup {
  pub fn new(name: String) -> Self {
    LightGroup { name, atomics: vec![], subgroups: vec![] }
  }
}
