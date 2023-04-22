mod light;
mod sensor;
mod remote;

pub use light::{Light, LightGroup};
pub use sensor::Sensor;
pub use remote::Remote;
use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceModel {
  TuyaHumidity,
  IkeaOutlet,
  IkeaDimmable,
  HueColor,
  IkeaMulti,
  IkeaDimmer,
}
