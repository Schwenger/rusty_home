use std::{
  sync::Arc,
  thread::{self, JoinHandle},
  time::Duration,
};

use crate::Error;
use futures::{executor::block_on, never::Never, stream, StreamExt};
use paho_mqtt::{AsyncClient, AsyncReceiver, CreateOptionsBuilder, Message, QOS_1};

pub struct Client {
  client: AsyncClient,
}

impl Client {
  pub async fn new(host: &str, port: u16) -> Result<Self, Error> {
    let url = format!("mqtt://{host}:{port}");
    let client = CreateOptionsBuilder::new().client_id("Mac").server_uri(url).create_client()?;
    Ok(Self { client })
  }

  pub async fn publish(&self, topic: &str, payload: &str) {
    assert!(topic.ends_with("set") || topic.ends_with("get"));
    let msg = Message::new(topic, payload, QOS_1);
    if self.client.publish(msg).await.is_err() {
      eprintln!("Failed to publish message.")
    }
  }

  pub async fn run(mut self) -> Result<(Arc<Self>, JoinHandle<Error>), Error> {
    println!("Running.");
    let stream = self.client.get_stream(None);
    println!("Connecting.");
    self.client.connect(None).await?;
    println!("Got stream.");
    let arc = Arc::new(self);
    println!("Created arc.");
    let clone = arc.clone();
    let handle = thread::spawn(move || clone._run(stream).unwrap_err());
    Ok((arc, handle))
  }

  pub fn _run(&self, stream: AsyncReceiver<Option<Message>>) -> Result<Never, Error> {
    block_on(async {
      println!("Starting to receive.");
      stream
        .for_each_concurrent(None, |msg| async {
          println!("Receft.");
          match msg {
            None => self.attempt_reconnect().await,
            Some(msg) => self.handle_message(msg).await,
          }
        })
        .await;
      unreachable!()
    })
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
