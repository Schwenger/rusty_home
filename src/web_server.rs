use guard::guard;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request as HyperRequest, Response, Server, StatusCode};
use local_ip_address::local_ip;
use serde_json::json;
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::Split;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use url::Url;

use crate::api::request::{LightCommand, Query, Request, SceneCommand};
use crate::api::topic::Topic;
use crate::convert::{Hue, RestApiPayload, Sat, Val};
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
    let addr = SocketAddr::new(local_ip().unwrap(), 8088);
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
    println!("\nReceived web request: {}", req.uri());
    if req.method() != Method::GET {
      println!("Bad request: Not a get-request.");
      return Ok(Self::bad_request("Only get requests are allowed."));
    }
    let url =
      Url::parse("http://localhost:8088").and_then(|b| b.join(&req.uri().to_string())).unwrap();
    let mut segments = url.path_segments().unwrap();
    let category = segments.next().unwrap();
    let response = match category {
      "query" => Self::handle_query(segments, &url, queue).await,
      "command" => Self::handle_command(segments, &url, queue).await,
      "scene" => Self::handle_scene(segments, &url, queue).await,
      "test" => Self::accepted("Test successful.".to_string()),
      _ => todo!(),
    };
    Ok(response)
  }

  async fn handle_scene(
    mut segments: Split<'_, char>,
    url: &Url,
    queue: UnboundedSender<Request>,
  ) -> Response<Body> {
    guard!(let Some(command) = segments.next() else { return Self::bad_request("Scene triggers need a command.") });
    if command != "TriggerScene" {
      return Self::bad_request("Unknown subcommand.");
    }
    let payload = Self::transform_query(url);
    guard!(let Some(name) = payload.name else { return Self::bad_request("Scene triggers need a name.") });
    queue.send(Request::SceneCommand(SceneCommand::Trigger(name))).unwrap();
    Self::accepted("Success".to_string())
  }

  async fn handle_command(
    mut segments: Split<'_, char>,
    url: &Url,
    queue: UnboundedSender<Request>,
  ) -> Response<Body> {
    guard!(let Some(command) = segments.next() else { return Self::bad_request("Command need a subcommand.") });
    let command = serde_json::from_value::<LightCommand>(json!(command));
    guard!(let Ok(command) = command else { return Self::bad_request("Unknown subcommand.") });
    let payload = Self::transform_query(url);
    queue.send(Request::LightCommand(command, payload)).unwrap();
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
        let payload = Self::transform_query(url);
        Request::Query(Query::DeviceState(payload.topic.unwrap()), sender)
      }
      Some("DeviceHistory") => {
        let payload = Self::transform_query(url);
        Request::Query(Query::DeviceHistory(payload.topic.unwrap()), sender)
      }
      None => return Self::bad_request("Queries need a subcommand."),
      Some(_) => return Self::bad_request("Unknown subcommand."),
    };
    queue.send(request).expect("Error handling");
    let resp = receiver.await.unwrap().to_str();
    if resp.len() > 50 {
      println!("Responding to query with: {}[truncated]", &resp[0..49]);
    } else {
      println!("Responding to query with: {}", &resp);
    }
    Self::accepted(resp)
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

  fn transform_query(url: &Url) -> RestApiPayload {
    let map: HashMap<Cow<'_, str>, Cow<'_, str>> = url.query_pairs().collect();
    let topic = map.get("topic").map(|b| Topic::try_from(b.to_string()).unwrap());
    let val = map.get("value").map(|b| b.parse::<f64>().unwrap()).map(Val::from_rest);
    let hue = map.get("hue").map(|b| b.parse().unwrap()).map(Hue::from_rest);
    let sat = map.get("saturation").map(|b| b.parse().unwrap()).map(Sat::from_rest);
    let name = map.get("name").map(|b| b.to_string());
    RestApiPayload { topic, val, hue, sat, name }
  }
}
