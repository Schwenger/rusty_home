use std::rc::Rc;

use chrono::{Duration, Local, NaiveTime};
use futures::future::join_all;
use guard::guard;
use serde::{Deserialize, Serialize};
use tokio::sync::{
  mpsc::{UnboundedReceiver, UnboundedSender},
  Mutex,
};

use crate::{
  api::{
    request::{LightCommand, Request},
    topic::Topic,
  },
  convert::RestApiPayload,
  home::Home,
  Result,
};

use serde_json::Value as JsonValue;

#[derive(Debug)]
pub struct SceneManager {
  home: Rc<Mutex<Home>>,
  queue: UnboundedSender<Request>,
  receiver: UnboundedReceiver<(Topic, JsonValue)>,
}

impl SceneManager {
  pub fn new(
    home: Rc<Mutex<Home>>,
    queue: UnboundedSender<Request>,
    receiver: UnboundedReceiver<(Topic, JsonValue)>,
  ) -> Self {
    Self { home, queue, receiver }
  }

  pub async fn run(mut self) -> Result<()> {
    loop {
      let update = self.receiver.recv().await.unwrap();
      let home = self.home.lock().await;
      join_all(home.scenes.iter().map(|scene| async {
        let se = SceneEvaluator { _home: &home, update: Some(&update) };
        se.eval_update(scene).await.into_iter().for_each(|r| self.queue.send(r).unwrap())
      }))
      .await;
    }
  }
}

struct SceneEvaluator<'a> {
  _home: &'a Home, // Will be required for time-triggered state-based constraints.
  update: Option<&'a (Topic, JsonValue)>,
}

impl<'a> SceneEvaluator<'a> {
  pub async fn eval_update(self, scene: &Scene) -> Vec<Request> {
    if self.evaluate_trigger(&scene.trigger) {
      println!("Scene {} was triggered.", scene.name);
      return self.execute_effect(&scene.effect);
    }
    println!("Scene {} was not triggered.", scene.name);
    vec![]
  }

  fn evaluate_trigger(&self, trigger: &Trigger) -> bool {
    match trigger {
      Trigger::And(a, b) => self.evaluate_trigger(a.as_ref()) && self.evaluate_trigger(b.as_ref()),
      Trigger::DeviceState(dst) => self.evaluate_update_trigger(dst),
      Trigger::Time(TimeTrigger { from, duration }) => self.evaluate_time_trigger(*from, *duration),
    }
  }

  fn evaluate_time_trigger(&self, from: NaiveTime, duration: Duration) -> bool {
    let now = Local::now().time();
    let res = if from < from + duration {
      from < now && now < from + duration
    } else {
      from < now && now > from + duration
    };
    println!("Evaluating timed trigger with result {res}.");
    res
  }

  fn evaluate_update_trigger(&self, dst: &DeviceStateTrigger) -> bool {
    guard!(let Some((upd_topic, state)) = self.update else { return false });
    println!("Evaluating update trigger.");
    let DeviceStateTrigger { target, field, op } = dst;
    if target != upd_topic {
      println!("Wrong target. {} versus {}", target.to_str(), upd_topic.to_str());
      return false;
    }
    println!("Found proper target.");
    let field = state.get(field);
    if field.is_none() {
      println!("Field not present.");
      return false;
    }
    println!("Found field with value {}.", field.unwrap());
    let field = field.unwrap();
    match op {
      Comparison::BoolComparison { pivot } => field.as_bool().map(|c| c == *pivot).unwrap_or(false),
      Comparison::Equality { value } => {
        println!("Comparing {value} against {}.", field.to_string());
        &field.to_string() == value
      }
    }
  }

  fn execute_effect(&self, effect: &Effect) -> Vec<Request> {
    let Effect { ref target, command } = *effect;
    let mut payload = RestApiPayload { topic: Some(target.clone()), ..RestApiPayload::default() };
    payload.topic = Some(target.clone());
    assert_ne!(command, LightCommand::ChangeState);
    vec![Request::LightCommand(command, payload)]
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
  pub name: String,
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
