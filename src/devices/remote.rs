use serde::{Serialize, Deserialize};

use super::DeviceModel;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remote {
  name: String,
  model: DeviceModel,
  icon: String,
  controls: String,
  actions: Vec<String>,
}