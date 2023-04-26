mod light;
mod remote;
mod sensor;

pub use light::{Light, LightGroup};
pub use remote::Remote;
pub use sensor::Sensor;
use serde::{Deserialize, Serialize};

use crate::api::DeviceKind;

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
