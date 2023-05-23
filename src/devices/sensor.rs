use std::collections::VecDeque;

use chrono::{DateTime, Local, TimeZone};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
  api::topic::DeviceKind,
  convert::{StateFromMqtt, StateToMqtt},
};

use super::{Capability, DeviceModel, DeviceTrait};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensor {
  model: DeviceModel,
  name: String,
  icon: String,
  room: String,
  #[serde(skip)]
  states: VecDeque<SensorState>,
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

  fn update_state(&mut self, state: StateFromMqtt) {
    let mut new = self.states.back().cloned().unwrap_or_default();
    new.with_mqtt_state(self.model(), state);
    self.states.push_back(new);
    if self.states.len() > 100 {
      self.states.pop_front();
    }
  }

  fn query_state(&self) -> StateToMqtt {
    if let Some(state) = self.states.back() {
      state.to_mqtt_state(self.model)
    } else {
      SensorState::default().to_mqtt_state(self.model)
    }
  }

  fn query_update(&self) -> StateToMqtt {
    StateToMqtt::empty().with_battery_query()
  }

  fn query_history(&self) -> Vec<StateToMqtt> {
    self.states.iter().map(|s| s.to_mqtt_state(self.model)).collect()
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct SensorState {
  time: DateTime<Local>,
  active: bool,
  humidity: f64,
  temp: f64,
  occupancy: bool,
}

impl SensorState {
  pub fn with_mqtt_state(&mut self, model: DeviceModel, state: StateFromMqtt) {
    if model.capable_of(Capability::State) {
      self.active = state.state().unwrap();
    }
    if model.capable_of(Capability::Humidity) {
      self.humidity = state.humidity.unwrap();
    }
    if model.capable_of(Capability::Temperature) {
      self.temp = state.temperature.unwrap();
    }
    if model.capable_of(Capability::Occupancy) {
      self.occupancy = state.occupancy.unwrap();
    }
    self.time = Local::now();
  }

  pub fn to_mqtt_state(&self, model: DeviceModel) -> StateToMqtt {
    let mut res = StateToMqtt::empty();
    if model.capable_of(Capability::State) {
      res = res.with_state(Some(self.active));
    }
    if model.capable_of(Capability::Humidity) {
      res = res.with_humidity(self.humidity);
    }
    if model.capable_of(Capability::Temperature) {
      res = res.with_temperature(self.temp);
    }
    if model.capable_of(Capability::Occupancy) {
      res = res.with_occupancy(self.occupancy);
    }
    res = res.with_time(self.time);
    res
  }
}

impl Default for SensorState {
  fn default() -> Self {
    Self { time: Local::now(), active: false, humidity: 0.0, temp: 0.0, occupancy: false }
  }
}

impl From<Value> for SensorState {
  fn from(value: Value) -> Self {
    let time = value
      .get("time")
      .and_then(Value::as_i64)
      .and_then(|v| Local.timestamp_millis_opt(v).earliest())
      .unwrap_or(Local::now());
    let active = value.get("state").map(|v| v == "ON").unwrap_or(false);
    let humidity = value.get("humid").and_then(Value::as_f64).unwrap_or(0.0);
    let temp = value.get("temperature").and_then(Value::as_f64).unwrap_or(0.0);
    let occupancy = value.get("occupancy").and_then(Value::as_bool).unwrap_or(false);
    SensorState { time, active, humidity, temp, occupancy }
  }
}
