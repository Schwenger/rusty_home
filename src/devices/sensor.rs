use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::topic::DeviceKind;

use super::{Device, DeviceModel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensor {
  model: DeviceModel,
  name: String,
  icon: String,
  room: String,
  #[serde(skip)]
  state: SensorState,
}

impl Sensor {
  pub fn update_state(&mut self, state: SensorState) {
    self.state = state;
  }
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

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct SensorState {
  active: bool,
  humidity: f64,
  temp: f64,
}

impl Default for SensorState {
  fn default() -> Self {
    Self { active: false, humidity: 0.0, temp: 0.0 }
  }
}

impl From<Value> for SensorState {
  fn from(value: Value) -> Self {
    let active = value.get("state").map(|v| v == "ON").unwrap_or(false);
    let humidity = value.get("humid").and_then(Value::as_f64).unwrap_or(0.0);
    let temp = value.get("temperature").and_then(Value::as_f64).unwrap_or(0.0);
    SensorState { active, humidity, temp }
  }
}
