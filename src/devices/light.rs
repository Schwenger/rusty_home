use serde::{Deserialize, Serialize};

use crate::api::topic::{DeviceKind, Topic, TopicMode};
use crate::api::traits::{Addressable, DeviceCollection, EffectiveLight, EffectiveLightCollection};
use crate::convert::{HsvColor, RestApiPayload, StateFromMqtt, StateToMqtt, Val};

use super::{Capability, Device, DeviceModel, DeviceTrait};

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

impl DeviceTrait for Light {
  fn virtual_kind(&self) -> DeviceKind {
    self.pseudo_kind.unwrap_or(DeviceKind::Light)
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

  fn update_state(&mut self, state: StateFromMqtt) {
    self.state.with_mqtt_state(self.model(), state);
  }

  fn query_state(&self) -> StateToMqtt {
    self.state.to_mqtt_state(self.model())
  }

  fn query_update(&self) -> StateToMqtt {
    StateToMqtt::empty().with_state(None)
  }

  fn query_history(&self) -> Vec<StateToMqtt> {
    vec![self.query_state()]
  }
}

impl EffectiveLight for Light {
  fn turn_on(&mut self, brightness: Option<Val>) -> Vec<(Topic, StateToMqtt)> {
    if self.state.on {
      return vec![];
    }
    self.state.on = true;
    vec![(
      self.topic(TopicMode::Set),
      StateToMqtt::empty().with_state(Some(self.state.on)).with_value(brightness).with_transition(),
    )]
  }

  fn turn_off(&mut self) -> Vec<(Topic, StateToMqtt)> {
    if !self.state.on {
      return vec![];
    }
    self.state.on = false;
    vec![(
      self.topic(TopicMode::Set),
      StateToMqtt::empty().with_state(Some(self.state.on)).with_transition(),
    )]
  }

  fn toggle(&mut self) -> Vec<(Topic, StateToMqtt)> {
    if self.state.on {
      self.turn_off()
    } else {
      self.turn_on(None)
    }
  }

  fn dim_down(&mut self) -> Vec<(Topic, StateToMqtt)> {
    vec![
      (self.topic(TopicMode::Set), StateToMqtt::empty().with_brightness_step(-1)),
      (self.topic(TopicMode::Get), StateToMqtt::empty().with_value(None)),
    ]
  }

  fn dim_up(&mut self) -> Vec<(Topic, StateToMqtt)> {
    vec![
      (self.topic(TopicMode::Set), StateToMqtt::empty().with_brightness_step(1)),
      (self.topic(TopicMode::Get), StateToMqtt::empty().with_value(None)),
    ]
  }

  fn start_dim_down(&mut self) -> Vec<(Topic, StateToMqtt)> {
    vec![(self.topic(TopicMode::Set), StateToMqtt::empty().with_brightness_move(-1))]
  }

  fn start_dim_up(&mut self) -> Vec<(Topic, StateToMqtt)> {
    vec![(self.topic(TopicMode::Set), StateToMqtt::empty().with_brightness_move(1))]
  }

  fn stop_dim(&mut self) -> Vec<(Topic, StateToMqtt)> {
    vec![
      (self.topic(TopicMode::Set), StateToMqtt::empty().with_brightness_move(0)),
      (self.topic(TopicMode::Get), StateToMqtt::empty().with_value(None)),
    ]
  }

  fn change_state(&mut self, payload: RestApiPayload) -> Vec<(Topic, StateToMqtt)> {
    let mqtt = if payload.hue.is_some() {
      assert!(payload.sat.is_some());
      assert!(payload.val.is_some());
      self.state.color =
        HsvColor::new(payload.hue.unwrap(), payload.sat.unwrap(), payload.val.unwrap());
      StateToMqtt::empty().with_color_change(&self.state.color).with_transition()
    } else if let Some(val) = payload.val {
      self.state.color.with_val(val);
      StateToMqtt::empty().with_value(Some(val)).with_transition()
    } else {
      return vec![];
    };
    vec![(self.topic(TopicMode::Set), mqtt)]
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightGroup {
  name: String,
  #[serde(serialize_with = "crate::devices::serialize_light_sequence")]
  #[serde(deserialize_with = "crate::devices::deserialize_light_sequence")]
  atomics: Vec<Device>,
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

impl DeviceCollection for LightGroup {
  fn flatten_devices(&self) -> Vec<&Device> {
    let subs = self.subgroups.iter().flat_map(LightGroup::flatten_devices);
    self.atomics.iter().chain(subs).collect()
  }

  fn flatten_devices_mut(&mut self) -> Vec<&mut Device> {
    let subs = self.subgroups.iter_mut().flat_map(LightGroup::flatten_devices_mut);
    self.atomics.iter_mut().chain(subs).collect()
  }
}

impl EffectiveLightCollection for LightGroup {
  fn find_effective_light(&self, topic: &Topic) -> Option<&dyn EffectiveLight> {
    if &self.topic(topic.mode()) == topic {
      return Some(self);
    }
    if let Some(res) = self.atomics.iter().find(|l| &l.topic(topic.mode()) == topic) {
      return Some(res.as_light().unwrap());
    }
    self.subgroups.iter().filter_map(|grp| grp.find_effective_light(topic)).last()
  }

  fn find_effective_light_mut(&mut self, topic: &Topic) -> Option<&mut dyn EffectiveLight> {
    if &self.topic(topic.mode()) == topic {
      return Some(self);
    }
    if let Some(res) = self.atomics.iter_mut().find(|l| &l.topic(topic.mode()) == topic) {
      return Some(res.as_light_mut().unwrap());
    }
    self.subgroups.iter_mut().find_map(|grp| grp.find_effective_light_mut(topic))
  }
}

#[allow(missing_copy_implementations)] // Avoid accidental copying.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub struct LightState {
  pub on: bool,
  pub color: HsvColor,
}

impl LightState {
  pub fn with_mqtt_state(&mut self, model: DeviceModel, state: StateFromMqtt) {
    if let Some(on) = state.state() {
      assert!(model.capable_of(Capability::State));
      self.on = on
    }
    if let Some(val) = state.val() {
      assert!(model.capable_of(Capability::Brightness));
      self.color.with_val(val);
    }
    if let Some(color) = state.hsv_color() {
      assert!(model.capable_of(Capability::Color));
      self.color = color;
    }
  }

  pub fn to_mqtt_state(&self, model: DeviceModel) -> StateToMqtt {
    let mut res = StateToMqtt::default();
    if model.capable_of(Capability::State) {
      res = res.with_state(Some(self.on));
    }
    if model.capable_of(Capability::Brightness) && !model.capable_of(Capability::Color) {
      res = res.with_value(Some(self.color.val()));
    }
    if model.capable_of(Capability::Color) {
      res = res.with_color_change(&self.color);
    }
    res
  }
}
