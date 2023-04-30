use futures::{stream, StreamExt};

use super::{
  executor::ExecutorLogic, request::LightCommand, topic::Topic, traits::EffectiveLightCollection,
};

impl ExecutorLogic {
  pub(super) async fn execute_light(&mut self, cmd: LightCommand, target: Topic) {
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
      LightCommand::QueryUpdate => light.query_update(),
    };
    stream::iter(payloads)
      .for_each_concurrent(None, |(t, p)| async { self.client.lock().await.publish(t, p).await })
      .await;
  }
}
