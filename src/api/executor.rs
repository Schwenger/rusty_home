use futures::{
  channel::mpsc::{UnboundedReceiver, UnboundedSender},
  never::Never,
  StreamExt,
};

use crate::{
  api::{JsonPayload, Request},
  home::Home,
  mqtt::ProtectedClient,
  Result,
};

#[derive(Debug)]
pub struct Executor {
  inner: ExecutorLogic,
  requests: UnboundedReceiver<Request>,
}

impl Executor {
  pub fn new(
    requests: UnboundedReceiver<Request>,
    client: ProtectedClient,
    response: UnboundedSender<JsonPayload>,
    home: Home,
  ) -> Self {
    let inner = ExecutorLogic { client, home, response };
    Executor { requests, inner }
  }

  pub async fn run(self) -> Result<Never> {
    let Executor { requests, inner } = self;
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
  async fn process(&mut self, req: Request) {
    match req {
      Request::Query(query) => self.respond(query).await,
      Request::LightCommand(lc) => self.execute_light(lc).await,
      Request::HomeEdit(he) => self.edit_home(he).await,
    }
  }
}