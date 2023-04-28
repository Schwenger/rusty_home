use crate::devices::remote::RemoteButton;

use super::{topic::Topic, traits::RemoteCollection, ExecutorLogic};

#[derive(Debug, Clone)]
pub struct RemoteAction {
  pub button: RemoteButton,
  pub target: Topic,
}

impl ExecutorLogic {
  pub(super) async fn remote_action(&mut self, action: RemoteAction) {
    let remote = self.home.find_remote(&action.target).expect("Implement error handling.");
    let cmd = remote.action(action.button).expect("Implement error handling");
    self.execute_light(cmd, remote.controls()).await;
  }
}
