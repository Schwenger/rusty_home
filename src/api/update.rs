use serde_json::Value;

use crate::devices::{light::LightState, sensor::SensorState, DeviceTrait};

use super::{
  topic::{DeviceKind, Topic},
  traits::{DeviceCollection, EffectiveLight},
  ExecutorLogic,
};

impl ExecutorLogic {
  pub(super) async fn update(&mut self, whom: Topic, with: Value) {
    match whom.device().unwrap() {
      DeviceKind::Light => {
        let light = self.home.find_physical_light_mut(&whom).expect("Error");
        light.update_state(LightState::from_payload(with, light.model()).expect("Error"))
      }
      DeviceKind::Sensor => {
        self.home.find_sensor_mut(&whom).expect("Error").update_state(SensorState::from(with))
      }
      DeviceKind::Outlet => todo!("Received non-pseudo-light-outlet update, dafuq?"),
      DeviceKind::Remote => todo!("Received remote update, dafuq?"),
    }
  }
}
