use futures::{StreamExt, stream};
use serde::{Deserialize, Serialize};

use crate::api::traits::LightCollection;

use super::{ExecutorLogic, Topic};

#[derive(Debug, Clone)]
pub struct LightCommand {
  pub target: Topic,
  pub cmd: Command,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
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
  pub(super) async fn execute_light(&mut self, lcmd: LightCommand) {
    let light = self.home.find_light_mut(&lcmd.target).expect("Implement error handling.");
    let payloads = match lcmd.cmd {
      Command::TurnOn => light.turn_on(),
      Command::TurnOff => light.turn_off(),
      Command::Toggle => light.toggle(),
      Command::DimUp => light.dim_up(),
      Command::DimDown => light.dim_down(),
      Command::StartDimUp => light.start_dim_up(),
      Command::StartDimDown => light.start_dim_down(),
      Command::StopDim => light.stop_dim(),
    };
    stream::iter(payloads).for_each_concurrent(None, |(t, p)| async {
      self.client.lock().await.publish(t, p).await
    }).await;
  }
}
