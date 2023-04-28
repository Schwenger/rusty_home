use core::fmt::Debug;

use crate::api::topic::Topic;
use crate::devices::light::LightState;
use crate::devices::{Device, Light, Remote, Sensor};
use crate::Result;

use super::payload::{JsonPayload, MqttPayload};
use super::topic::TopicMode;

pub trait QueryableHome {
  fn query_architecture(&self) -> JsonPayload;
  fn query_device(&self, topic: Topic) -> JsonPayload;
}

pub trait LightCollection: Debug {
  fn flatten_lights(&self) -> Vec<&Light>;
  fn flatten_lights_mut(&mut self) -> Vec<&mut Light>;
  fn find_light(&self, topic: &Topic) -> Option<&dyn EffectiveLight>;
  fn find_light_mut(&mut self, topic: &Topic) -> Option<&mut dyn EffectiveLight>;
  fn find_physical_light(&self, topic: &Topic) -> Option<&Light> {
    self.flatten_lights().into_iter().find(|l| &l.topic(topic.mode()) == topic)
  }
  fn find_physical_light_mut(&mut self, topic: &Topic) -> Option<&mut Light> {
    self.flatten_lights_mut().into_iter().find(|l| &l.topic(topic.mode()) == topic)
  }
}

pub trait RemoteCollection {
  fn flatten_remotes(&self) -> Vec<&Remote>;
  fn flatten_remotes_mut(&mut self) -> Vec<&mut Remote>;

  fn find_remote(&self, topic: &Topic) -> Option<&Remote> {
    self.flatten_remotes().into_iter().find(|s| &s.topic(topic.mode()) == topic)
  }

  fn find_remote_mut(&mut self, topic: &Topic) -> Option<&mut Remote> {
    self.flatten_remotes_mut().into_iter().find(|s| &s.topic(topic.mode()) == topic)
  }
}

pub trait SensorCollection {
  fn flatten_sensors(&self) -> Vec<&Sensor>;
  fn flatten_sensors_mut(&mut self) -> Vec<&mut Sensor>;

  fn find_sensor(&self, topic: &Topic) -> Option<&Sensor> {
    self.flatten_sensors().into_iter().find(|s| &s.topic(topic.mode()) == topic)
  }

  fn find_sensor_mut(&mut self, topic: &Topic) -> Option<&mut Sensor> {
    self.flatten_sensors_mut().into_iter().find(|s| &s.topic(topic.mode()) == topic)
  }
}

pub trait DeviceCollection {
  fn flatten_devices(&self) -> Vec<&dyn Device>;
}

impl<T> DeviceCollection for T
where
  T: SensorCollection + RemoteCollection + LightCollection,
{
  fn flatten_devices(&self) -> Vec<&dyn Device> {
    std::iter::empty()
      .chain(self.flatten_lights().into_iter().map(|x| {
        let y: &dyn Device = x;
        y
      }))
      .chain(self.flatten_remotes().into_iter().map(|x| {
        let y: &dyn Device = x;
        y
      }))
      .chain(self.flatten_sensors().into_iter().map(|x| {
        let y: &dyn Device = x;
        y
      }))
      .collect()
  }
}

impl<T: LightCollection> EffectiveLight for T {
  fn turn_on(&mut self) -> Vec<(Topic, MqttPayload)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.turn_on()).collect()
  }

  fn turn_off(&mut self) -> Vec<(Topic, MqttPayload)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.turn_off()).collect()
  }

  fn toggle(&mut self) -> Vec<(Topic, MqttPayload)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.toggle()).collect()
  }

  fn dim_down(&mut self) -> Vec<(Topic, MqttPayload)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.dim_down()).collect()
  }

  fn dim_up(&mut self) -> Vec<(Topic, MqttPayload)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.dim_up()).collect()
  }

  fn start_dim_down(&mut self) -> Vec<(Topic, MqttPayload)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.start_dim_down()).collect()
  }

  fn start_dim_up(&mut self) -> Vec<(Topic, MqttPayload)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.start_dim_up()).collect()
  }

  fn stop_dim(&mut self) -> Vec<(Topic, MqttPayload)> {
    self.flatten_lights_mut().into_iter().flat_map(|l| l.stop_dim()).collect()
  }

  fn update_state(&mut self, state: LightState) {
    self.flatten_lights_mut().into_iter().for_each(|l| l.update_state(state))
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
  fn turn_on(&mut self) -> Vec<(Topic, MqttPayload)>;
  fn turn_off(&mut self) -> Vec<(Topic, MqttPayload)>;
  fn toggle(&mut self) -> Vec<(Topic, MqttPayload)>;
  fn dim_down(&mut self) -> Vec<(Topic, MqttPayload)>;
  fn dim_up(&mut self) -> Vec<(Topic, MqttPayload)>;
  fn start_dim_down(&mut self) -> Vec<(Topic, MqttPayload)>;
  fn start_dim_up(&mut self) -> Vec<(Topic, MqttPayload)>;
  fn stop_dim(&mut self) -> Vec<(Topic, MqttPayload)>;
  fn update_state(&mut self, state: LightState);
}

pub trait Addressable {
  fn topic(&self, mode: TopicMode) -> Topic;
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
