use super::{executor::ExecutorLogic, payload::JsonPayload, request::Query, traits::QueryableHome};
use tokio::sync::oneshot::Sender;

impl ExecutorLogic {
  pub(super) async fn respond(&mut self, to: Query, over: Sender<JsonPayload>) {
    let res = match to {
      Query::Architecture => self.home.query_architecture(),
      Query::DeviceState(target) => {
        JsonPayload::from_string(self.home.query_device(target).to_json_str(true))
      }
    };
    over.send(res).expect("Failed to send response.");
  }
}
