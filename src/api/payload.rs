use crate::Error;
use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value as SerdeValue;

use crate::Scalar;

use super::JsonConvertible;

pub struct JsonPayload(String);

impl JsonPayload {
  pub fn from<T: Serialize>(v: &T) -> Self {
    Self(serde_json::to_string(v).expect("Why would this fail?"))
  }
  pub fn inner(&self) -> &str {
    &self.0
  }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MqttPayload(SerdeValue);

impl MqttPayload {
  const TRANSITION: u32 = 50;

  pub fn new() -> Self {
    Self(json!({}))
  }

  pub fn with_transition(mut self) -> Self {
    self.insert("transition", json!(Self::TRANSITION));
    self
  }

  #[allow(dead_code)]
  pub fn with_state_query(self) -> Self {
    self.with_state(json!(""))
  }

  pub fn with_state_change(self, on: bool) -> Self {
    self.with_state(json!(if on { "ON" } else { "OFF" }))
  }

  fn with_state(mut self, val: SerdeValue) -> Self {
    self.insert("state", val);
    self
  }

  #[allow(dead_code)]
  pub fn with_brightness_query(self) -> Self {
    self.with_brightness(json!(""))
  }

  pub fn with_brightness_change(self, val: Scalar) -> Self {
    self.with_brightness(json!(val.inner()))
  }

  fn with_brightness(mut self, val: SerdeValue) -> Self {
    self.insert("brightness", val);
    self
  }

  fn insert(&mut self, key: &str, value: SerdeValue) {
    self.0.as_object_mut().unwrap().insert(String::from(key), value);
  }
}

impl JsonConvertible for MqttPayload {
  fn to_json(&self) -> JsonPayload {
    JsonPayload::from(self)
  }

  fn from_str(string: &str) -> Result<Self> {
    serde_json::from_str(string).map_err(|_| Error::UnexpectedMqttPayload)
  }
}
