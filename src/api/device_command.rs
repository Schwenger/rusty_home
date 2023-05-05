use crate::devices::DeviceTrait;

use super::{
  executor::ExecutorLogic,
  request::DeviceCommand,
  topic::{Topic, TopicMode},
  traits::DeviceCollection,
};

impl ExecutorLogic {
  pub(super) async fn execute_device(&mut self, target: Topic, cmd: DeviceCommand) {
    match cmd {
      DeviceCommand::UpdateState(state) => {
        self.home.lock().await.find_device_mut(&target).unwrap().update_state(state);
      }
      DeviceCommand::QueryUpdate => {
        let home = self.home.lock().await;
        let device = home.find_device(&target).unwrap();
        let payload = device.query_update();
        drop(home);
        self.send_mqtt_payloads(vec![(target.with_mode(TopicMode::Get), payload)]).await;
      }
    }
  }
}
