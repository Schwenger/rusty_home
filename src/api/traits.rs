use crate::api::Topic;

pub trait Queryable {
  type Output;
  fn query_architecture(&self) -> Self::Output;
  fn query_device(&self, topic: Topic) -> Self::Output;
}

pub trait Editable {}