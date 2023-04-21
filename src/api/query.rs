use super::{ExecutorLogic, Queryable};

#[derive(Debug, Clone, Copy)]
pub enum Query {
  Architecture,
}

impl ExecutorLogic {
  pub(super) async fn respond(&mut self, to: Query) {
    let res = match to {
      Query::Architecture => self.home.query_architecture(),
    };
    self.response.start_send(res).expect("Failed to send response.");
  }
}
