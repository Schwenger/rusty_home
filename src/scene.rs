use chrono::{Duration, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::api::{request::LightCommand, topic::Topic};

#[derive(Debug, Clone)]
pub struct Scene {
  pub trigger: Trigger,
  pub effect: Effect,
}

#[derive(Debug, Clone)]
pub enum Trigger {
  DeviceState(DeviceStateTrigger),
  And(Box<Trigger>, Box<Trigger>),
  Time { from: NaiveTime, duration: Duration },
}

#[derive(Debug, Clone)]
pub struct DeviceStateTrigger {
  pub target: Topic,
  pub field: String,
  pub op: Comparison,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Comparison {
  Equality { value: String },
  BoolComparison { pivot: bool },
}

#[derive(Debug, Clone)]
pub struct Effect {
  pub target: Topic,
  pub command: LightCommand,
}
