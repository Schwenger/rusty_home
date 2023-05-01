use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot::Sender;

use crate::{convert::RestApiPayload, convert::StateFromMqtt, devices::remote::RemoteButton};

use super::{payload::JsonPayload, topic::Topic};

pub type Additional = HashMap<String, String>;

#[derive(Debug)]
pub enum Request {
  Query(Query, Sender<JsonPayload>),
  DeviceCommand(DeviceCommand, Topic),
  LightCommand(LightCommand, RestApiPayload),
  RemoteAction(RemoteAction),
  HomeEdit(HomeEdit),
  General(General),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
  Architecture,
  DeviceState(Topic),
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
  ChangeState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceCommand {
  UpdateState(StateFromMqtt),
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
