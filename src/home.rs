use std::fs::File;

use serde::{Deserialize, Serialize};

use crate::{
  api::{
    payload::JsonPayload,
    topic::{Topic, TopicMode},
    traits::{
      Addressable, EditableHome, EffectiveLight, LightCollection, QueryableHome, ReadWriteHome,
      RemoteCollection, SensorCollection,
    },
  },
  devices::{Light, Remote},
  Result,
};

use room::Room;

mod room;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Home {
  name: String,
  rooms: Vec<Room>,
}

impl Addressable for Home {
  fn topic(&self, mode: TopicMode) -> Topic {
    Topic::Home { mode }
  }
}

impl LightCollection for Home {
  fn flatten_lights(&self) -> Vec<&Light> {
    self.rooms.iter().flat_map(Room::flatten_lights).collect()
  }

  fn flatten_lights_mut(&mut self) -> Vec<&mut Light> {
    self.rooms.iter_mut().flat_map(|r| r.flatten_lights_mut()).collect()
  }

  fn find_light(&self, topic: &Topic) -> Option<&dyn EffectiveLight> {
    if &self.topic(topic.mode()) == topic {
      return Some(self);
    }
    self.rooms.iter().flat_map(|r| r.find_light(topic)).last()
  }

  fn find_light_mut(&mut self, topic: &Topic) -> Option<&mut dyn EffectiveLight> {
    if &self.topic(topic.mode()) == topic {
      return Some(self);
    }
    self.rooms.iter_mut().flat_map(|r| r.find_light_mut(topic)).last()
  }
}

impl RemoteCollection for Home {
  fn flatten_remotes(&self) -> Vec<&Remote> {
    self.rooms.iter().flat_map(Room::flatten_remotes).collect()
  }

  fn flatten_remotes_mut(&mut self) -> Vec<&mut Remote> {
    self.rooms.iter_mut().flat_map(|r| r.flatten_remotes_mut()).collect()
  }
}

impl SensorCollection for Home {
  fn flatten_sensors(&self) -> Vec<&crate::devices::Sensor> {
    self.rooms.iter().flat_map(Room::flatten_sensors).collect()
  }

  fn flatten_sensors_mut(&mut self) -> Vec<&mut crate::devices::Sensor> {
    self.rooms.iter_mut().flat_map(|r| r.flatten_sensors_mut()).collect()
  }
}

impl ReadWriteHome for Home {
  fn read(from: &str) -> Self {
    let content = std::fs::read_to_string(from).expect("Cannot open file.");
    let home: Home = serde_yaml::from_str(&content).expect("Cannot read home.");
    home
  }

  fn persist(&self, to: &str) -> Result<()> {
    let mut path = to.to_string();
    path.push_str(".out");
    serde_yaml::to_writer(&File::create(path)?, self)?;
    Ok(())
  }
}

impl EditableHome for Home {
  fn add_room(&mut self, name: String) {
    self.rooms.push(Room::new(name))
  }
}

impl QueryableHome for Home {
  fn query_architecture(&self) -> JsonPayload {
    JsonPayload::from(self)
  }

  fn query_device(&self, _topic: Topic) -> JsonPayload {
    todo!()
  }
}
