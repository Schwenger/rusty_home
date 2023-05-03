#![deny(
    // missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    rustdoc::broken_intra_doc_links,
)]

use config::GlobalConfig;
use controller::Controller;
use error::HomeBaseError;

pub mod api;
pub mod common;
pub mod config;
pub mod controller;
pub mod convert;
pub mod devices;
pub mod error;
pub mod home;
pub mod mqtt;
pub mod scene;
pub mod web_server;

type Error = HomeBaseError;
pub type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
  std::env::set_var("RUST_BACKTRACE", "1");
  let controller: Controller = GlobalConfig::read()?.try_into().await?;
  controller.run().await;
  Ok(())
}
