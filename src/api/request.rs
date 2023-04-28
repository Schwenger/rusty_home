use serde_json::Value;
use tokio::sync::oneshot::Sender;

use super::{
  home_edit::HomeEdit, light_command::LightCommand, payload::JsonPayload, query::Query,
  remote::RemoteAction, topic::Topic, General,
};

#[derive(Debug)]
pub enum Request {
  Query(Query, Sender<JsonPayload>),
  Update(Value, Topic),
  LightCommand(LightCommand, Topic),
  RemoteAction(RemoteAction),
  HomeEdit(HomeEdit),
  General(General),
}
