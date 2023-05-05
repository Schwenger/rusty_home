use super::{executor::ExecutorLogic, request::General, traits::ReadWriteHome};

impl ExecutorLogic {
  pub(super) async fn execute_general(&mut self, cmd: General) {
    match cmd {
      General::Shutdown { home_path } => {
        self.client.lock().await.disconnect().await;
        self.home.lock().await.persist(&home_path).expect("Couldn't write home.")
      }
    }
  }
}
