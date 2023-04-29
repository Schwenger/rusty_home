pub mod light;
pub mod remote;
pub mod sensor;

pub use light::{Light, LightGroup};
pub use remote::Remote;
pub use sensor::Sensor;
use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize};

use crate::api::{
  topic::{DeviceKind, Topic, TopicMode},
  traits::Addressable,
};

pub trait DeviceTrait: Addressable {
  fn virtual_kind(&self) -> DeviceKind;
  fn model(&self) -> DeviceModel;
  fn name(&self) -> &str;
  fn room(&self) -> &str;
  fn physical_kind(&self) -> DeviceKind {
    self.model().kind()
  }
}

#[derive(Debug, Clone)]
pub enum Device {
  Light(Light),
  Sensor(Sensor),
  Remote(Remote),
}

impl Device {
  pub fn as_light(&self) -> Option<&Light> {
    match self {
      Device::Light(l) => Some(l),
      Device::Sensor(_) | Device::Remote(_) => None,
    }
  }
  pub fn as_light_mut(&mut self) -> Option<&mut Light> {
    match self {
      Device::Light(l) => Some(l),
      Device::Sensor(_) | Device::Remote(_) => None,
    }
  }
  pub fn as_sensor(&self) -> Option<&Sensor> {
    match self {
      Device::Sensor(s) => Some(s),
      Device::Light(_) | Device::Remote(_) => None,
    }
  }
  pub fn as_sensor_mut(&mut self) -> Option<&mut Sensor> {
    match self {
      Device::Sensor(s) => Some(s),
      Device::Light(_) | Device::Remote(_) => None,
    }
  }
  pub fn as_remote(&self) -> Option<&Remote> {
    match self {
      Device::Remote(r) => Some(r),
      Device::Light(_) | Device::Sensor(_) => None,
    }
  }
  pub fn as_remote_mut(&mut self) -> Option<&mut Remote> {
    match self {
      Device::Remote(r) => Some(r),
      Device::Light(_) | Device::Sensor(_) => None,
    }
  }
  fn inner(&self) -> &dyn DeviceTrait {
    match self {
      Device::Light(l) => l,
      Device::Sensor(s) => s,
      Device::Remote(r) => r,
    }
  }
}

impl From<Light> for Device {
  fn from(value: Light) -> Self {
    Device::Light(value)
  }
}

impl From<Sensor> for Device {
  fn from(value: Sensor) -> Self {
    Device::Sensor(value)
  }
}

impl From<Remote> for Device {
  fn from(value: Remote) -> Self {
    Device::Remote(value)
  }
}

impl DeviceTrait for Device {
  fn virtual_kind(&self) -> DeviceKind {
    self.inner().virtual_kind()
  }

  fn model(&self) -> DeviceModel {
    self.inner().model()
  }

  fn name(&self) -> &str {
    self.inner().name()
  }

  fn room(&self) -> &str {
    self.inner().room()
  }
}

impl<T: DeviceTrait> Addressable for T {
  fn topic(&self, mode: TopicMode) -> Topic {
    Topic::Device {
      device: self.physical_kind(),
      room: self.room().to_string(),
      groups: vec![],
      name: self.name().to_string(),
      mode,
    }
  }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceModel {
  TuyaHumidity,
  IkeaOutlet,
  IkeaDimmable,
  HueColor,
  IkeaMultiButton,
  IkeaDimmer,
}

impl DeviceModel {
  pub fn kind(&self) -> DeviceKind {
    match self {
      DeviceModel::TuyaHumidity => DeviceKind::Sensor,
      DeviceModel::IkeaOutlet => DeviceKind::Outlet,
      DeviceModel::IkeaDimmable => DeviceKind::Light,
      DeviceModel::HueColor => DeviceKind::Light,
      DeviceModel::IkeaMultiButton => DeviceKind::Remote,
      DeviceModel::IkeaDimmer => DeviceKind::Remote,
    }
  }

  pub fn vendor(&self) -> Vendor {
    match self {
      DeviceModel::TuyaHumidity => Vendor::Tuya,
      DeviceModel::IkeaOutlet
      | DeviceModel::IkeaDimmable
      | DeviceModel::IkeaMultiButton
      | DeviceModel::IkeaDimmer => Vendor::Ikea,
      DeviceModel::HueColor => Vendor::Philips,
    }
  }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Vendor {
  Ikea,
  Philips,
  Tuya,
}

pub fn serialize_light_sequence<S>(val: &Vec<Device>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer,
{
  let mut seq = serializer.serialize_seq(Some(val.len()))?;
  for e in val {
    seq.serialize_element(e.as_light().unwrap())?;
  }
  seq.end()
}

pub fn serialize_remote_sequence<S>(val: &Vec<Device>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer,
{
  let mut seq = serializer.serialize_seq(Some(val.len()))?;
  for e in val {
    seq.serialize_element(e.as_remote().unwrap())?;
  }
  seq.end()
}

pub fn serialize_sensor_sequence<S>(val: &Vec<Device>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer,
{
  let mut seq = serializer.serialize_seq(Some(val.len()))?;
  for e in val {
    seq.serialize_element(e.as_sensor().unwrap())?;
  }
  seq.end()
}

pub fn deserialize_light_sequence<'de, D>(deserializer: D) -> Result<Vec<Device>, D::Error>
where
  D: Deserializer<'de>,
{
  let lights: Vec<Light> = Deserialize::deserialize(deserializer)?;
  Ok(lights.into_iter().map(Device::Light).collect())
}

pub fn deserialize_remote_sequence<'de, D>(deserializer: D) -> Result<Vec<Device>, D::Error>
where
  D: Deserializer<'de>,
{
  let lights: Vec<Remote> = Deserialize::deserialize(deserializer)?;
  Ok(lights.into_iter().map(Device::Remote).collect())
}

pub fn deserialize_sensor_sequence<'de, D>(deserializer: D) -> Result<Vec<Device>, D::Error>
where
  D: Deserializer<'de>,
{
  let lights: Vec<Sensor> = Deserialize::deserialize(deserializer)?;
  Ok(lights.into_iter().map(Device::Sensor).collect())
}
