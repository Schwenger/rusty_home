use crate::devices::DeviceModel;
use crate::Error;
use crate::Result;
use palette::IntoColor;
use palette::{FromColor, Hsv, Yxy};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value as SerdeValue;

use crate::Scalar;

use super::traits::JsonConvertible;

#[derive(Debug, Clone)]
pub struct JsonPayload(String);

impl JsonPayload {
  pub fn from<T: Serialize>(v: &T) -> Self {
    Self(serde_json::to_string(v).expect("Why would this fail?"))
  }
  pub fn inner(&self) -> &str {
    &self.0
  }
  pub fn from_string(inner: String) -> Self {
    Self(inner)
  }
  pub fn to_str(&self) -> String {
    String::from(self.inner())
  }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MqttPayload(SerdeValue);

impl MqttPayload {
  const TRANSITION: u32 = 3;
  const DIM_SPEED: u32 = 40;

  pub fn new() -> Self {
    Self(json!({}))
  }

  pub fn with_transition(mut self) -> Self {
    self.insert("transition", json!(Self::TRANSITION));
    self
  }

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

  pub fn with_battery_query(mut self) -> Self {
    self.insert("battery", json!(""));
    self
  }

  pub fn with_color_change(mut self, color: Hsv) -> Self {
    let color: Yxy = color.into_color();
    let json_color = json!({
      "x": color.x,
      "y": color.y,
    });
    self.insert("color", json_color);
    self.with_brightness(json!(color.luma * 254.0))
  }

  pub fn with_brightness_change(self, val: Scalar) -> Self {
    self.with_brightness(json!(val.inner() * 254.0))
  }

  fn with_brightness(mut self, val: SerdeValue) -> Self {
    self.insert("brightness", val);
    self
  }

  pub fn with_start_dimming(mut self, up: bool) -> Self {
    let f = if up { 1 } else { -1 };
    self.insert("brightness_move", json!(f * (Self::DIM_SPEED as i64)));
    self
  }

  pub fn with_stop_dimming(mut self) -> Self {
    self.insert("brightness_move", json!(0));
    self
  }

  fn insert(&mut self, key: &str, value: SerdeValue) {
    self.0.as_object_mut().unwrap().insert(String::from(key), value);
  }
}

impl Default for MqttPayload {
  fn default() -> Self {
    Self::new()
  }
}

impl JsonConvertible for MqttPayload {
  fn to_json(self) -> JsonPayload {
    JsonPayload::from(&self)
  }

  fn from_str(string: &str) -> Result<Self> {
    serde_json::from_str(string).map_err(|_| Error::UnexpectedMqttPayload)
  }
}

impl MqttPayload {
  pub fn read_color(model: DeviceModel, payload: SerdeValue) -> Hsv {
    let bright = payload.get("brightness").and_then(SerdeValue::as_i64).unwrap();
    let bright = Self::read_brightness_scalar(model, bright);
    let bright = bright.inner() as f32;

    if let Some(color) = payload.get("color") {
      Self::read_color_encoding(color, bright)
    } else {
      Hsv::new_srgb(1.0, 1.0, bright)
    }
  }

  fn read_color_encoding(color: &SerdeValue, brightness: f32) -> Hsv {
    if let Some(x) = color.get("x").and_then(SerdeValue::as_f64) {
      let y = color.get("y").and_then(SerdeValue::as_f64).expect("Error");
      let res: Yxy = Yxy::new(x as f32, y as f32, brightness);
      let res: Hsv = Hsv::from_color(res);
      res
    } else if let Some(hue) = color.get("hue").and_then(SerdeValue::as_i64) {
      let sat = color.get("saturation").and_then(SerdeValue::as_i64).expect("Error");
      let hue = hue as f32 / 360.0;
      let sat = sat as f32 / 100.0;
      Hsv::new_srgb(hue, sat, brightness)
    } else {
      unimplemented!("Unexpected encoding.")
    }
  }

  fn read_brightness_scalar(model: DeviceModel, brightness: i64) -> Scalar {
    let max = match model.vendor() {
      crate::devices::Vendor::Ikea => 254.0,
      crate::devices::Vendor::Philips => 254.0,
      crate::devices::Vendor::Tuya => unreachable!(),
    };
    (brightness as f64 / max).into()
  }
}
