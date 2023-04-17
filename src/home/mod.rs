mod room;

use room::Room;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Home {
  name: String,
  rooms: Vec<Room>,
}
