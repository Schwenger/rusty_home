use super::{home_edit::HomeEdit, light_command::LightCommand, query::Query};

#[derive(Debug, Clone)]
pub enum Request {
  Query(Query),
  LightCommand(LightCommand),
  HomeEdit(HomeEdit),
}
