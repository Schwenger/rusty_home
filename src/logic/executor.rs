use futures::{
  channel::mpsc::{UnboundedReceiver, UnboundedSender},
  never::Never,
  StreamExt,
};

use crate::{
  api::{Command, JsonPayload, Query, Request},
  mqtt::ProtectedClient,
  Result,
};

pub struct Executor {
  requests: UnboundedReceiver<Request>,
  soldier: Soldier,
  scholar: Scholar,
}

impl Executor {
  pub fn new(
    requests: UnboundedReceiver<Request>,
    client: ProtectedClient,
    response: UnboundedSender<JsonPayload>,
  ) -> Self {
    Executor {
      requests,
      soldier: Soldier::new(client.clone()),
      scholar: Scholar::new(client, response),
    }
  }

  pub async fn run(self) -> Result<Never> {
    let Executor { requests, soldier, scholar } = self;
    requests
      .for_each(|req| async {
        match req {
          Request::Command(cmd) => soldier.exec(cmd).await,
          Request::Query(query) => scholar.respond(query).await,
        }
      })
      .await;
    unreachable!()
  }
}

struct Soldier {
  _client: ProtectedClient,
}

impl Soldier {
  fn new(client: ProtectedClient) -> Self {
    Soldier { _client: client }
  }

  async fn exec(&self, _cmd: Command) {
    todo!()
  }
}

struct Scholar {
  _client: ProtectedClient,
  _response: UnboundedSender<JsonPayload>,
}

impl Scholar {
  fn new(client: ProtectedClient, response: UnboundedSender<JsonPayload>) -> Self {
    Scholar { _client: client, _response: response }
  }

  async fn respond(&self, _to: Query) {
    todo!()
  }
}
