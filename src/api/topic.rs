#![allow(dead_code)]

use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum DeviceKind {
  Light,
  Sensor,
  Outlet,
  Remote,
}

impl Display for DeviceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TopicMode {
  Set,
  Get,
  Blank,
}

pub enum Topic {
  Home,
  Bridge,
  Room { name: String },
  Group { name: String, room: String, groups: Vec<String> },
  Device { name: String, room: String, groups: Vec<String>, kind: DeviceKind },
}

impl Topic {

  const SEPARATOR: &str = "/";

  fn components(&self, mode: TopicMode) -> Vec<String> {
    let mut base = vec![String::from("zigbee2mqtt")];
    match self {
      Topic::Home => base.push(String::from("Home")),
      Topic::Bridge => {
        base.push(String::from("bridge"));
        base.push(String::from("event"));
      },
      Topic::Room { name } => {
        base.push(String::from("Room"));
        base.push(name.clone());
      }
      Topic::Group { name, room, groups } => {
        base.push(String::from("Group"));
        base.push(room.clone());
        base.extend(groups.clone());
        base.push(name.clone());
      }
      Topic::Device { name, room, groups, kind } => {
        let mut res = vec![];
        res.push(String::from("Device"));
        res.push(kind.to_string());
        res.push(room.clone());
        res.extend(groups.clone());
        res.push(name.clone());
      }
    };
    match mode {
      TopicMode::Get => base.push(String::from("get")),
      TopicMode::Set => base.push(String::from("set")),
      TopicMode::Blank => {}
    };
    base
  }

  pub fn string(&self, mode: TopicMode) -> String {
    self.components(mode).join(Self::SEPARATOR)
  }
}
