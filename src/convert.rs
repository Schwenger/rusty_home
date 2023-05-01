use palette::{FromColor, Hsv, IntoColor};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
  api::topic::Topic,
  common::{Scalar, Tertiary},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]

pub struct Hue {
  inner: Scalar,
}

impl Hue {
  pub fn from_hsv_radians(hue: f32) -> Self {
    use std::f32::consts::PI;
    let hue = (hue + PI) / (2.0 * PI);
    Self { inner: Scalar::from(hue as f64) }
  }

  pub fn from_rest(val: f64) -> Self {
    Self { inner: Scalar::from(val) }
  }

  pub fn from_mqtt(val: f64) -> Self {
    Self { inner: Scalar::from(val / 360.0) }
  }

  pub fn to_hsv(&self) -> f32 {
    self.inner.inner() as f32
  }

  pub fn to_rest(&self) -> Scalar {
    self.inner
  }

  pub fn to_mqtt(&self) -> f64 {
    self.inner.inner() * 360.0
  }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sat {
  inner: Scalar,
}

impl Sat {
  pub fn from_hsv(sat: f32) -> Self {
    Self { inner: Scalar::from(sat as f64) }
  }

  pub fn from_rest(val: f64) -> Self {
    Self { inner: Scalar::from(val) }
  }

  pub fn from_mqtt(val: f64) -> Self {
    Self { inner: Scalar::from(val / 100.0) }
  }

  pub fn to_hsv(&self) -> f32 {
    self.inner.inner() as f32
  }

  pub fn to_rest(&self) -> Scalar {
    self.inner
  }

  pub fn to_mqtt(&self) -> f64 {
    self.inner.inner() * 100.0
  }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Val {
  inner: Scalar,
}

impl Val {
  pub fn from_hsv(val: f32) -> Self {
    Self { inner: Scalar::from(val as f64) }
  }

  pub fn from_mqtt(val: f64) -> Self {
    Self { inner: Scalar::from(val / 254.0) }
  }

  pub fn from_rest(val: f64) -> Self {
    Self { inner: Scalar::from(val) }
  }

  pub fn to_hsv(&self) -> f32 {
    self.inner.inner() as f32
  }

  pub fn to_rest(&self) -> Scalar {
    self.inner
  }

  pub fn to_mqtt(&self) -> f64 {
    self.inner.inner() * 254.0
  }
}

#[allow(missing_copy_implementations)] // Avoid accidental copying.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct HsvColor {
  hue: Hue,
  sat: Sat,
  val: Val,
}

impl Default for HsvColor {
  fn default() -> Self {
    Self { hue: Hue::from_hsv_radians(0.0), sat: Sat::from_hsv(0.0), val: Val::from_hsv(0.33) }
  }
}

impl HsvColor {
  pub fn new(hue: Hue, sat: Sat, val: Val) -> Self {
    Self { hue, sat, val }
  }

  pub fn as_color<T: FromColor<Hsv>>(&self) -> T {
    Hsv::new(self.hue.to_hsv(), self.sat.to_hsv(), self.val.to_hsv()).into_color()
  }

  pub fn val(&self) -> Val {
    self.val
  }

  pub fn with_val(&mut self, val: Val) {
    self.val = val
  }

  pub fn sat(&self) -> Sat {
    self.sat
  }

  pub fn with_sat(&mut self, sat: Sat) {
    self.sat = sat
  }

  pub fn hue(&self) -> Hue {
    self.hue
  }

  pub fn with_hue(&mut self, hue: Hue) {
    self.hue = hue
  }
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq)]
pub struct StateFromMqtt {
  #[serde(default)]
  brightness: Option<f64>,
  #[serde(default)]
  color: Option<MqttColorIn>,
  #[serde(default)]
  state: Option<MqttOnOff>,
  #[serde(default)]
  pub temperature: Option<f64>,
  #[serde(default)]
  pub humidity: Option<f64>,
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq)]
pub struct MqttColorIn {
  hue: f64,
  saturation: f64,
}

impl StateFromMqtt {
  pub fn hsv_color(&self) -> Option<HsvColor> {
    if self.val().is_none() || self.hue().is_none() || self.sat().is_none() {
      return None;
    }
    Some(HsvColor::new(self.hue().unwrap(), self.sat().unwrap(), self.val().unwrap()))
  }

  pub fn val(&self) -> Option<Val> {
    self.brightness.map(Val::from_mqtt)
  }

  pub fn hue(&self) -> Option<Hue> {
    self.color.map(|c| Hue::from_mqtt(c.hue))
  }

  pub fn sat(&self) -> Option<Sat> {
    self.color.map(|c| Sat::from_mqtt(c.saturation))
  }

