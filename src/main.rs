#![allow(clippy::new_without_default)]
#![allow(clippy::needless_range_loop)]

use std::net::{SocketAddr, TcpListener};

use hyper::Server as HyperServer;
use structopt::StructOpt;
use tower::make::Shared;
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;

use crate::errors::{ServerResult, Contextable};
use crate::logging::Logger;
use crate::opt::Opt;
use crate::server::Server;

mod logging;
mod opt;
mod server;
mod types;
mod utils;
mod constants;
mod errors;
mod ops;
mod metadata;
mod locks;

static LOGGER: Logger = Logger;

#[tokio::main]
async fn main() -> ServerResult<()> {
  let opts: Opt = Opt::from_args();
  opts.validate();
  log::set_max_level(opts.log_level);
  log::set_logger(&LOGGER)
    .expect("unable to initialize logging");

  let server = Server::new(opts.clone());
  server.recover()
    .await
    .with_context(|| "while recovering server state")?;

  let backgrounds = server.init()
    .await
    .with_context(|| "while initializing background processes")?;
  log::info!("initialized server background processes in dir {:?}", opts.dir);

  let filter = server.warp_filter();
  let warp_service = warp::service(filter);
  let tower_service = ServiceBuilder::new()
    .layer(AddExtensionLayer::new(server.clone()))
    .service(warp_service);
  let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], opts.port)))
    .expect("port busy");
  log::info!("bound TCP listener to port {}", opts.port);
  let hyper_future = HyperServer::from_tcp(listener)
    .unwrap()
    .serve(Shared::new(tower_service));
  log::info!("ready to serve requests");
  let outcomes = futures::future::join3(
    hyper_future,
    backgrounds.0,
    backgrounds.1,
  )
    .await;

  outcomes.0.expect("server crashed");
  Ok(())
}
