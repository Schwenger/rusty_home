use serde::{Deserialize, Serialize};

use crate::api::topic::DeviceKind;

use super::{Device, DeviceModel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensor {
  model: DeviceModel,
  name: String,
  icon: String,
  room: String,
}

impl Device for Sensor {
  fn kind(&self) -> DeviceKind {
    DeviceKind::Sensor
  }

  fn model(&self) -> DeviceModel {
    self.model
  }

  fn name(&self) -> &str {
    &self.name
  }

  fn room(&self) -> &str {
    &self.room
  }
}
