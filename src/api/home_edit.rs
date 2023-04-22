use crate::api::Editable;

use super::ExecutorLogic;

#[derive(Debug, Clone)]
pub enum HomeEdit {
  AddRoom { name: String },
}

impl ExecutorLogic {
  pub(super) async fn edit_home(&mut self, edit: HomeEdit) {
    match edit {
      HomeEdit::AddRoom { name } => self.home.add_room(name),
    }
  }
}
