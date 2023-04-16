pub fn startup() {
  ctrlc::set_handler(|| {
    eprintln!("Detected shutdown. Initiating procedure.");
    shutdown();
  })
  .expect("Failed to set shutdown handler.");
}

pub fn shutdown() {
  // QUICK (!!) shutdown procedure, presumably little time left.
  // Persist temporary home configuration by renaming file.
  todo!()
}
