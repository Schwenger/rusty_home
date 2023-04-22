use tokio::sync::oneshot::Sender;

use super::{home_edit::HomeEdit, light_command::LightCommand, query::Query, General, JsonPayload};

#[derive(Debug)]
pub enum Request {
  Query(Query, Sender<JsonPayload>),
  LightCommand(LightCommand),
  HomeEdit(HomeEdit),
  General(General),
}
