use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::api::{light_command, DeviceKind, Topic};

use super::{Device, DeviceModel};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
#[serde(untagged)]
pub enum RemoteButton {
  IkeaMulti(IkeaMulti),
  IkeaDimmer(IkeaDimmer),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum IkeaMulti {
  #[serde(rename = "toggle")]
  Toggle,
  #[serde(rename = "arrow_left_click")]
  ArrLeftClick,
  #[serde(rename = "arrow_left_hold")]
  ArrLeftHold,
  #[serde(rename = "arrow_left_release")]
  ArrLeftRelease,
  #[serde(rename = "arrow_right_click")]
  ArrRightClick,
  #[serde(rename = "arrow_right_hold")]
  ArrRightHold,
  #[serde(rename = "arrow_right_release")]
  ArrRightRelease,
  #[serde(rename = "brightness_down_click")]
  BriDownClick,
  #[serde(rename = "brightness_down_hold")]
  BriDownHold,
  #[serde(rename = "brightness_down_release")]
  BriDownRelease,
  #[serde(rename = "brightness_up_click")]
  BriUpClick,
  #[serde(rename = "brightness_up_hold")]
  BriUpHold,
  #[serde(rename = "brightness_up_release")]
  BriUpRelease,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum IkeaDimmer {
  #[serde(rename = "on")]
  On,
  #[serde(rename = "off")]
  Off,
  #[serde(rename = "brightness_move_up")]
  BriMoveUp,
  #[serde(rename = "brightness_move_down")]
  BriMoveDown,
  #[serde(rename = "brightness_stop")]
  BriMoveStop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remote {
  name: String,
  model: DeviceModel,
  icon: String,
  controls: String,
  room: String,
  actions: HashMap<RemoteButton, light_command::Command>, // ToDo: Map to Api Command.
}

impl Device for Remote {
  fn kind(&self) -> DeviceKind {
    DeviceKind::Remote
  }

  fn model(&self) -> DeviceModel {
    self.model
  }

  fn name(&self) -> &str {
    &self.name
  }

  fn room(&self) -> &str {
    &self.room
  }
}

impl Remote {
  pub fn action(&self, button: RemoteButton) -> Option<light_command::Command> {
    self.actions.get(&button).copied()
  }

  pub fn controls(&self) -> Topic {
    Topic::try_from(self.controls.clone()).expect("Implement error handling.")
  }
}
