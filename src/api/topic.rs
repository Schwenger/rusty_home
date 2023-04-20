#![allow(dead_code)]
use crate::{api::TopicConvertible, Error, Result};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum DeviceKind {
  Light,
  Sensor,
  Outlet,
  Remote,
}

impl TopicConvertible for DeviceKind {
  fn to_topic(&self) -> String {
    format!("{:?}", self)
  }

  fn from_str(s: &str) -> Result<Self> {
    match s {
      "Light" => Ok(DeviceKind::Light),
      "Sensor" => Ok(DeviceKind::Sensor),
      "Outlet" => Ok(DeviceKind::Outlet),
      "Remote" => Ok(DeviceKind::Remote),
      _ => Err(Error::ImpossibleStrConversion),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TopicMode {
  Set,
  Get,
  Blank,
}

impl TopicConvertible for TopicMode {
  fn to_topic(&self) -> String {
    if matches!(self, TopicMode::Blank) {
      return String::new();
    }
    format!("{:?}", self).to_lowercase()
  }

  fn from_str(s: &str) -> Result<Self> {
    match s {
      "set" => Ok(TopicMode::Set),
      "get" => Ok(TopicMode::Get),
      "" => Ok(TopicMode::Blank),
      _ => Err(Error::ImpossibleStrConversion),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TopicKind {
  Home,
  Bridge,
  Room,
  Group,
  Device,
}

impl TopicConvertible for TopicKind {
  fn to_topic(&self) -> String {
    if matches!(self, TopicKind::Bridge) {
      return format!("{:?}", self).to_lowercase();
    }
    format!("{:?}", self)
  }

  fn from_str(s: &str) -> Result<Self> {
    match s {
      "Home" => Ok(TopicKind::Home),
      "bridge" => Ok(TopicKind::Bridge),
      "Room" => Ok(TopicKind::Room),
      "Group" => Ok(TopicKind::Group),
      "Device" => Ok(TopicKind::Device),
      _ => Err(Error::ImpossibleStrConversion),
    }
  }
}

pub enum Topic {
  Home,
  Bridge,
  Room { name: String },
  Group { room: String, groups: Vec<String>, name: String },
  Device { device: DeviceKind, room: String, groups: Vec<String>, name: String },
}

impl Topic {
  const SEPARATOR: &str = "/";
  const BASE: &str = "zigbee2mqtt";

  fn kind(&self) -> TopicKind {
    match self {
      Topic::Home => TopicKind::Home,
      Topic::Bridge => TopicKind::Bridge,
      Topic::Room { .. } => TopicKind::Room,
      Topic::Group { .. } => TopicKind::Group,
      Topic::Device { .. } => TopicKind::Device,
    }
  }

  fn components(&self, mode: TopicMode) -> Vec<String> {
    let mut base = vec![String::from(Self::BASE)];
    base.push(self.kind().to_topic());
    match self {
      Topic::Home => {}
      Topic::Bridge => {
        base.push(String::from("event"));
      }
      Topic::Room { name } => {
        base.push(name.clone());
      }
      Topic::Group { name, room, groups } => {
        base.push(room.clone());
        base.extend(groups.clone());
        base.push(name.clone());
      }
      Topic::Device { name, room, groups, device } => {
        base.push(device.to_topic());
        base.push(room.clone());
        base.extend(groups.clone());
        base.push(name.clone());
      }
    };
    match mode {
      TopicMode::Get | TopicMode::Set => base.push(mode.to_topic()),
      TopicMode::Blank => {}
    };
    base
  }

  pub fn to_str(&self, mode: TopicMode) -> String {
    self.components(mode).join(Self::SEPARATOR)
  }

  fn room_groups_name<'a, I>(mut iter: I) -> Result<(String, Vec<String>, String)>
  where
    I: Iterator<Item = &'a str>,
  {
    let room = iter.next().ok_or(Error::InvalidTopic).map(String::from)?;
    let vec: Vec<String> = iter.map(String::from).collect();
    let (name, groups) = vec.split_last().ok_or(Error::InvalidTopic)?;
    Ok((room, groups.to_vec(), name.to_owned()))
  }

  pub fn try_from(value: String) -> Result<Self> {
    let mut split = value.split(Topic::SEPARATOR);
    if split.next().ok_or(Error::InvalidTopic)? != Self::BASE {
      return Err(Error::InvalidTopic);
    }
    let kind = split.next().map(TopicKind::from_str).ok_or(Error::InvalidTopic)??;
    match kind {
      TopicKind::Home => Ok(Topic::Home),
      TopicKind::Bridge if split.next().ok_or(Error::InvalidTopic)? == "event" => Ok(Topic::Bridge),
      TopicKind::Bridge => Err(Error::InvalidTopic),
      TopicKind::Room => {
        let name = split.next().map(String::from).ok_or(Error::InvalidTopic)?;
        Ok(Topic::Room { name })
      }
      TopicKind::Group => {
        let (room, groups, name) = Self::room_groups_name(split)?;
        Ok(Topic::Group { room, groups, name })
      }
      TopicKind::Device => {
        let device: DeviceKind =
          split.next().map(DeviceKind::from_str).ok_or(Error::InvalidTopic)??;
        let (room, groups, name) = Self::room_groups_name(split)?;
        Ok(Topic::Device { device, room, groups, name })
      }
    }
  }
}
