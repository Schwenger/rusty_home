use std::{sync::Arc, thread, time::Duration};

use crate::Result;
use futures::{lock::Mutex, never::Never, stream, StreamExt};
use paho_mqtt::{AsyncClient, AsyncReceiver, CreateOptionsBuilder, Message, QOS_1};

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct MqttClient {
  client: AsyncClient,
}

#[derive(Debug, Clone)]
pub struct MqttReceiver {
  stream: AsyncReceiver<Option<Message>>,
  client: ProtectedClient,
}

pub type ProtectedClient = Arc<Mutex<MqttClient>>;

pub async fn setup_client(host: &str, port: u16) -> Result<(ProtectedClient, MqttReceiver)> {
  let url = format!("mqtt://{host}:{port}");
  let mut client = CreateOptionsBuilder::new().client_id("Mac").server_uri(url).create_client()?;
  let stream = client.get_stream(None);
  client.connect(None).await?;
  let mqtt_client = MqttClient { client };
  let protected = Arc::new(Mutex::new(mqtt_client));
  let receiver = MqttReceiver { stream, client: protected.clone() };
  Ok((protected, receiver))
}

impl MqttReceiver {
  pub async fn run(self) -> Result<Never> {
    println!("Starting to receive.");
    self
      .stream
      .for_each_concurrent(None, |msg| async {
        println!("Receft.");
        match msg {
          None => self.client.lock().await.attempt_reconnect().await,
          Some(msg) => self.client.lock().await.handle_message(msg).await,
        }
      })
      .await;
    unreachable!()
  }
}

impl MqttClient {
  pub async fn publish(&self, topic: &str, payload: &str) {
    assert!(topic.ends_with("set") || topic.ends_with("get"));
    let msg = Message::new(topic, payload, QOS_1);
    if self.client.publish(msg).await.is_err() {
      eprintln!("Failed to publish message.")
    }
  }

  async fn handle_message(&self, msg: Message) {
    println!("Handling a message. {msg}");
    todo!()
  }

  async fn attempt_reconnect(&self) {
    println!("Detected disconnect.  Attempting to reconnect now.");
    while self.client.connect(None).await.is_err() {
      eprintln!("Failed to reconnect. Retrying....");
      thread::sleep(Duration::from_secs(3));
    }
    println!("Connection re-established.");
  }

  pub async fn query_states(&self, topics: &[&str]) {
    stream::iter(topics).for_each_concurrent(None, |topic| self.query_state(topic)).await;
  }

  async fn query_state(&self, _topic: &str) {
    todo!();
  }

  pub async fn subscribe_to_all(&self, topics: &[&str]) {
    stream::iter(topics).for_each_concurrent(None, |topic| self.subscribe_to(topic)).await;
  }

  async fn subscribe_to(&self, topic: &str) {
    let _ = self.client.subscribe(topic, QOS_1).await;
  }

  pub async fn disconnect(&self) {
    if let Err(err) = self.client.disconnect(None).await {
      eprintln!("Failed to disconnect with error: {err}.")
    }
  }
}
