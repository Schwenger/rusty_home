#[allow(dead_code)]
pub enum Request {
  Query(Query),
  Command(Command),
}

pub enum Query {}

pub enum Command {}
