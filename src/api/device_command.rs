use crate::devices::DeviceTrait;

use super::{
  executor::ExecutorLogic,
  request::DeviceCommand,
  topic::{Topic, TopicMode},
  traits::{DeviceCollection, Scenable},
};

impl ExecutorLogic {
  pub(super) async fn execute_device(&mut self, target: Topic, cmd: DeviceCommand) {
    match cmd {
      DeviceCommand::UpdateState(state) => {
        self.home.find_device_mut(&target).unwrap().update_state(state);
        self.home.trigger_update_scene(&self.queue);
      }
      DeviceCommand::QueryUpdate => {
        let device = self.home.find_device(&target).unwrap();
        let payload = device.query_update();
        self.send_mqtt_payloads(vec![(target.with_mode(TopicMode::Get), payload)]).await;
      }
    }
  }
}
