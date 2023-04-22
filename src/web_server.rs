use std::net::TcpListener;

use futures::never::Never;

use crate::Result;


#[derive(Debug)]
pub struct WebServer {
  listener: TcpListener,
}

impl WebServer {
  pub fn new() -> Self {
    let listener = TcpListener::bind("localhost:8088").unwrap();
    Self { listener }
  }
  pub fn run(self) -> Result<Never> {
    for stream in self.listener.incoming() {
        let _stream = stream.unwrap();
        println!("Connection established!");
    }
    unreachable!();
  }
}

impl Default for WebServer {
  fn default() -> Self {
    Self::new()
  }
}