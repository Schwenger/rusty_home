use std::rc::Rc;

use chrono::{Duration, Local, NaiveTime};
use futures::future::join_all;
use guard::guard;
use serde_json::Value as JsonValue;
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
  scenes::scene::*,
  Result,
};

#[derive(Debug)]
pub struct SceneManager {
  home: Rc<Mutex<Home>>,
  queue: UnboundedSender<Request>,
  receiver: UnboundedReceiver<SceneEvent>,
}

#[derive(Debug, Clone)]
pub enum SceneEvent {
  SensorUpdate(Topic, JsonValue),
  ManualTrigger(String),
}

impl SceneManager {
  pub fn new(
    home: Rc<Mutex<Home>>,
    queue: UnboundedSender<Request>,
    receiver: UnboundedReceiver<SceneEvent>,
  ) -> Self {
    Self { home, queue, receiver }
  }

  pub async fn run(mut self) -> Result<()> {
    loop {
      let event = self.receiver.recv().await.unwrap();
      let home = self.home.lock().await;
      join_all(home.scenes.iter().map(|scene| async {
        let se = SceneEvaluator { _home: &home, event: &event };
        se.eval_sensor_update(scene).await.into_iter().for_each(|r| self.queue.send(r).unwrap())
      }))
      .await;
    }
  }
}

struct SceneEvaluator<'a> {
  _home: &'a Home, // Will be required for time-triggered state-based constraints.
  event: &'a SceneEvent,
}

impl<'a> SceneEvaluator<'a> {
  pub async fn eval_sensor_update(self, scene: &Scene) -> Vec<Request> {
    let active = match self.event {
      SceneEvent::SensorUpdate(_, _) => self.evaluate_trigger(&scene.trigger),
      SceneEvent::ManualTrigger(ref name) => name == &scene.name,
    };
    if active {
      println!("Scene {} was triggered.", scene.name);
      return self.execute_effect(&scene.effect);
    }
    vec![]
  }

  fn evaluate_trigger(&self, trigger: &Trigger) -> bool {
    match trigger {
      Trigger::And(a, b) => self.evaluate_trigger(a.as_ref()) && self.evaluate_trigger(b.as_ref()),
      Trigger::DeviceState(dst) => self.evaluate_update_trigger(dst),
      Trigger::Time(TimeTrigger { from, duration }) => {
        Self::evaluate_time_trigger(*from, *duration, Local::now().time())
      }
      Trigger::ManualOnly => false,
    }
  }

  fn evaluate_time_trigger(from: NaiveTime, duration: Duration, now: NaiveTime) -> bool {
    if from < from + duration {
      from < now && now < from + duration
    } else {
      now > from || now < from + duration
    }
  }

  fn evaluate_update_trigger(&self, dst: &DeviceStateTrigger) -> bool {
    guard!(let SceneEvent::SensorUpdate(updated, state) = self.event else { return false });
    let DeviceStateTrigger { target, field, op } = dst;
    if target != updated {
      return false;
    }
    guard!(let Some(field) = state.get(field) else { return false });
    let field = field;
    match op {
      Comparison::BoolComparison { pivot } => field.as_bool().map(|c| c == *pivot).unwrap_or(false),
      Comparison::Equality { value } => &field.to_string() == value,
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

#[cfg(test)]
mod test {
  use chrono::{Duration, NaiveTime};

  use super::SceneEvaluator;

  #[test]
  fn test_time_trigger_eval() {
    let cases = vec![
      (8, 4, 10, true),   // morning: basic
      (8, 1, 10, false),  // morning: too late
      (8, 1, 7, false),   // morning: too early
      (16, 4, 18, true),  // afternoon: basic
      (16, 4, 22, false), // afternoon: too late
      (16, 4, 13, false), // afternoon: too early
      (10, 4, 13, true),  // morning to afternoon: basic
      (10, 4, 15, false), // morning to afternoon: too late
      (10, 4, 9, false),  // morning to afternoon: too early
      (22, 4, 1, true),   // afternoon to morning: basic
      (22, 4, 3, false),  // afternoon to morning: too late
      (22, 4, 21, false), // afternoon to morning: too early
    ];
    for case in cases {
      let (from, duration, now, res) = case;
      let from = NaiveTime::from_hms_opt(from, 0, 0).unwrap();
      let duration = Duration::hours(duration);
      let now = NaiveTime::from_hms_opt(now, 0, 0).unwrap();
      assert_eq!(SceneEvaluator::evaluate_time_trigger(from, duration, now), res);
    }
  }
}