  pub fn state(&self) -> Option<bool> {
    self.state.map(|v| v == MqttOnOff::On)
  }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum MqttOnOff {
  On,
  Off,
}

#[derive(Debug, Clone, Default)]
pub struct StateToMqtt {
  brightness: Tertiary<Val>,
  color: Option<MqttColorOut>,
  state: Tertiary<MqttOnOff>,
  transition: Option<i8>,
  brightness_move: Option<i8>,
  battery: Option<()>,
  temperature: Option<f64>,
  humidity: Option<f64>,
}

impl StateToMqtt {
  const TRANSITION: i8 = 3;
  const DIM_SPEED: i8 = 40;

  pub fn to_json_str(self, rest: bool) -> String {
    let mut obj = json!({});

    let (label, brightness) = if rest {
      ("val", self.brightness.map(|v| v.to_rest()).to_json())
    } else {
      ("brightness", self.brightness.map(|v| v.to_mqtt()).to_json())
    };
    if let Some(json) = brightness {
      assert_eq!(self.color, None);
      obj.as_object_mut().unwrap().insert(String::from(label), json);
    }

    if let Some(color) = self.color {
      let mut col_obj = json!({});
      if rest {
        col_obj.as_object_mut().unwrap().insert(String::from("hue"), json!(color.hue.to_rest()));
        col_obj.as_object_mut().unwrap().insert(String::from("sat"), json!(color.sat.to_rest()));
        // Value on object level.
        obj.as_object_mut().unwrap().insert(String::from("val"), json!(color.val.to_rest()));
      } else {
        col_obj.as_object_mut().unwrap().insert(String::from("h"), json!(color.hue.to_mqtt()));
        col_obj.as_object_mut().unwrap().insert(String::from("s"), json!(color.sat.to_mqtt()));
        col_obj.as_object_mut().unwrap().insert(String::from("v"), json!(color.val.to_mqtt()));
      }
      obj.as_object_mut().unwrap().insert(String::from("color"), col_obj);
    }

    if let Some(json) = self.state.to_json() {
      obj.as_object_mut().unwrap().insert(String::from("state"), json);
    }

    if let Some(v) = self.transition {
      obj.as_object_mut().unwrap().insert(String::from("transition"), json!(v));
    }

    if let Some(v) = self.brightness_move {
      obj.as_object_mut().unwrap().insert(String::from("brightness_move"), json!(v));
    }

    if self.battery.is_some() {
      obj.as_object_mut().unwrap().insert(String::from("battery"), json!(""));
    }

    if let Some(v) = self.humidity {
      obj.as_object_mut().unwrap().insert(String::from("humidity"), json!(v));
    }

    if let Some(v) = self.temperature {
      obj.as_object_mut().unwrap().insert(String::from("temperature"), json!(v));
    }

    obj.to_string()
  }

  pub fn empty() -> Self {
    Self::default()
  }

  pub fn with_color_change(mut self, color: &HsvColor) -> Self {
    self.color = Some(MqttColorOut::from_hsv(color));
    self
  }

  pub fn with_state(mut self, on: Option<bool>) -> Self {
    self.state = match on {
      Some(true) => Tertiary::Some(MqttOnOff::On),
      Some(false) => Tertiary::Some(MqttOnOff::Off),
      None => Tertiary::Query,
    };
    self
  }

  pub fn with_value(mut self, val: Option<Val>) -> Self {
    self.brightness = match val {
      Some(val) => Tertiary::Some(val),
      None => Tertiary::Query,
    };
    self
  }

  pub fn with_brightness_move(mut self, factor: i8) -> Self {
    self.brightness_move = Some(factor * Self::DIM_SPEED);
    self
  }

  pub fn with_transition(mut self) -> Self {
    self.transition = Some(Self::TRANSITION);
    self
  }

  pub fn with_battery_query(mut self) -> Self {
    self.battery = Some(());
    self
  }

  pub fn with_humidity(mut self, h: f64) -> Self {
    self.humidity = Some(h);
    self
  }

  pub fn with_temperature(mut self, t: f64) -> Self {
    self.temperature = Some(t);
    self
  }
}

#[derive(Debug, Clone, Copy, Serialize, Default, PartialEq)]
pub struct MqttColorOut {
  hue: Hue,
  sat: Sat,
  val: Val,
}

impl MqttColorOut {
  pub fn from_hsv(color: &HsvColor) -> Self {
    MqttColorOut { hue: color.hue(), sat: color.sat(), val: color.val() }
  }
}

#[derive(Debug, Clone, Default)]
pub struct RestApiPayload {
  pub topic: Option<Topic>,
  pub val: Option<Val>,
  pub hue: Option<Hue>,
  pub sat: Option<Sat>,
}
