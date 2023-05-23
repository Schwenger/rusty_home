use crate::scenes::manager::SceneEvent;

use super::{executor::ExecutorLogic, request::SceneCommand};

impl ExecutorLogic {
  pub(super) async fn execute_scene(&mut self, cmd: SceneCommand) {
    match cmd {
      SceneCommand::Trigger(name) => {
        self.scene_events.send(SceneEvent::ManualTrigger(name)).unwrap();
      }
    }
  }
}
