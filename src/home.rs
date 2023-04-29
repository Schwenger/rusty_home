use std::fs::File;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
  api::{
    payload::JsonPayload,
    topic::{Topic, TopicMode},
    traits::{
      Addressable, DeviceCollection, EditableHome, EffectiveLight, EffectiveLightCollection,
      QueryableHome, ReadWriteHome,
    },
    LightCommand, Request,
  },
  devices::Device,
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

impl DeviceCollection for Home {
  fn flatten_devices(&self) -> Vec<&Device> {
    self.rooms.iter().flat_map(Room::flatten_devices).collect()
  }

  fn flatten_devices_mut(&mut self) -> Vec<&mut Device> {
    self.rooms.iter_mut().flat_map(Room::flatten_devices_mut).collect()
  }
}

impl EffectiveLightCollection for Home {
  fn find_effective_light(&self, topic: &Topic) -> Option<&dyn EffectiveLight> {
    if &self.topic(topic.mode()) == topic {
      return Some(self);
    }
    self.rooms.iter().flat_map(|r| r.find_effective_light(topic)).last()
  }

  fn find_effective_light_mut(&mut self, topic: &Topic) -> Option<&mut dyn EffectiveLight> {
    if &self.topic(topic.mode()) == topic {
      return Some(self);
    }
    self.rooms.iter_mut().find_map(|r| r.find_effective_light_mut(topic))
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

  fn query_device(&self, topic: Topic) -> JsonPayload {
    JsonPayload::from(&self.find_physical_light(&topic).unwrap().state())
  }
}

impl Home {
  pub fn initialize(&self, queue: &UnboundedSender<Request>) {
    self
      .flatten_lights()
      .into_iter()
      .map(|l| l.topic(TopicMode::Blank))
      .map(|t| Request::LightCommand(LightCommand::QueryUpdate, t))
      .for_each(|req| queue.send(req).expect("Should work"))
  }
}
