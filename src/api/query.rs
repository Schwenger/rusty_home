use super::{payload::JsonPayload, topic::Topic, traits::QueryableHome, ExecutorLogic};
use tokio::sync::oneshot::Sender;

#[derive(Debug, Clone)]
pub enum Query {
  Architecture,
  LightState(Topic),
}

impl ExecutorLogic {
  pub(super) async fn respond(&mut self, to: Query, over: Sender<JsonPayload>) {
    let res = match to {
      Query::Architecture => self.home.query_architecture(),
      Query::LightState(target) => self.home.query_device(target),
    };
    over.send(res).expect("Failed to send response.");
  }
}
