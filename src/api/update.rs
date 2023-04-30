use crate::{devices::DeviceTrait, mqtt::MqttState};

use super::{
  executor::ExecutorLogic,
  topic::{DeviceKind, Topic},
  traits::DeviceCollection,
};

impl ExecutorLogic {
  pub(super) async fn update(&mut self, whom: Topic, with: MqttState) {
    match whom.device().unwrap() {
      DeviceKind::Light => {
        let light = self.home.find_physical_light_mut(&whom).expect("Error");
        light.update_state(with);
      }
      DeviceKind::Sensor => self.home.find_sensor_mut(&whom).expect("Error").update_state(with),
      DeviceKind::Outlet => todo!("Received non-pseudo-light-outlet update, dafuq?"),
      DeviceKind::Remote => todo!("Received remote update, dafuq?"),
    }
  }
}
