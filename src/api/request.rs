#![allow(dead_code)]

use super::{query::Query, light_command::LightCommand, home_edit::HomeEdit};

#[derive(Debug, Clone)]
pub enum Request {
  Query(Query),
  LightCommand(LightCommand),
  HomeEdit(HomeEdit),
}

