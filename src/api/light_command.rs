use super::{
  executor::ExecutorLogic, request::LightCommand, topic::Topic, traits::EffectiveLightCollection,
};

impl ExecutorLogic {
  pub(super) async fn execute_light(&mut self, cmd: LightCommand, target: Topic) {
    println!("Executing for light {}", target.to_str());
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
    };
    self.send_mqtt_payloads(payloads).await;
  }
}
