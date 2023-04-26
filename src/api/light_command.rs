use crate::api::traits::Searchable;

use super::{ExecutorLogic, Topic};

#[derive(Debug, Clone)]
pub struct LightCommand {
  target: Topic,
  cmd: Command,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    let payload = match lcmd.cmd {
      Command::TurnOn => light.turn_on(),
      Command::TurnOff => light.turn_off(),
      Command::Toggle => light.toggle(),
      Command::DimUp => light.dim_up(),
      Command::DimDown => light.dim_down(),
      Command::StartDimUp => light.start_dim_up(),
      Command::StartDimDown => light.start_dim_down(),
      Command::StopDim => light.stop_dim(),
    };
    if let Some(payload) = payload {
      self.client.lock().await.publish(lcmd.target, payload).await;
    }
  }
}
