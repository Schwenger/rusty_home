use std::fs::File;

use serde::{Deserialize, Serialize};

use crate::{
  api::{Editable, JsonPayload, Queryable},
  config::GlobalConfig,
  Result,
};

use room::Room;

mod room;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Home {
  name: String,
  rooms: Vec<Room>,
}

impl From<&GlobalConfig> for Home {
  fn from(cfg: &GlobalConfig) -> Self {
    let content = std::fs::read_to_string(&cfg.home.dir).expect("Cannot open file.");
    let home: Home = serde_yaml::from_str(&content).expect("Cannot read home.");
    home
  }
}

impl Home {
  pub fn persist(&self, to: &str) -> Result<()> {
    let mut path = to.to_string();
    path.push_str(".out");
    serde_yaml::to_writer(&File::create(path)?, self)?;
    Ok(())
  }
}

impl Editable for Home {
  fn add_room(&mut self, name: String) {
    self.rooms.push(Room::new(name))
  }
}

impl Queryable for Home {
  type Output = JsonPayload;

  fn query_architecture(&self) -> Self::Output {
    JsonPayload::from(self)
  }

  fn query_device(&self, _topic: crate::api::Topic) -> Self::Output {
    todo!()
  }
}
