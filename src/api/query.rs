use super::{payload::JsonPayload, traits::QueryableHome, ExecutorLogic};
use tokio::sync::oneshot::Sender;

#[derive(Debug, Clone, Copy)]
pub enum Query {
  Architecture,
}

impl ExecutorLogic {
  pub(super) async fn respond(&mut self, to: Query, over: Sender<JsonPayload>) {
    let res = match to {
      Query::Architecture => self.home.query_architecture(),
    };
    over.send(res).expect("Failed to send response.");
  }
}
