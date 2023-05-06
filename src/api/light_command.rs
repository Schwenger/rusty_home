use chrono::{Local, Timelike};

use crate::{
  common::Scalar,
  convert::{RestApiPayload, Val},
};

use super::{executor::ExecutorLogic, request::LightCommand, traits::EffectiveLightCollection};

impl ExecutorLogic {
  pub(super) async fn execute_light(&mut self, cmd: LightCommand, adds: RestApiPayload) {
    let target = adds.topic.clone().unwrap();
    let mut home = self.home.lock().await;
    let light = home.find_effective_light_mut(&target).expect("Implement error handling.");
    let payloads = match cmd {
      LightCommand::TurnOn => light.turn_on(Some(Self::dynamic_brightness())),
      LightCommand::TurnOff => light.turn_off(),
      LightCommand::Toggle => light.toggle(),
      LightCommand::DimUp => light.dim_up(),
      LightCommand::DimDown => light.dim_down(),
      LightCommand::StartDimUp => light.start_dim_up(),
      LightCommand::StartDimDown => light.start_dim_down(),
      LightCommand::StopDim => light.stop_dim(),
      LightCommand::ChangeState => light.change_state(adds),
    };
    drop(home);
    self.send_mqtt_payloads(payloads).await;
  }

  fn dynamic_brightness() -> Val {
    let (afternoon, hour) = Local::now().hour12();
    Self::_dynamic_brightness(afternoon, hour)
  }
  fn _dynamic_brightness(afternoon: bool, hour: u32) -> Val {
    // Factor ∈ [0.0, 1.0]
    let factor = if afternoon { (11 - hour) as f64 / 11.0 } else { hour as f64 / 11.0 };
    // Value ∈ [0.1, 0.9]
    let value = 0.9 * factor + 0.1;
    Val::from_scalar(Scalar::from(value))
  }
}

#[cfg(test)]
mod test {
  use super::ExecutorLogic;
  #[test]
  fn test_dynamic_brightness() {
    let res = ExecutorLogic::_dynamic_brightness(true, 0).to_rest().inner();
    assert!(0.9 <= res && res <= 1.0);
    let res = ExecutorLogic::_dynamic_brightness(false, 0).to_rest().inner();
    assert!(0.05 <= res && res <= 0.15);
    let res = ExecutorLogic::_dynamic_brightness(true, 11).to_rest().inner();
    assert!(0.05 <= res && res <= 0.15);
    let res = ExecutorLogic::_dynamic_brightness(false, 11).to_rest().inner();
    assert!(0.9 <= res && res <= 1.0);
  }
}
