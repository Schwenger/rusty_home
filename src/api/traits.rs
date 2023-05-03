use core::fmt::Debug;

use tokio::sync::mpsc::UnboundedSender;

use crate::api::topic::Topic;
use crate::convert::RestApiPayload;
use crate::convert::StateToMqtt;
use crate::devices::{Device, Light, Remote, Sensor};
use crate::Result;

use super::payload::JsonPayload;
use super::request::Request;
use super::topic::TopicMode;

pub trait QueryableHome {
  fn query_architecture(&self) -> JsonPayload;
  fn query_device(&self, topic: Topic) -> StateToMqtt;
}

pub trait DeviceCollection: Debug {
  fn flatten_devices(&self) -> Vec<&Device>;
  fn flatten_devices_mut(&mut self) -> Vec<&mut Device>;

  fn find_device(&self, topic: &Topic) -> Option<&Device> {
    self.flatten_devices().into_iter().find(|d| &d.topic(topic.mode()) == topic)
  }
  fn find_device_mut(&mut self, topic: &Topic) -> Option<&mut Device> {
    self.flatten_devices_mut().into_iter().find(|d| &d.topic(topic.mode()) == topic)
  }

  fn flatten_lights(&self) -> Vec<&Light> {
    self.flatten_devices().into_iter().flat_map(Device::as_light).collect()
  }
  fn flatten_lights_mut(&mut self) -> Vec<&mut Light> {
    self.flatten_devices_mut().into_iter().flat_map(Device::as_light_mut).collect()
  }

  fn find_physical_light(&self, topic: &Topic) -> Option<&Light> {
    self.flatten_lights().into_iter().find(|l| &l.topic(topic.mode()) == topic)
  }
  fn find_physical_light_mut(&mut self, topic: &Topic) -> Option<&mut Light> {
    self.flatten_lights_mut().into_iter().find(|l| &l.topic(topic.mode()) == topic)
  }

  fn flatten_remotes(&self) -> Vec<&Remote> {
    self.flatten_devices().into_iter().flat_map(Device::as_remote).collect()
  }
  fn flatten_remotes_mut(&mut self) -> Vec<&mut Remote> {
    self.flatten_devices_mut().into_iter().flat_map(Device::as_remote_mut).collect()
  }

  fn find_remote(&self, topic: &Topic) -> Option<&Remote> {
    self.flatten_remotes().into_iter().find(|s| &s.topic(topic.mode()) == topic)
  }

  fn find_remote_mut(&mut self, topic: &Topic) -> Option<&mut Remote> {
    self.flatten_remotes_mut().into_iter().find(|s| &s.topic(topic.mode()) == topic)
  }

  fn flatten_sensors(&self) -> Vec<&Sensor> {
    self.flatten_devices().into_iter().flat_map(Device::as_sensor).collect()
  }
  fn flatten_sensors_mut(&mut self) -> Vec<&mut Sensor> {
    self.flatten_devices_mut().into_iter().flat_map(Device::as_sensor_mut).collect()
  }

  fn find_sensor(&self, topic: &Topic) -> Option<&Sensor> {
    self.flatten_sensors().into_iter().find(|s| &s.topic(topic.mode()) == topic)
  }

  fn find_sensor_mut(&mut self, topic: &Topic) -> Option<&mut Sensor> {
    self.flatten_sensors_mut().into_iter().find(|s| &s.topic(topic.mode()) == topic)
  }
}

pub trait EffectiveLightCollection {
  fn find_effective_light(&self, topic: &Topic) -> Option<&dyn EffectiveLight>;
  fn find_effective_light_mut(&mut self, topic: &Topic) -> Option<&mut dyn EffectiveLight>;
}

impl<T: DeviceCollection> EffectiveLight for T {
  fn turn_on(&mut self) -> Vec<(Topic, StateToMqtt)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.turn_on()).collect()
  }

  fn turn_off(&mut self) -> Vec<(Topic, StateToMqtt)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.turn_off()).collect()
  }

  fn toggle(&mut self) -> Vec<(Topic, StateToMqtt)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.toggle()).collect()
  }

  fn dim_down(&mut self) -> Vec<(Topic, StateToMqtt)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.dim_down()).collect()
  }

  fn dim_up(&mut self) -> Vec<(Topic, StateToMqtt)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.dim_up()).collect()
  }

  fn start_dim_down(&mut self) -> Vec<(Topic, StateToMqtt)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.start_dim_down()).collect()
  }

  fn start_dim_up(&mut self) -> Vec<(Topic, StateToMqtt)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.start_dim_up()).collect()
  }

  fn stop_dim(&mut self) -> Vec<(Topic, StateToMqtt)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.stop_dim()).collect()
  }

  fn change_state(&mut self, payload: RestApiPayload) -> Vec<(Topic, StateToMqtt)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.change_state(payload.clone())).collect()
  }
}

pub trait EditableHome {
  fn add_room(&mut self, name: String);
}

pub trait ReadWriteHome {
  fn read(from: &str) -> Self;
  fn persist(&self, to: &str) -> Result<()>;
}

pub trait EffectiveLight: Debug {
  fn turn_on(&mut self) -> Vec<(Topic, StateToMqtt)>;
  fn turn_off(&mut self) -> Vec<(Topic, StateToMqtt)>;
  fn toggle(&mut self) -> Vec<(Topic, StateToMqtt)>;
  fn dim_down(&mut self) -> Vec<(Topic, StateToMqtt)>;
  fn dim_up(&mut self) -> Vec<(Topic, StateToMqtt)>;
  fn start_dim_down(&mut self) -> Vec<(Topic, StateToMqtt)>;
  fn start_dim_up(&mut self) -> Vec<(Topic, StateToMqtt)>;
  fn stop_dim(&mut self) -> Vec<(Topic, StateToMqtt)>;
  fn change_state(&mut self, payload: RestApiPayload) -> Vec<(Topic, StateToMqtt)>;
}

pub trait Addressable {
  fn topic(&self, mode: TopicMode) -> Topic;
}

pub trait Scenable {
  fn trigger_update_scene(&self, queue: &UnboundedSender<Request>);
}

pub trait JsonConvertible: Sized {
  fn to_json(self) -> JsonPayload;
  fn from_str(string: &str) -> Result<Self>;
}

pub trait YamlConvertible: Sized {
  fn to_yaml(&self) -> String;
  fn from_str(string: &str) -> Result<Self>;
}

pub trait TopicConvertible: Sized {
  fn to_topic(&self) -> String;
  fn from_str(string: &str) -> Result<Self>;
}
