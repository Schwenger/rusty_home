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
use futures::executor::block_on;

mod api;
mod config;
mod controller;
mod devices;
mod error;
mod frame;
mod home;
mod logic;
mod mqtt;

type Error = HomeBaseError;
pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
  let controller: Controller = GlobalConfig::read()?.try_into()?;
  controller.test_mode();
  block_on(async { controller.run().await });
  Ok(())
}
