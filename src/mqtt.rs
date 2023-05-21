use std::{borrow::Borrow, sync::Arc, thread, time::Duration};

use crate::{
  api::{
    request::RemoteAction,
    request::{DeviceCommand, Request},
    topic::TopicKind,
    topic::{Topic, TopicMode},
    traits::Addressable,
  },
  convert::StateToMqtt,
  devices::{
    remote::{HueButton, IkeaDimmer, IkeaMulti, RemoteButton},
    Device,
  },
  Result,
};
use paho_mqtt::{AsyncClient, AsyncReceiver, CreateOptionsBuilder, Message, QOS_1};
use serde_json::Value as JsonValue;
use tokio::sync::{mpsc::UnboundedSender, Mutex};

#[allow(missing_debug_implementations)]
pub struct MqttClient {
  client: AsyncClient,
  queue: UnboundedSender<Request>,
  updates: UnboundedSender<(Topic, JsonValue)>,
}

#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct MqttReceiver {
  stream: AsyncReceiver<Option<Message>>,
  client: ProtectedClient,
}

pub type ProtectedClient = Arc<Mutex<MqttClient>>;

pub async fn setup_client(
  host: &str,
  port: u16,
  queue: UnboundedSender<Request>,
  updates: UnboundedSender<(Topic, JsonValue)>,
) -> Result<(ProtectedClient, MqttReceiver)> {
  let url = format!("mqtt://{host}:{port}");
  let mut client = CreateOptionsBuilder::new().client_id("Mac").server_uri(url).create_client()?;
  let stream = client.get_stream(None);
  client.connect(None).await?;
  let mqtt_client = MqttClient { client, queue, updates };
  let protected = Arc::new(Mutex::new(mqtt_client));
  let receiver = MqttReceiver { stream, client: protected.clone() };
  Ok((protected, receiver))
}

impl MqttReceiver {
  pub async fn run(self) -> Result<()> {
    println!("Starting to receive.");
    loop {
      let msg = self.stream.recv().await;
      match msg {
        Ok(None) | Err(_) => {} //self.client.lock().await.attempt_reconnect().await,
        Ok(Some(msg)) => self.client.lock().await.handle_message(msg).await,
      }
    }
  }
}

impl MqttClient {
  pub async fn publish(&self, topic: Topic, payload: StateToMqtt) {
    assert_ne!(topic.mode(), TopicMode::Blank);
    let payload = payload.to_json_str(false);
    println!("Sent: {} to {}", &payload, topic.to_str());
    let msg = Message::new(topic.to_str(), payload, QOS_1);
    if self.client.publish(msg).await.is_err() {
      eprintln!("Failed to publish message.");
    }
  }

  async fn handle_message(&self, msg: Message) {
    println!("Handling a message. \n{msg}");
    let target = msg.topic();
    let target = Topic::try_from(target.to_string()).unwrap();
    let payload: JsonValue = serde_json::from_str(msg.payload_str().borrow()).unwrap();
    if target.kind() == TopicKind::Bridge {
      println!("Received bridge event.  Ignored.")
    } else if let Some(action) = payload.get("action") {
      println!(
        "Received remote action: {:?}",
        serde_json::from_value::<IkeaDimmer>(action.clone())
      );
      if let Ok(dimmer) = serde_json::from_value::<IkeaDimmer>(action.clone()) {
        let ra = RemoteAction { button: RemoteButton::IkeaDimmer(dimmer), target };
        self.queue.send(Request::RemoteAction(ra)).unwrap();
      } else if let Ok(multi) = serde_json::from_value::<IkeaMulti>(action.clone()) {
        let ra = RemoteAction { button: RemoteButton::IkeaMulti(multi), target };
        self.queue.send(Request::RemoteAction(ra)).unwrap();
      } else if let Ok(multi) = serde_json::from_value::<HueButton>(action.clone()) {
        let ra = RemoteAction { button: RemoteButton::HueButton(multi), target };
        self.queue.send(Request::RemoteAction(ra)).unwrap();
      }
    } else if target.device().is_some() {
      println!("Received device state update");
      let parsed = serde_json::from_value(payload.clone()).unwrap();
      let req = Request::DeviceCommand(DeviceCommand::UpdateState(parsed), target.clone());
      self.queue.send(req).expect("Error handling.");
      self.updates.send((target, payload)).expect("Error handling.");
    }
  }

  #[allow(dead_code)]
  async fn attempt_reconnect(&self) {
    println!("Detected disconnect.  Attempting to reconnect now.");
    while self.client.connect(None).await.is_err() {
      eprintln!("Failed to reconnect. Retrying....");
      thread::sleep(Duration::from_secs(3));
    }
    println!("Connection re-established.");
  }

  pub async fn query_states(&self, devices: Vec<&Device>, queue: UnboundedSender<Request>) {
    devices
      .into_iter()
      .map(|d| d.topic(TopicMode::Blank))
      .map(|t| Request::DeviceCommand(DeviceCommand::QueryUpdate, t))
      .for_each(|r| queue.send(r).unwrap())
  }

  pub async fn subscribe_to_all<I: IntoIterator<Item = Topic>>(&self, topics: I) {
    for topic in topics {
      println!("Subscribing to {}", &topic.to_str());
      self.subscribe_to(topic).await;
    }
  }

  async fn subscribe_to(&self, topic: Topic) {
    let _ = self.client.subscribe(&topic.to_str(), QOS_1).await;
  }

  pub async fn disconnect(&self) {
    if let Err(err) = self.client.disconnect(None).await {
      eprintln!("Failed to disconnect with error: {err}.")
    }
  }
}
