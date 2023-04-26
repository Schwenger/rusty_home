use lazy_static::lazy_static;
use regex::{Captures, Regex};

use crate::{Error, Result};

use super::traits::TopicConvertible;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Topic {
  Home { mode: TopicMode },
  Bridge,
  Room { name: String, mode: TopicMode },
  Group { room: String, groups: Vec<String>, name: String, mode: TopicMode },
  Device { device: DeviceKind, room: String, groups: Vec<String>, name: String, mode: TopicMode },
}

impl Topic {
  const SEPARATOR: &str = "/";
  const BASE: &str = "zigbee2mqtt";

  pub fn kind(&self) -> TopicKind {
    match self {
      Topic::Home { .. } => TopicKind::Home,
      Topic::Bridge => TopicKind::Bridge,
      Topic::Room { .. } => TopicKind::Room,
      Topic::Group { .. } => TopicKind::Group,
      Topic::Device { .. } => TopicKind::Device,
    }
  }

  pub fn mode(&self) -> TopicMode {
    match self {
      Topic::Home { mode } => *mode,
      Topic::Bridge => TopicMode::Blank,
      Topic::Room { mode, .. } => *mode,
      Topic::Group { mode, .. } => *mode,
      Topic::Device { mode, .. } => *mode,
    }
  }

  pub fn with_mode(self, mode: TopicMode) -> Self {
    match self {
      Topic::Home { .. } => Topic::Home { mode },
      Topic::Bridge => Topic::Bridge,
      Topic::Room { mode: _, name } => Topic::Room { mode, name },
      Topic::Group { mode: _, room, groups, name } => Topic::Group { room, groups, name, mode },
      Topic::Device { mode: _, device, room, groups, name } => Topic::Device { device, room, groups, name, mode },
    }
  }

  fn components(&self) -> Vec<String> {
    let mut base = vec![String::from(Self::BASE)];
    base.push(self.kind().to_topic());
    match self {
      Topic::Home { mode: _mode } => {}
      Topic::Bridge => {
        base.push(String::from("event"));
      }
      Topic::Room { name, mode: _mode } => {
        base.push(name.clone());
      }
      Topic::Group { name, room, groups, mode: _mode } => {
        base.push(room.clone());
        base.extend(groups.clone());
        base.push(name.clone());
      }
      Topic::Device { name, room, groups, device, mode: _mode } => {
        base.push(device.to_topic());
        base.push(room.clone());
        base.extend(groups.clone());
        base.push(name.clone());
      }
    };
    match self.mode() {
      TopicMode::Get | TopicMode::Set => base.push(self.mode().to_topic()),
      TopicMode::Blank => {}
    };
    base
  }

  pub fn to_str(&self) -> String {
    self.components().join(Self::SEPARATOR)
  }

  const REGEX_HOME: &str = r"^zigbee2mqtt/Home(?:/(P?<mode>set|get))?$";
  const REGEX_BRIDGE: &str = r"^zigbee2mqtt/bridge/event$";
  const REGEX_ROOM: &str = r"^zigbee2mqtt/Room/(?P<name>(?:\w| )+)(?:/(?P<mode>set|get))?$";
  const REGEX_GROUP: &str = r"^zigbee2mqtt/Group/(?P<room>(?:\w| )+)(?:/(?:\w| )+)*?/(?P<name>(?:\w| )+)(?:/(?P<mode>set|get))?$";
  const REGEX_DEVICE: &str = r"^zigbee2mqtt/Device/(?P<kind>(?:\w| )+)/(?P<room>(?:\w| )+)(?:/(?:\w| )+)*?/(?P<name>(?:\w| )+)(?:/(?P<mode>set|get))?$";

