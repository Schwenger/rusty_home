use chrono::{Duration, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::api::{request::LightCommand, topic::Topic};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
  pub trigger: Trigger,
  pub effect: Effect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trigger {
  DeviceState(DeviceStateTrigger),
  And(Box<Trigger>, Box<Trigger>),
  Time(TimeTrigger),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStateTrigger {
  pub target: Topic,
  pub field: String,
  pub op: Comparison,
}
#[serde_with::serde_as]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimeTrigger {
  pub from: NaiveTime,
  #[serde_as(as = "serde_with::DurationSeconds<i64>")]
  pub duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Comparison {
  Equality { value: String },
  BoolComparison { pivot: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
  pub target: Topic,
  pub command: LightCommand,
}
