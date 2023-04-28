pub mod light;
pub mod remote;
pub mod sensor;

pub use light::{Light, LightGroup};
pub use remote::Remote;
pub use sensor::Sensor;
use serde::{Deserialize, Serialize};

use crate::api::{
  topic::{DeviceKind, Topic, TopicMode},
  traits::Addressable,
};

pub trait Device: Addressable {
  fn kind(&self) -> DeviceKind;
  fn model(&self) -> DeviceModel;
  fn name(&self) -> &str;
  fn room(&self) -> &str;
}

impl<T: Device> Addressable for T {
  fn topic(&self, mode: TopicMode) -> Topic {
    Topic::Device {
      device: self.kind(),
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
  fn kind(&self) -> DeviceKind {
    match self {
      DeviceModel::TuyaHumidity => DeviceKind::Sensor,
      DeviceModel::IkeaOutlet => DeviceKind::Outlet,
      DeviceModel::IkeaDimmable => DeviceKind::Remote,
      DeviceModel::HueColor => DeviceKind::Light,
      DeviceModel::IkeaMultiButton => DeviceKind::Remote,
      DeviceModel::IkeaDimmer => DeviceKind::Light,
    }
  }
}
