use crate::convert::RestApiPayload;

use super::{executor::ExecutorLogic, request::RemoteAction, traits::DeviceCollection};

impl ExecutorLogic {
  pub(super) async fn remote_action(&mut self, action: RemoteAction) {
    let remote = self.home.find_remote(&action.target).expect("Implement error handling.");
    let cmd = remote.action(action.button).expect("Implement error handling");
    let mqtt = RestApiPayload { topic: Some(remote.controls()), ..Default::default() };
    self.execute_light(cmd, mqtt).await;
  }
}
