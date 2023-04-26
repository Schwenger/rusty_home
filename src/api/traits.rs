use crate::api::Topic;
use crate::devices::{DeviceModel, Light};
use crate::Result;

use super::payload::{JsonPayload, MqttPayload};
use super::{DeviceKind, TopicMode};

pub trait QueryableHome {
  fn query_architecture(&self) -> JsonPayload;
  fn query_device(&self, topic: Topic) -> JsonPayload;
}

pub trait LightCollection {
  fn flatten_lights(&self) -> Vec<&Light>;
  fn flatten_lights_mut(&mut self) -> Vec<&mut Light>;
}

pub trait Searchable {
  fn find_light(&self, topic: &Topic) -> Option<&dyn EffectiveLight>;
  fn find_light_mut(&mut self, topic: &Topic) -> Option<&mut dyn EffectiveLight>;
}

impl<T: Addressable + LightCollection + EffectiveLight> Searchable for T {
  fn find_light(&self, topic: &Topic) -> Option<&dyn EffectiveLight> {
    if &self.topic(topic.mode()) == topic {
      return Some(self);
    }
    self.flatten_lights().into_iter().find(|l| &l.topic(topic.mode()) == topic).map(|l| {
      let x: &dyn EffectiveLight = l;
      x
    })
  }

  fn find_light_mut(&mut self, topic: &Topic) -> Option<&mut dyn EffectiveLight> {
    if &self.topic(topic.mode()) == topic {
      return Some(self);
    }
    self.flatten_lights_mut().into_iter().find(|l| &l.topic(topic.mode()) == topic).map(|l| {
      let x: &mut dyn EffectiveLight = l;
      x
    })
  }
}

impl<T: LightCollection> EffectiveLight for T {
  fn turn_on(&mut self) -> Option<MqttPayload> {
    todo!()
  }

  fn turn_off(&mut self) -> Option<MqttPayload> {
    todo!()
  }

  fn toggle(&mut self) -> Option<MqttPayload> {
    todo!()
  }

  fn dim_down(&mut self) -> Option<MqttPayload> {
    todo!()
  }

  fn dim_up(&mut self) -> Option<MqttPayload> {
    todo!()
  }

  fn start_dim_down(&mut self) -> Option<MqttPayload> {
    todo!()
  }

  fn start_dim_up(&mut self) -> Option<MqttPayload> {
    todo!()
  }

  fn stop_dim(&mut self) -> Option<MqttPayload> {
    todo!()
  }
}

pub trait EditableHome {
  fn add_room(&mut self, name: String);
}

pub trait ReadWriteHome {
  fn read(from: &str) -> Self;
  fn persist(&self, to: &str) -> Result<()>;
}

pub trait EffectiveLight {
  fn turn_on(&mut self) -> Option<MqttPayload>;
  fn turn_off(&mut self) -> Option<MqttPayload>;
  fn toggle(&mut self) -> Option<MqttPayload>;
  fn dim_down(&mut self) -> Option<MqttPayload>;
  fn dim_up(&mut self) -> Option<MqttPayload>;
  fn start_dim_down(&mut self) -> Option<MqttPayload>;
  fn start_dim_up(&mut self) -> Option<MqttPayload>;
  fn stop_dim(&mut self) -> Option<MqttPayload>;
}

pub trait Addressable {
  fn topic(&self, mode: TopicMode) -> Topic;
}

pub trait DeviceTrait: Addressable {
  fn kind(&self) -> DeviceKind;
  fn model(&self) -> DeviceModel;
  fn name(&self) -> &str;
  fn room(&self) -> &str;
}

impl<T: DeviceTrait> Addressable for T {
  fn topic(&self, mode: TopicMode) -> Topic {
    Topic::Device {
      device: self.kind(),
      room: self.room().to_string(),
      groups: vec![],
      name: self.name().to_string(),
      mode,
    }
  }
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
