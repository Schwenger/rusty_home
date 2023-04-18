use futures::executor::block_on;

use crate::mqtt::ProtectedClient;

pub fn startup(client: ProtectedClient) {
  ctrlc::set_handler(move || {
    eprintln!("Detected shutdown. Initiating procedure.");
    shutdown(client.clone());
  })
  .expect("Failed to set shutdown handler.");
}

pub fn shutdown(client: ProtectedClient) {
  // QUICK (!!) shutdown procedure, presumably little time left.
  // Persist temporary home configuration by renaming file.
  block_on(async { client.lock().await.disconnect().await });
  todo!()
}
