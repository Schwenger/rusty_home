use std::fs::File;

use serde::{Deserialize, Serialize};

use crate::{
  api::{
    payload::JsonPayload,
    traits::{Addressable, EditableHome, LightCollection, QueryableHome, ReadWriteHome},
    Topic, TopicMode,
  },
  devices::Light,
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