  pub fn try_from(value: String) -> Result<Self> {
    lazy_static! {
      static ref RE_HOME: Regex = Regex::new(Topic::REGEX_HOME).unwrap();
      static ref RE_BRIDGE: Regex = Regex::new(Topic::REGEX_BRIDGE).unwrap();
      static ref RE_ROOM: Regex = Regex::new(Topic::REGEX_ROOM).unwrap();
      static ref RE_GROUP: Regex = Regex::new(Topic::REGEX_GROUP).unwrap();
      static ref RE_DEVICE: Regex = Regex::new(Topic::REGEX_DEVICE).unwrap();
    }
    if let Some(captures) = RE_HOME.captures(&value) {
      let mode = Self::read_mode(&captures);
      return Ok(Topic::Home { mode });
    }
    if RE_BRIDGE.is_match(&value) {
      return Ok(Topic::Bridge);
    }
    if let Some(captures) = RE_ROOM.captures(&value) {
      let name = captures.name("name").unwrap().as_str().to_string();
      let mode = Self::read_mode(&captures);
      return Ok(Topic::Room { name, mode });
    }
    if let Some(captures) = RE_GROUP.captures(&value) {
      let room = captures.name("room").unwrap().as_str().to_string();
      let name = captures.name("name").unwrap().as_str().to_string();
      let mode = Self::read_mode(&captures);
      let groups = Self::read_groups(&value, mode, 3).into_iter().map(String::from).collect();
      return Ok(Topic::Group { room, groups, name, mode });
    }
    if let Some(captures) = RE_DEVICE.captures(&value) {
      let room = captures.name("room").unwrap().as_str().to_string();
      let device = DeviceKind::from_str(captures.name("kind").unwrap().as_str())?;
      let mode = Self::read_mode(&captures);
      let name = captures.name("name").unwrap().as_str().to_string();
      let groups = Self::read_groups(&value, mode, 4).into_iter().map(String::from).collect();
      return Ok(Topic::Device { device, room, groups, name, mode });
    }
    Err(Error::InvalidTopic)
  }

  fn read_mode(cap: &Captures) -> TopicMode {
    cap
      .name("mode")
      .map(|m| m.as_str())
      .map(|m| TopicMode::from_str(m).unwrap())
      .unwrap_or(TopicMode::Blank)
  }

  fn read_groups(topic: &str, mode: TopicMode, skip: usize) -> Vec<&str> {
    let split: Vec<&str> = topic.split(Self::SEPARATOR).collect();
    let start = skip;
    let end = split.len();
    let end = end - 1; // For name.
    let end = end - if mode == TopicMode::Blank { 0 } else { 1 };
    split[start..end].to_vec()
  }
}

#[cfg(test)]
mod test {

  use crate::api::DeviceKind;

  use super::{Topic, TopicMode};

  #[test]
  fn test_out_home() {
    let topic = Topic::Home { mode: TopicMode::Get };
    assert_eq!(topic.to_str(), "zigbee2mqtt/Home/get");
    let re_topic = Topic::try_from(topic.to_str());
    assert!(re_topic.is_ok());
    assert_eq!(re_topic.unwrap(), topic);
  }

  #[test]
  fn test_out_bridge() {
    let topic = Topic::Bridge;
    assert_eq!(topic.to_str(), "zigbee2mqtt/bridge/event");
    let re_topic = Topic::try_from(topic.to_str());
    assert!(re_topic.is_ok());
    assert_eq!(re_topic.unwrap(), topic);
  }

  #[test]
  fn test_out_room() {
    let topic = Topic::Room { name: String::from("Living Room"), mode: TopicMode::Set };
    assert_eq!(topic.to_str(), "zigbee2mqtt/Room/Living Room/set");
    let re_topic = Topic::try_from(topic.to_str());
    assert!(re_topic.is_ok());
    assert_eq!(re_topic.unwrap(), topic);
  }

  #[test]
  fn test_out_group() {
    let topic = Topic::Group {
      room: String::from("Office"),
      groups: vec![String::from("Low"), String::from("Middle"), String::from("Last")],
      name: String::from("Main"),
      mode: TopicMode::Blank,
    };
    assert_eq!(topic.to_str(), "zigbee2mqtt/Group/Office/Low/Middle/Last/Main");
    let re_topic = Topic::try_from(topic.to_str());
    assert!(re_topic.is_ok());
    assert_eq!(re_topic.unwrap(), topic);
  }

  #[test]
  fn test_out_device() {
    let topic = Topic::Device {
      device: DeviceKind::Light,
      room: String::from("Office"),
      groups: vec![String::from("Grp")],
      name: String::from("Comfort Light"),
      mode: TopicMode::Get,
    };
    assert_eq!(topic.to_str(), "zigbee2mqtt/Device/Light/Office/Grp/Comfort Light/get");
    let re_topic = Topic::try_from(topic.to_str());
    assert!(re_topic.is_ok());
    assert_eq!(re_topic.unwrap(), topic);
  }
}
