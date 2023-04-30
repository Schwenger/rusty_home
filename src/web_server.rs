use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request as HyperRequest, Response, Server, StatusCode};
use serde_json::json;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::Split;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use url::Url;

use crate::api::request::{LightCommand, Query, Request};
use crate::api::topic::Topic;
use crate::Result;

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
      async move { Ok::<_, Infallible>(service_fn(move |req| Self::process(req, clone.clone()))) }
    });
    let server = Server::bind(&addr).serve(make_svc);
    println!("Server started.");
    if let Err(e) = server.await {
      eprintln!("server error: {}", e);
    }
    unreachable!()
  }

  async fn process(
    req: HyperRequest<Body>,
    queue: UnboundedSender<Request>,
  ) -> std::result::Result<Response<Body>, Infallible> {
    println!("\nProcessing request: {}", req.uri());
    if req.method() != Method::GET {
      println!("Bad request: Not a get-request.");
      return Ok(Self::bad_request("Only get requests are allowed."));
    }
    let url =
      Url::parse("http://localhost:8088").and_then(|b| b.join(&req.uri().to_string())).unwrap();
    let mut segments = url.path_segments().unwrap();
    let category = segments.next().unwrap();
    println!("Received a {} request.", category);
    let response = match category {
      "query" => Self::handle_query(segments, &url, queue).await,
      "command" => Self::handle_command(segments, &url, queue).await,
      "test" => Self::accepted("Test successful.".to_string()),
      _ => todo!(),
    };
    Ok(response)
  }

  async fn handle_command(
    mut segments: Split<'_, char>,
    url: &Url,
    queue: UnboundedSender<Request>,
  ) -> Response<Body> {
    let command = segments.next();
    if command.is_none() {
      return Self::bad_request("Command need a subcommand.");
    }
    let command = json!(command.unwrap());
    let command = serde_json::from_value::<LightCommand>(command);
    if command.is_err() {
      return Self::bad_request("Unknown subcommand.");
    }
    let command = command.unwrap();
    let target = Self::find_topic(url).expect("Error handling.");
    queue.send(Request::LightCommand(command, target)).unwrap();
    Self::accepted("Success".to_string())
  }

  async fn handle_query(
    mut segments: Split<'_, char>,
    url: &Url,
    queue: UnboundedSender<Request>,
  ) -> Response<Body> {
    let (sender, receiver) = oneshot::channel();
    let request = match segments.next() {
      Some("Structure") => Request::Query(Query::Architecture, sender),
      Some("DeviceState") => {
        let target = Self::find_topic(url).expect("Error handling.");
        Request::Query(Query::DeviceState(target), sender)
      }
      None => return Self::bad_request("Queries need a subcommand."),
      Some(_) => return Self::bad_request("Unknown subcommand."),
    };
    queue.send(request).expect("Error handling");
    let resp = receiver.await.unwrap();
    Self::accepted(resp.to_str())
  }

  fn bad_request(msg: &'static str) -> Response<Body> {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::BAD_REQUEST;
    *response.body_mut() = Body::from(msg);
    response
  }

  fn accepted(msg: String) -> Response<Body> {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::ACCEPTED;
    *response.body_mut() = Body::from(msg);
    response
  }

  fn find_topic(url: &Url) -> Option<Topic> {
    url
      .query_pairs()
      .filter_map(|(k, v)| if k == "topic" { Some(v.to_string()) } else { None })
      .map(Topic::try_from)
      .find_map(Result::ok)
  }
}
