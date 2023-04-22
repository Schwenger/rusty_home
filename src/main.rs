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

use common::Scalar;
use config::GlobalConfig;
use controller::Controller;
use error::HomeBaseError;
use futures::executor::block_on;

pub mod api;
pub mod common;
pub mod config;
pub mod controller;
pub mod devices;
pub mod error;
pub mod home;
pub mod mqtt;
pub mod web_server;

type Error = HomeBaseError;
pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
  let controller: Controller = GlobalConfig::read()?.try_into()?;
  block_on(async { controller.run().await });
  Ok(())
}
