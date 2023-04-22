use std::thread::sleep;
use std::time::Duration;

use crate::api::{Request, General};
use crate::home::Home;
use crate::web_server::WebServer;
use tokio::{join, select, pin};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::{
  api::Executor,
  config::GlobalConfig,
  mqtt::{self, MqttReceiver, ProtectedClient},
  Error,
};

#[allow(missing_debug_implementations)]
pub struct Controller {
  mqtt_receiver: MqttReceiver,
  executor: Executor,
  web_server: WebServer,
}

impl Controller {
  pub async fn new(config: GlobalConfig) -> Result<Self, Error> {
    let (client, mqtt_receiver) =  Self::setup_client(&config).await?;
    let home = Home::from(&config);
    let (q_send, q_recv) = unbounded_channel(); // Distribute send more liberally.
    let executor = Executor::new(q_recv, client, home);
    let web_server = WebServer::new(q_send.clone());
    Self::startup(q_send, &config.home.dir);
    Ok(Self { mqtt_receiver, executor, web_server })
  }

  pub fn startup(queue: UnboundedSender<Request>, home_path: &str) {
    let s = home_path.to_string();
    ctrlc::set_handler(move || {
      eprintln!("Detected shutdown. Initiating procedure.");
      let req = Request::General(General::Shutdown { home_path: s.clone() });
      queue.send(req).expect("Cannot send.");
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
    println!("Running controller.");
    let Controller { mqtt_receiver, executor, web_server, .. } = self;
    let mqtt_recv = mqtt_receiver.run();
    let requ_exec = executor.run();
    let web_serve = web_server.run();

    pin!(mqtt_recv, requ_exec, web_serve);

    select! {
      err = mqtt_recv => eprintln!("{:?}", err.unwrap_err()),
      err = requ_exec => eprintln!("{:?}", err.unwrap_err()),
      err = web_serve => eprintln!("{:?}", err.unwrap_err()),
    }; // Todo: Handle if one of them returned.  Re-use all but the crashed one.
    unreachable!()
  }
}
