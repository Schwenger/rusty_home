use std::fs::File;

use chrono::{Duration, Local, NaiveTime};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
  api::{
    payload::JsonPayload,
    request::{LightCommand, Request},
    topic::{Topic, TopicMode},
    traits::{
      Addressable, DeviceCollection, EditableHome, EffectiveLight, EffectiveLightCollection,
      QueryableHome, ReadWriteHome, Scenable,
    },
  },
  convert::{RestApiPayload, StateToMqtt},
  devices::{Device, DeviceTrait},
  scene::{Comparison, DeviceStateTrigger, Effect, Scene, TimeTrigger, Trigger},
  Result,
};

use room::Room;

mod room;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Home {
  name: String,
  rooms: Vec<Room>,
  scenes: Vec<Scene>,
}

impl Scenable for Home {
  fn trigger_update_scene(&self, queue: &UnboundedSender<Request>) {
    for scene in self.scenes.iter() {
      let Scene { trigger, effect } = scene;
      let triggered = self.evaluate_trigger(trigger);
      if triggered {
        queue.send(self.execute_effect(effect)).unwrap();
      }
    }
  }
}

impl Home {
  fn evaluate_trigger(&self, trigger: &Trigger) -> bool {
    match trigger {
      Trigger::And(a, b) => self.evaluate_trigger(a.as_ref()) && self.evaluate_trigger(b.as_ref()),
      Trigger::DeviceState(dst) => self.evaluate_device_state_trigger(dst),
      Trigger::Time(TimeTrigger { from, duration }) => self.evaluate_time_trigger(*from, *duration),
    }
  }

  fn evaluate_time_trigger(&self, from: NaiveTime, duration: Duration) -> bool {
    let now = Local::now().time();
    if from < from + duration {
      from < now && now < from + duration
    } else {
      from < now && now > from + duration
    }
  }

  fn evaluate_device_state_trigger(&self, dst: &DeviceStateTrigger) -> bool {
    let DeviceStateTrigger { target, field, op } = dst;
    let device = self.find_device(target).unwrap();
    let state = device.query_state().to_json_value(false);
    let field = state.get(field);
    if field.is_none() {
      return false;
    }
    let field = field.unwrap();
    match op {
      Comparison::BoolComparison { pivot } => field.as_bool().map(|c| c == *pivot).unwrap_or(false),
      Comparison::Equality { value } => &field.to_string() == value,
    }
  }

  fn execute_effect(&self, effect: &Effect) -> Request {
    let Effect { ref target, command } = *effect;
    let mut payload = RestApiPayload { topic: Some(target.clone()), ..RestApiPayload::default() };
    payload.topic = Some(target.clone());
    assert_ne!(command, LightCommand::ChangeState);
    Request::LightCommand(command, payload)
  }
}

impl Addressable for Home {
  fn topic(&self, mode: TopicMode) -> Topic {
    Topic::Home { mode }
  }
}

impl DeviceCollection for Home {
  fn flatten_devices(&self) -> Vec<&Device> {
    self.rooms.iter().flat_map(Room::flatten_devices).collect()
  }

  fn flatten_devices_mut(&mut self) -> Vec<&mut Device> {
    self.rooms.iter_mut().flat_map(Room::flatten_devices_mut).collect()
  }
}

impl EffectiveLightCollection for Home {
  fn find_effective_light(&self, topic: &Topic) -> Option<&dyn EffectiveLight> {
    if &self.topic(topic.mode()) == topic {
      return Some(self);
    }
    self.rooms.iter().flat_map(|r| r.find_effective_light(topic)).last()
  }

  fn find_effective_light_mut(&mut self, topic: &Topic) -> Option<&mut dyn EffectiveLight> {
    if &self.topic(topic.mode()) == topic {
      return Some(self);
    }
    self.rooms.iter_mut().find_map(|r| r.find_effective_light_mut(topic))
  }
}

impl ReadWriteHome for Home {
  fn read(from: &str) -> Self {
    let content = std::fs::read_to_string(from).expect("Cannot open file.");
    let home: Home = serde_yaml::from_str(&content).expect("Cannot read home.");
    home
  }

  fn persist(&self, to: &str) -> Result<()> {
    let mut path = to.to_string();
    path.push_str(".out");
    serde_yaml::to_writer(&File::create(path)?, self)?;
    Ok(())
  }
}

impl EditableHome for Home {
  fn add_room(&mut self, name: String) {
    self.rooms.push(Room::new(name))
  }
}

impl QueryableHome for Home {
  fn query_architecture(&self) -> JsonPayload {
    JsonPayload::from(self)
  }

  fn query_device(&self, topic: Topic) -> StateToMqtt {
    self.find_device(&topic).unwrap().query_state()
  }
}
