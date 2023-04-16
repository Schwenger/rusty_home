use std::{
  collections::HashMap,
  sync::Arc,
  thread::{self, JoinHandle},
  time::Duration,
};

use futures::{executor::block_on, join};

use crate::{config::GlobalConfig, frame, mqtt::Client, Error};

type Handle = JoinHandle<Error>;

pub struct Controller {
  _components: HashMap<MainComponent, Handle>,
  client: Arc<Client>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MainComponent {
  Client,
  // WebApi,
  // Responder,
  // Commander,
  // Refresher,
}

impl Controller {
  pub fn new(config: GlobalConfig) -> Result<Self, Error> {
    frame::startup();
    let (client, handle) = block_on(async { Self::setup_client(&config).await })?;
    let components = hashmap! {
      MainComponent::Client => handle
    };
    Ok(Self { client, _components: components })
  }

  pub fn test_mode(&self) {
    block_on(async {
      let topic = "zigbee2mqtt/Device/Light/Living Room/Orb/set";
      let payload = "{\"state\": \"OFF\"}";
      println!("Publishing");
      self.client.publish(topic, payload).await;
      println!("Disconnecting");
      self.client.disconnect().await;
      println!("Nap Time.");
      thread::sleep(Duration::from_secs(20));
    });
  }

  pub fn shutdown(&self) {
    block_on(async { self.client.disconnect().await });
    frame::shutdown();
  }

  async fn setup_client(config: &GlobalConfig) -> Result<(Arc<Client>, Handle), Error> {
    let client = Client::new(&config.mosquitto.ip, config.mosquitto.port).await?;
    let empty = vec![];
    let sub = client.subscribe_to_all(&empty);
    let que = client.query_states(&empty);
    join!(sub, que);
    let (client, handle) = client.run().await?;
    Ok((client, handle))
  }
}
