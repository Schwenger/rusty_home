use serde::{Deserialize, Serialize};

use crate::{
  api::{JsonPayload, Queryable, Editable},
  config::GlobalConfig,
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
    let path = &cfg.home.dir;
    let content = std::fs::read_to_string(path).expect("Cannot open file.");
    let home: Home = serde_yaml::from_str(&content).expect("Cannot read home.");
    println!("{:?}", home);
    home
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
