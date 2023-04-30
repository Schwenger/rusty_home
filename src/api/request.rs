use serde::{Deserialize, Serialize};
use tokio::sync::oneshot::Sender;

use crate::{devices::remote::RemoteButton, mqtt::MqttState};

use super::{payload::JsonPayload, topic::Topic};

#[derive(Debug)]
pub enum Request {
  Query(Query, Sender<JsonPayload>),
  Update(Update),
  LightCommand(LightCommand, Topic),
  RemoteAction(RemoteAction),
  HomeEdit(HomeEdit),
  General(General),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
  Architecture,
  DeviceState(Topic),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Update {
  pub state: MqttState,
  pub target: Topic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightCommand {
  TurnOn,
  TurnOff,
  Toggle,
  DimUp,
  DimDown,
  StartDimUp,
  StartDimDown,
  StopDim,
  QueryUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAction {
  pub button: RemoteButton,
  pub target: Topic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HomeEdit {
  AddRoom { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum General {
  Shutdown { home_path: String },
}
