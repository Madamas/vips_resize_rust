#[macro_use]
extern crate lazy_static;

use futures::{future::FutureExt, pin_mut, select};
use std::sync::atomic::Ordering;
use tokio::signal::unix::{signal, SignalKind};

mod api;
mod config;
mod image;
mod telemetry;

use api::filters;
use api::APP_LIVE;
use config::Config;

async fn handle_exit() {
  let mut wait_sigint = signal(SignalKind::interrupt()).unwrap();
  let mut wait_sigterm = signal(SignalKind::terminate()).unwrap();

  let wait_sigint_bind = wait_sigint.recv().fuse();
  let wait_sigterm_bind = wait_sigterm.recv().fuse();

  pin_mut!(wait_sigint_bind, wait_sigterm_bind);
  select! {
      _option = wait_sigint_bind => {
          let cfg = Config::get_current();
          unsafe {
              APP_LIVE.store(false, Ordering::Relaxed);
          }
          println!("\nCaught interrupt sigint. Exiting in {}s", cfg.exec.timeout);
          std::thread::sleep(cfg.exec.timeout_duration);
          std::process::exit(0);
      },
      _option = wait_sigterm_bind => {
          let cfg = Config::get_current();
          unsafe {
              APP_LIVE.store(false, Ordering::Relaxed);
          }
          println!("\nCaught interrupt sigterm. Exiting in {}s", cfg.exec.timeout);
          std::thread::sleep(cfg.exec.timeout_duration);
          std::process::exit(0);
      }
  };
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  // reqwest client is Arc already so no need to manually make it
  let base_client = reqwest::Client::new();

  tokio::spawn(handle_exit());
  // tokio::spawn(telemetry::telemetry_export_server());

  let api = filters::service(base_client);
  let addr: std::net::SocketAddr = ([127, 0, 0, 1], 3000).into();
  let server = warp::serve(api).run(addr);

  println!("Application listening http://{}", addr);
  server.await;
  Ok(())
}
