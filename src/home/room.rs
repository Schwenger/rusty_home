use serde::{Deserialize, Serialize};

use crate::{
  api::{
    topic::{Topic, TopicMode},
    traits::{Addressable, EffectiveLight, LightCollection, RemoteCollection, SensorCollection},
  },
  devices::{Light, LightGroup, Remote, Sensor},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
  name: String,
  lights: LightGroup,
  icon: String,
  sensors: Vec<Sensor>,
  remotes: Vec<Remote>,
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

impl LightCollection for Room {
  fn flatten_lights(&self) -> Vec<&Light> {
    self.lights.flatten_lights()
  }

  fn flatten_lights_mut(&mut self) -> Vec<&mut Light> {
    self.lights.flatten_lights_mut()
  }

  fn find_light(&self, topic: &Topic) -> Option<&dyn EffectiveLight> {
    if &self.topic(topic.mode()) == topic {
      return Some(self);
    }
    self.lights.find_light(topic)
  }

  fn find_light_mut(&mut self, topic: &Topic) -> Option<&mut dyn EffectiveLight> {
    if &self.topic(topic.mode()) == topic {
      return Some(self);
    }
    self.lights.find_light_mut(topic)
  }
}

impl RemoteCollection for Room {
  fn flatten_remotes(&self) -> Vec<&Remote> {
    self.remotes.iter().collect()
  }

  fn flatten_remotes_mut(&mut self) -> Vec<&mut Remote> {
    self.remotes.iter_mut().collect()
  }
}

impl SensorCollection for Room {
  fn flatten_sensors(&self) -> Vec<&Sensor> {
    self.sensors.iter().collect()
  }

  fn flatten_sensors_mut(&mut self) -> Vec<&mut Sensor> {
    self.sensors.iter_mut().collect()
  }
}
