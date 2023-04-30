use tokio::sync::mpsc::UnboundedReceiver;

use crate::{home::Home, mqtt::ProtectedClient, Result};

use super::request::Request;

#[allow(missing_debug_implementations)]
pub struct Executor {
  inner: ExecutorLogic,
  requests: UnboundedReceiver<Request>,
}

impl Executor {
  pub fn new(requests: UnboundedReceiver<Request>, client: ProtectedClient, home: Home) -> Self {
    let inner = ExecutorLogic { client, home };
    Executor { requests, inner }
  }

  pub async fn run(self) -> Result<()> {
    let Executor { mut requests, mut inner } = self;
    loop {
      let req = requests.recv().await;
      inner.process(req.unwrap()).await;
    }
  }
}

#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct ExecutorLogic {
  pub(super) client: ProtectedClient,
  pub(super) home: Home,
}

impl ExecutorLogic {
  async fn process(&mut self, req: Request) {
    match req {
      Request::Query(query, resp) => self.respond(query, resp).await,
      Request::LightCommand(cmd, target) => self.execute_light(cmd, target).await,
      Request::HomeEdit(he) => self.edit_home(he).await,
      Request::General(general) => self.execute_general(general).await,
      Request::RemoteAction(ra) => self.remote_action(ra).await,
      Request::Update(update) => self.update(update.target, update.state).await,
    }
  }
}
