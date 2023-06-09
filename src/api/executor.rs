use std::rc::Rc;

use futures::{stream, StreamExt};
use tokio::sync::{
  mpsc::{UnboundedReceiver, UnboundedSender},
  Mutex,
};

use crate::{
  convert::StateToMqtt, home::Home, mqtt::ProtectedClient, scenes::manager::SceneEvent, Result,
};

use super::{request::Request, topic::Topic};

#[allow(missing_debug_implementations)]
pub struct Executor {
  inner: ExecutorLogic,
  requests: UnboundedReceiver<Request>,
}

impl Executor {
  pub fn new(
    requests: UnboundedReceiver<Request>,
    scene_events: UnboundedSender<SceneEvent>,
    client: ProtectedClient,
    home: Rc<Mutex<Home>>,
  ) -> Self {
    let inner = ExecutorLogic { client, home, scene_events };
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
pub struct ExecutorLogic {
  pub(super) client: ProtectedClient,
  pub(super) home: Rc<Mutex<Home>>,
  pub(super) scene_events: UnboundedSender<SceneEvent>,
}

impl ExecutorLogic {
  pub(super) async fn process(&mut self, req: Request) {
    match req {
      Request::Query(query, resp) => self.respond(query, resp).await,
      Request::LightCommand(cmd, additional) => self.execute_light(cmd, additional).await,
      Request::HomeEdit(he) => self.edit_home(he).await,
      Request::General(general) => self.execute_general(general).await,
      Request::RemoteAction(ra) => self.remote_action(ra).await,
      Request::DeviceCommand(cmd, target) => self.execute_device(target, cmd).await,
      Request::SceneCommand(cmd) => self.execute_scene(cmd).await,
    }
  }

  pub(super) async fn send_mqtt_payloads(&mut self, payloads: Vec<(Topic, StateToMqtt)>) {
    stream::iter(payloads)
      .for_each_concurrent(None, |(t, p)| async { self.client.lock().await.publish(t, p).await })
      .await;
  }
}
