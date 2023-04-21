use super::{ExecutorLogic, Topic, MqttPayload};

#[derive(Debug, Clone)]
pub enum LightCommand {
  TurnOn { target: Topic }
}

impl ExecutorLogic {
  pub(super) async fn execute_light(&mut self, cmd: LightCommand) {
    match cmd {
      LightCommand::TurnOn { target } => self.client.lock().await.publish(target, MqttPayload::new().with_state_change(true)).await
    }
  }
}
