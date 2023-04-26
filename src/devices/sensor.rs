use serde::{Deserialize, Serialize};

use super::DeviceModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensor {
  model: DeviceModel,
  name: String,
  icon: String,
}
