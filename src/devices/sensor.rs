use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
  api::{payload::MqttPayload, topic::DeviceKind},
  mqtt::MqttState,
};

use super::{Capability, DeviceModel, DeviceTrait};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensor {
  model: DeviceModel,
  name: String,
  icon: String,
  room: String,
  #[serde(skip)]
  state: SensorState,
}

impl DeviceTrait for Sensor {
  fn virtual_kind(&self) -> DeviceKind {
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

  fn update_state(&mut self, state: MqttState) {
    self.state.with_mqtt_state(self.model(), state);
  }

  fn query_state(&self) -> MqttState {
    self.state.to_mqtt_state(self.model)
  }

  fn query_update(&self) -> MqttPayload {
    MqttPayload::new().with_state_query()
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct SensorState {
  active: bool,
  humidity: f64,
  temp: f64,
}

impl SensorState {
  pub fn with_mqtt_state(&mut self, model: DeviceModel, state: MqttState) {
    if model.capable_of(Capability::State) {
      self.active = state.state.unwrap().into();
    }
    if model.capable_of(Capability::Humidity) {
      self.humidity = state.humidity.unwrap();
    }
    if model.capable_of(Capability::Temperature) {
      self.temp = state.temperature.unwrap();
    }
  }

  pub fn to_mqtt_state(&self, model: DeviceModel) -> MqttState {
    let mut res = MqttState::default();
    if model.capable_of(Capability::State) {
      res.state = Some(self.active.into());
    }
    if model.capable_of(Capability::Humidity) {
      res.humidity = Some(self.humidity);
    }
    if model.capable_of(Capability::Temperature) {
      res.temperature = Some(self.temp);
    }
    res
  }
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
