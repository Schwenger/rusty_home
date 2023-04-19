use futures::{
  channel::mpsc::{UnboundedReceiver, UnboundedSender},
  never::Never,
  StreamExt,
};

use crate::{
  api::{Command, JsonPayload, Query, Request, Queryable},
  mqtt::ProtectedClient,
  Result, home::Home,
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
    home: Home,
  ) -> Self {
    Executor {
      requests,
      soldier: Soldier::new(client),
      scholar: Scholar::new( response, home),
    }
  }

  pub async fn run(self) -> Result<Never> {
    let Executor { requests, soldier, scholar } = self;
    requests
      .fold(scholar, |mut scholar, req| async {
        match req {
          Request::Command(cmd) => soldier.exec(cmd).await,
          Request::Query(query) => scholar.respond(query).await,
        }
        scholar
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
