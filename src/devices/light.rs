use palette::{DarkenAssign, Hsv, IntoColor, LightenAssign, Yxy};
use serde::{Deserialize, Serialize};

use crate::api::payload::MqttPayload;
use crate::api::topic::{DeviceKind, Topic, TopicMode};
use crate::api::traits::{Addressable, DeviceCollection, EffectiveLight, EffectiveLightCollection};
use crate::common::Scalar;
use crate::mqtt::{MqttColor, MqttState};
use crate::web_server::RestApiPayload;

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

impl Light {
  pub fn state(&self) -> LightState {
    self.state
  }
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

  fn update_state(&mut self, state: MqttState) {
    self.state.with_mqtt_state(self.model(), state);
  }

  fn query_state(&self) -> MqttState {
    self.state.to_mqtt_state(self.model())
  }

  fn query_update(&self) -> MqttPayload {
    MqttPayload::new().with_state_query()
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

  fn change_state(&mut self, payload: RestApiPayload) -> Vec<(Topic, MqttPayload)> {
    let mqtt = if payload.hue.is_some() {
      assert!(payload.sat.is_some());
      assert!(payload.val.is_some());
      self.state.color = Hsv::new(
        payload.hue.unwrap() as f32,
        payload.sat.unwrap() as f32,
        payload.val.unwrap() as f32,
      );
      MqttPayload::new().with_color_change(self.state.color)
    } else if let Some(val) = payload.val {
      MqttPayload::new().with_brightness_change(val.into())
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
    self.subgroups.iter().flat_map(|grp| grp.find_effective_light(topic)).last()
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
  pub fn with_mqtt_state(&mut self, model: DeviceModel, state: MqttState) {
    println!("\n\nwith mqtt state\n");
    println!("received: {:?}", state);
    if model.capable_of(Capability::State) {
      self.on = state.state.unwrap().into();
    }
    if model.capable_of(Capability::Brightness) {
      self.color.value = state.brightness(model.max_brightness()).unwrap().inner() as f32;
    }
    if model.capable_of(Capability::Color) {
      let color = state.color.unwrap();
      let (x, y) = color.x_y();
      let val = self.brightness().inner() as f32;
      self.color = Yxy::new(x, y, val).into_color();
      println!(
        "{:?}, hue: {}, val: {val}",
        self.color,
        self.color.hue.into_radians() + std::f32::consts::PI,
      );
    }
  }

  pub fn to_mqtt_state(&self, model: DeviceModel) -> MqttState {
    let mut res = MqttState::default();
    if model.capable_of(Capability::State) {
      res.state = Some(self.on.into());
    }
    if model.capable_of(Capability::Brightness) {
      res.set_brightness(self.brightness(), model.max_brightness());
    }
    if model.capable_of(Capability::Color) {
      res.color = Some(MqttColor::new(self.color));
    }
    res
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
