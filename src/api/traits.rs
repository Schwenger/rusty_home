use crate::api::Topic;
use crate::Result;

use super::JsonPayload;

pub trait Queryable {
  type Output;
  fn query_architecture(&self) -> Self::Output;
  fn query_device(&self, topic: Topic) -> Self::Output;
}

pub trait Editable {
  fn add_room(&mut self, name: String);
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
