use palette::{DarkenAssign, Hsv, LightenAssign};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::payload::MqttPayload;
use crate::api::topic::{DeviceKind, Topic, TopicMode};
use crate::api::traits::{Addressable, EffectiveLight, LightCollection};
use crate::common::Scalar;
use crate::error::HomeBaseError;
use crate::Result;

use super::{Device, DeviceModel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
  name: String,
  model: DeviceModel,
  icon: String,
  room: String,
  pseudo_kind: Option<DeviceKind>,
  #[serde(skip)]
  state: LightState,
}

impl Light {
  pub fn state(&self) -> LightState {
    self.state
  }
}

impl Device for Light {
  fn kind(&self) -> DeviceKind {
    if let Some(k) = self.pseudo_kind {
      return k;
    }
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
  fn turn_on(&mut self) -> Vec<(Topic, MqttPayload)> {
    if self.state.on {
      return vec![];
    }
    self.state.on = true;
    vec![(
      self.topic(TopicMode::Set),
      MqttPayload::new().with_state_change(self.state.on).with_transition(),
    )]
  }

  fn turn_off(&mut self) -> Vec<(Topic, MqttPayload)> {
    if !self.state.on {
      return vec![];
    }
    self.state.on = false;
    vec![(
      self.topic(TopicMode::Set),
      MqttPayload::new().with_state_change(self.state.on).with_transition(),
    )]
  }

  fn toggle(&mut self) -> Vec<(Topic, MqttPayload)> {
    if self.state.on {
      self.turn_off()
    } else {
      self.turn_on()
    }
  }

  fn dim_down(&mut self) -> Vec<(Topic, MqttPayload)> {
    self.state.dim_down();
    vec![(
      self.topic(TopicMode::Set),
      MqttPayload::new().with_brightness_change(self.state.brightness()).with_transition(),
    )]
  }

  fn dim_up(&mut self) -> Vec<(Topic, MqttPayload)> {
    self.state.dim_up();
    vec![(
      self.topic(TopicMode::Set),
      MqttPayload::new().with_brightness_change(self.state.brightness()).with_transition(),
    )]
  }

  fn start_dim_down(&mut self) -> Vec<(Topic, MqttPayload)> {
    vec![(
      self.topic(TopicMode::Set),
      MqttPayload::new().with_start_dimming(false).with_transition(),
    )]
  }

  fn start_dim_up(&mut self) -> Vec<(Topic, MqttPayload)> {
    vec![(
      self.topic(TopicMode::Set),
      MqttPayload::new().with_start_dimming(true).with_transition(),
    )]
  }

  fn stop_dim(&mut self) -> Vec<(Topic, MqttPayload)> {
    vec![
      (self.topic(TopicMode::Set), MqttPayload::new().with_stop_dimming().with_transition()),
      (self.topic(TopicMode::Set), MqttPayload::new().with_state_query()),
    ]
  }

  fn update_state(&mut self, state: LightState) {
    self.state = state;
  }

  fn query_update(&self) -> Vec<(Topic, MqttPayload)> {
    vec![(self.topic(TopicMode::Get), MqttPayload::new().with_state_query())]
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightGroup {
  name: String,
  atomics: Vec<Light>,
  subgroups: Vec<LightGroup>,
  room: String,
}

impl LightGroup {
  pub fn new(name: String, room: String) -> Self {
    LightGroup { name, atomics: vec![], subgroups: vec![], room }
  }
}

impl Addressable for LightGroup {
  fn topic(&self, mode: TopicMode) -> Topic {
    Topic::Group { room: self.room.clone(), groups: vec![], name: self.name.clone(), mode }
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

  fn find_light(&self, topic: &Topic) -> Option<&dyn EffectiveLight> {
    if &self.topic(topic.mode()) == topic {
      return Some(self);
    }
    if let Some(res) = self.atomics.iter().find(|l| &l.topic(topic.mode()) == topic) {
      return Some(res);
    }
    self.subgroups.iter().flat_map(|grp| grp.find_light(topic)).last()
  }

  fn find_light_mut(&mut self, topic: &Topic) -> Option<&mut dyn EffectiveLight> {
    if &self.topic(topic.mode()) == topic {
      return Some(self);
    }
    if let Some(res) = self.atomics.iter_mut().find(|l| &l.topic(topic.mode()) == topic) {
      return Some(res);
    }
    self.subgroups.iter_mut().find_map(|grp| grp.find_light_mut(topic))
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct LightState {
  pub on: bool,
  pub color: Hsv,
}

impl Default for LightState {
  fn default() -> Self {
    Self { on: false, color: Hsv::new(1.0, 1.0, 1.0) }
  }
}

impl LightState {
  pub fn from_payload(value: Value, model: DeviceModel) -> Result<Self> {
    if value.get("state").is_none() {
      return Err(HomeBaseError::InvalidLightState);
    }
    let on = value.get("state").unwrap() == "ON";
    let color = if value.get("brightness").is_some() {
      MqttPayload::read_color(model, value)
    } else {
      Hsv::new(1.0, 1.0, 1.0)
    };
    Ok(LightState { on, color })
  }
  pub fn brightness(&self) -> Scalar {
    Scalar::from(self.color.value as f64)
  }
  pub fn dim_down(&mut self) {
    self.color.darken_assign(0.8);
  }
  pub fn dim_up(&mut self) {
    self.color.lighten_assign(0.8);
  }
}
