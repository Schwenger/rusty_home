use super::{traits::ReadWriteHome, ExecutorLogic};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum General {
  Shutdown { home_path: String },
}

impl ExecutorLogic {
  pub(super) async fn execute_general(&mut self, cmd: General) {
    match cmd {
      General::Shutdown { home_path } => {
        self.client.lock().await.disconnect().await;
        self.home.persist(&home_path).expect("Couldn't write home.")
      }
    }
  }
}
