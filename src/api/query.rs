use super::{executor::ExecutorLogic, payload::JsonPayload, request::Query, traits::QueryableHome};
use tokio::sync::oneshot::Sender;

impl ExecutorLogic {
  pub(super) async fn respond(&mut self, to: Query, over: Sender<JsonPayload>) {
    let res = match to {
      Query::Architecture => self.home.lock().await.query_architecture(),
      Query::DeviceState(target) => {
        JsonPayload::from_string(self.home.lock().await.query_device(target).to_json_str(true))
      }
      Query::DeviceHistory(target) => {
        let history = self
          .home
          .lock()
          .await
          .query_history(target)
          .into_iter()
          .map(|state| state.to_json_value(true))
          .collect::<Vec<_>>();
        JsonPayload::from(&history)
      }
    };
    over.send(res).expect("Failed to send response.");
  }
}
