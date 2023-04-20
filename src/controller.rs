use std::thread::sleep;
use std::time::Duration;

use crate::api::{DeviceKind, Topic, TopicMode};
use crate::home::Home;
use futures::{channel::mpsc::unbounded, executor::block_on, join, pin_mut, select, FutureExt};

use crate::{
  config::GlobalConfig,
  frame,
  logic::Executor,
  mqtt::{self, MqttReceiver, ProtectedClient},
  Error,
};

pub struct Controller {
  client: ProtectedClient,
  mqtt_receiver: MqttReceiver,
  executor: Executor,
}

impl Controller {
  pub fn new(config: GlobalConfig) -> Result<Self, Error> {
    let (client, mqtt_receiver) = block_on(async { Self::setup_client(&config).await })?;
    let home = Home::from(&config);
    let (web_send, _web_recv) = unbounded(); // Create in WebApi
    let (_q_send, q_recv) = unbounded(); // Distribute send more liberally.
    let executor = Executor::new(q_recv, client.clone(), web_send, home);
    frame::startup(client.clone());
    Ok(Self { client, mqtt_receiver, executor })
  }

  pub fn test_mode(&self) {
    block_on(async {
      let topic = Topic::Device {
        name: "Orb".to_string(),
        room: "Living Room".to_string(),
        groups: vec![],
        device: DeviceKind::Light,
      };
      let payload = "{\"state\": \"OFF\"}";
      println!("Publishing");
      self.client.lock().await.publish(&topic.to_str(TopicMode::Set), payload).await;
      println!("Disconnecting");
      self.client.lock().await.disconnect().await;
      println!("Nap Time.");
      sleep(Duration::from_secs(20));
    });
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
    let Controller { client: _client, mqtt_receiver, executor } = self;
    let mqtt_recv = mqtt_receiver.run().fuse();
    let requ_exec = executor.run().fuse();

    pin_mut!(mqtt_recv, requ_exec);

    select! {
      err = mqtt_recv => eprintln!("{:?}", err.unwrap_err()),
      err = requ_exec => eprintln!("{:?}", err.unwrap_err()),
    }; // Todo: Handle if one of them returned.  Re-use all but the crashed one.
    unreachable!()
  }
}
