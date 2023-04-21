use futures::{
  channel::mpsc::{UnboundedReceiver, UnboundedSender},
  never::Never,
  StreamExt,
};

use crate::{
  api::{JsonPayload, Query, Queryable, Request},
  home::Home,
  mqtt::ProtectedClient,
  Result,
};

#[derive(Debug)]
pub struct Executor {
  inner: ExecutorLogic,
  requests: UnboundedReceiver<Request>,
  // soldier: Soldier,
  // scholar: Scholar,
}

impl Executor {
  pub fn new(
    requests: UnboundedReceiver<Request>,
    client: ProtectedClient,
    response: UnboundedSender<JsonPayload>,
    home: Home,
  ) -> Self {
    let inner = ExecutorLogic { client, home, response };
    Executor { requests, inner }//, soldier: Soldier::new(client), scholar: Scholar::new(response, home) }
  }

  pub async fn run(self) -> Result<Never> {
    let Executor { requests, inner /*soldier, scholar*/ } = self;
    requests
      .fold(inner, |mut inner, req| async {
        inner.process(req).await;
        inner
      })
      .await;
    unreachable!()
  }
}

#[derive(Debug, Clone)]
pub struct ExecutorLogic {
  pub(super) client: ProtectedClient,
  pub(super) home: Home,
  pub(super) response: UnboundedSender<JsonPayload>,
}

impl ExecutorLogic {
  fn new(client: ProtectedClient, response: UnboundedSender<JsonPayload>, home: Home) -> Self {
    ExecutorLogic { client, response, home }
  }

  async fn process(&mut self, req: Request) {
    match req {
      Request::Query(query) => self.respond(query).await,
      Request::LightCommand(lc) => self.execute_light(lc).await,
      Request::HomeEdit(he) => self.edit_home(he).await,
    }
  }
}

struct Scholar {
  home: Home,
  response: UnboundedSender<JsonPayload>,
}

impl Scholar {
  fn new(response: UnboundedSender<JsonPayload>, home: Home) -> Self {
    Scholar { home, response }
  }

  async fn respond(&mut self, to: Query) {
    let res = match to {
      Query::Architecture => self.home.query_architecture(),
    };
    self.response.start_send(res).expect("Failed to send response.");
  }
}
