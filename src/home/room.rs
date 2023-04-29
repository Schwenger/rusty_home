use serde::{Deserialize, Serialize};

use crate::{
  api::{
    topic::{Topic, TopicMode},
    traits::{Addressable, DeviceCollection, EffectiveLight, EffectiveLightCollection},
  },
  devices::{Device, LightGroup},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
  name: String,
  lights: LightGroup,
  icon: String,
  #[serde(serialize_with = "crate::devices::serialize_sensor_sequence")]
  #[serde(deserialize_with = "crate::devices::deserialize_sensor_sequence")]
  sensors: Vec<Device>,
  #[serde(serialize_with = "crate::devices::serialize_remote_sequence")]
  #[serde(deserialize_with = "crate::devices::deserialize_remote_sequence")]
  remotes: Vec<Device>,
}

impl Room {
  pub fn new(name: String) -> Self {
    Room {
      name: name.clone(),
      lights: LightGroup::new("Main".to_string(), name),
      sensors: vec![],
      remotes: vec![],
      icon: String::from("square.split.bottomrightquarter.fill"),
    }
  }
}

impl Addressable for Room {
  fn topic(&self, mode: TopicMode) -> Topic {
    Topic::Room { name: self.name.clone(), mode }
  }
}

impl DeviceCollection for Room {
  fn flatten_devices(&self) -> Vec<&Device> {
    self.lights.flatten_devices().into_iter().chain(self.remotes.iter()).collect()
  }

  fn flatten_devices_mut(&mut self) -> Vec<&mut Device> {
    self.lights.flatten_devices_mut().into_iter().chain(self.remotes.iter_mut()).collect()
  }
}

impl EffectiveLightCollection for Room {
  fn find_effective_light(&self, topic: &Topic) -> Option<&dyn EffectiveLight> {
    if &self.topic(topic.mode()) == topic {
      return Some(self);
    }
    self.lights.find_effective_light(topic)
  }

  fn find_effective_light_mut(&mut self, topic: &Topic) -> Option<&mut dyn EffectiveLight> {
    if &self.topic(topic.mode()) == topic {
      return Some(self);
    }
    self.lights.find_effective_light_mut(topic)
  }
}
