use crate::api::traits::EditableHome;

use super::{executor::ExecutorLogic, request::HomeEdit};

impl ExecutorLogic {
  pub(super) async fn edit_home(&mut self, edit: HomeEdit) {
    match edit {
      HomeEdit::AddRoom { name } => self.home.lock().await.add_room(name),
    }
  }
}
