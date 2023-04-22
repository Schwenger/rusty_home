
use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request as HyperRequest, Response, Server, Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use crate::Result;
use crate::api::{Request, Query};


#[derive(Debug)]
pub struct WebServer {
  queue: UnboundedSender<Request>,
}

impl WebServer {
  pub fn new(queue: UnboundedSender<Request>) -> Self {
    println!("WebServer created.");
    Self { queue }
  }

  pub async fn run(self) -> Result<Infallible> {
    println!("Running web server.");
    let WebServer { queue } = self;
    let addr = SocketAddr::from(([127, 0, 0, 1], 8088));
    let make_svc = make_service_fn(move |_conn| {
      let clone = queue.clone();
      async move {
        Ok::<_, Infallible>(service_fn(move |req| Self::process(req, clone.clone())))
      }
    });
    let server = Server::bind(&addr).serve(make_svc);
    println!("Server started.");
    if let Err(e) = server.await {
      eprintln!("server error: {}", e);
    }
    unreachable!()
  }

  async fn process(req: HyperRequest<Body>, queue: UnboundedSender<Request>) -> std::result::Result<Response<Body>, Infallible> {
    println!("Processing request");
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
      (&Method::GET, "/") => *response.body_mut() = Body::from("Test successful."),
      (&Method::GET, "/architecture") => {
        let (sender, receiver) = oneshot::channel();
        queue.send(Request::Query(Query::Architecture, sender)).unwrap();
        let resp = receiver.await.unwrap();
        *response.body_mut() = Body::from(resp.to_str());
      },
      (&Method::POST, _) => panic!("Received post request"),
      _ => {
          *response.status_mut() = StatusCode::NOT_FOUND;
          *response.body_mut() = Body::from("Unknown page.");
      },
    };
    Ok(response)
  }

}