use std::thread::{sleep, self};
use std::time::Duration;

use crate::api::{Request, General};
use crate::home::Home;
use crate::web_server::WebServer;
use futures::channel::mpsc::UnboundedSender;
use futures::{channel::mpsc::unbounded, executor::block_on, join, pin_mut, select, FutureExt};

use crate::{
  api::Executor,
  config::GlobalConfig,
  mqtt::{self, MqttReceiver, ProtectedClient},
  Error,
};

#[derive(Debug)]
pub struct Controller {
  mqtt_receiver: MqttReceiver,
  executor: Executor,
  web_server: WebServer,
}

impl Controller {
  pub fn new(config: GlobalConfig) -> Result<Self, Error> {
    let (client, mqtt_receiver) = block_on(async { Self::setup_client(&config).await })?;
    let home = Home::from(&config);
    let (web_send, _web_recv) = unbounded(); // Create in WebApi
    let (q_send, q_recv) = unbounded(); // Distribute send more liberally.
    let executor = Executor::new(q_recv, client, web_send, home);
    let web_server = WebServer::new();
    Self::startup(q_send, &config.home.dir);
    Ok(Self { mqtt_receiver, executor, web_server })
  }

  pub fn startup(mut queue: UnboundedSender<Request>, home_path: &str) {
    let s = home_path.to_string();
    ctrlc::set_handler(move || {
      eprintln!("Detected shutdown. Initiating procedure.");
      let req = Request::General(General::Shutdown { home_path: s.clone() });
      queue.start_send(req).unwrap();
      sleep(Duration::from_secs(1));
      std::process::exit(0);
    })
    .expect("Failed to set shutdown handler.");
  }

  async fn setup_client(config: &GlobalConfig) -> Result<(ProtectedClient, MqttReceiver), Error> {
    let (client, receiver) =
      mqtt::setup_client(&config.mosquitto.ip, config.mosquitto.port).await?;
    let empty = vec![];
    {
      let client = client.lock().await;
      let sub = client.subscribe_to_all(&empty);
      let que = client.query_states(&empty);
      join!(sub, que);
    }
    Ok((client, receiver))
  }

  pub async fn run(self) -> ! {
    let Controller { mqtt_receiver, executor, web_server, .. } = self;
    let mqtt_recv = mqtt_receiver.run().fuse();
    let requ_exec = executor.run().fuse();
    let _web_hndl = thread::spawn(|| web_server.run());

    pin_mut!(mqtt_recv, requ_exec);

    select! {
      err = mqtt_recv => eprintln!("{:?}", err.unwrap_err()),
      err = requ_exec => eprintln!("{:?}", err.unwrap_err()),
    }; // Todo: Handle if one of them returned.  Re-use all but the crashed one.
    unreachable!()
  }
}
