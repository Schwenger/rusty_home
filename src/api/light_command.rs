use crate::convert::RestApiPayload;

use super::{executor::ExecutorLogic, request::LightCommand, traits::EffectiveLightCollection};

impl ExecutorLogic {
  pub(super) async fn execute_light(&mut self, cmd: LightCommand, adds: RestApiPayload) {
    let target = adds.topic.clone().unwrap();
    println!("Executing {:?} for light {}", cmd, target.to_str());
    let light = self.home.find_effective_light_mut(&target).expect("Implement error handling.");
    let payloads = match cmd {
      LightCommand::TurnOn => light.turn_on(),
      LightCommand::TurnOff => light.turn_off(),
      LightCommand::Toggle => light.toggle(),
      LightCommand::DimUp => light.dim_up(),
      LightCommand::DimDown => light.dim_down(),
      LightCommand::StartDimUp => light.start_dim_up(),
      LightCommand::StartDimDown => light.start_dim_down(),
      LightCommand::StopDim => light.stop_dim(),
      LightCommand::ChangeState => light.change_state(adds),
    };
    self.send_mqtt_payloads(payloads).await;
  }
}
