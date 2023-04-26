use tokio::sync::oneshot::Sender;

use super::{
  home_edit::HomeEdit, light_command::LightCommand, payload::JsonPayload, query::Query,
  remote::RemoteAction, General,
};

#[derive(Debug)]
pub enum Request {
  Query(Query, Sender<JsonPayload>),
  LightCommand(LightCommand),
  RemoteAction(RemoteAction),
  HomeEdit(HomeEdit),
  General(General),
}
