use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};

use crate::api::traits::LightCollection;

use super::{topic::Topic, ExecutorLogic};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightCommand {
  TurnOn,
  TurnOff,
  Toggle,
  DimUp,
  DimDown,
  StartDimUp,
  StartDimDown,
  StopDim,
}

impl ExecutorLogic {
  pub(super) async fn execute_light(&mut self, cmd: LightCommand, target: Topic) {
    let light = self.home.find_light_mut(&target).expect("Implement error handling.");
    let payloads = match cmd {
      LightCommand::TurnOn => light.turn_on(),
      LightCommand::TurnOff => light.turn_off(),
      LightCommand::Toggle => light.toggle(),
      LightCommand::DimUp => light.dim_up(),
      LightCommand::DimDown => light.dim_down(),
      LightCommand::StartDimUp => light.start_dim_up(),
      LightCommand::StartDimDown => light.start_dim_down(),
      LightCommand::StopDim => light.stop_dim(),
    };
    stream::iter(payloads)
      .for_each_concurrent(None, |(t, p)| async { self.client.lock().await.publish(t, p).await })
      .await;
  }
}
