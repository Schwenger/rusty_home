use serde::{Deserialize, Serialize};

use crate::api::payload::MqttPayload;
use crate::api::traits::{DeviceTrait, EffectiveLight, LightCollection};
use crate::api::DeviceKind;
use crate::common::Scalar;

use super::DeviceModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
  name: String,
  model: DeviceModel,
  icon: String,
  room: String,
  #[serde(skip)]
  on: bool,
  #[serde(skip)]
  brightness: Scalar,
}

impl DeviceTrait for Light {
  fn kind(&self) -> DeviceKind {
    self.model.kind()
  }

  fn model(&self) -> DeviceModel {
    self.model
  }

  fn name(&self) -> &str {
    &self.name
  }

  fn room(&self) -> &str {
    &self.room
  }
}

impl EffectiveLight for Light {
  fn turn_on(&mut self) -> Option<MqttPayload> {
    if self.on {
      return None;
    }
    self.on = true;
    Some(MqttPayload::new().with_state_change(self.on).with_transition())
  }

  fn turn_off(&mut self) -> Option<MqttPayload> {
    if !self.on {
      return None;
    }
    self.on = false;
    Some(MqttPayload::new().with_state_change(self.on).with_transition())
  }

  fn toggle(&mut self) -> Option<MqttPayload> {
    if self.on {
      self.turn_off()
    } else {
      self.turn_on()
    }
  }

  fn dim_down(&mut self) -> Option<MqttPayload> {
    self.brightness -= 0.2;
    Some(MqttPayload::new().with_brightness_change(self.brightness).with_transition())
  }

  fn dim_up(&mut self) -> Option<MqttPayload> {
    self.brightness += 0.2;
    Some(MqttPayload::new().with_brightness_change(self.brightness).with_transition())
  }

  fn start_dim_down(&mut self) -> Option<MqttPayload> {
    Some(MqttPayload::new().with_start_dimming(false).with_transition())
  }

  fn start_dim_up(&mut self) -> Option<MqttPayload> {
    Some(MqttPayload::new().with_start_dimming(true).with_transition())
  }

  fn stop_dim(&mut self) -> Option<MqttPayload> {
    // ToDo: Query state to keep internal brightness up to date.
    Some(MqttPayload::new().with_stop_dimming().with_transition())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightGroup {
  name: String,
  atomics: Vec<Light>,
  subgroups: Vec<LightGroup>,
}

impl LightGroup {
  pub fn new(name: String) -> Self {
    LightGroup { name, atomics: vec![], subgroups: vec![] }
  }
}

impl LightCollection for LightGroup {
  fn flatten_lights(&self) -> Vec<&Light> {
    let subs = self.subgroups.iter().flat_map(LightGroup::flatten_lights);
    self.atomics.iter().chain(subs).collect()
  }

  fn flatten_lights_mut(&mut self) -> Vec<&mut Light> {
    let subs = self.subgroups.iter_mut().flat_map(|l| l.flatten_lights_mut());
    self.atomics.iter_mut().chain(subs).collect()
  }
  // fn find_light(&self, topic: &Topic) -> Option<&dyn EffectiveLight> {
  //   if let Some(res) = self.atomics.iter().find(|l| &l.topic(topic.mode()) == topic) {
  //     return Some(res);
  //   }
  //   self.subgroups.iter().flat_map(|grp| grp.find_light(topic)).last()
  // }

  // fn find_light_mut(&mut self, topic: &Topic) -> Option<&mut dyn EffectiveLight> {
  //   if let Some(res) = self.atomics.iter_mut().find(|l| &l.topic(topic.mode()) == topic) {
  //     return Some(res);
  //   }
  //   self.subgroups.iter_mut().flat_map(|grp| grp.find_light_mut(topic)).last()
  // }
}
