use super::ExecutorLogic;

#[derive(Debug, Clone, Copy)]
pub enum LightCommand {
}

impl ExecutorLogic {
  pub(super) async fn execute_light(&mut self, cmd: LightCommand) {
    todo!()
  }
}