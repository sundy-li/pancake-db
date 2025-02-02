use std::convert::Infallible;

use hyper::body::Bytes;
use pancake_db_idl::ddl::{GetSchemaRequest, GetSchemaResponse};
use warp::{Filter, Rejection, Reply};

use crate::errors::ServerResult;

use crate::server::Server;
use crate::utils::common;
use crate::ops::get_schema::GetSchemaOp;
use crate::ops::traits::ServerOp;

const ROUTE_NAME: &str = "get_schema";

impl Server {
  pub async fn get_schema(&self, req: GetSchemaRequest) -> ServerResult<GetSchemaResponse> {
    GetSchemaOp { req }.execute(self).await
  }

  pub fn get_schema_filter() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::get()
      .and(warp::path(ROUTE_NAME))
      .and(warp::filters::ext::get::<Server>())
      .and(warp::filters::body::bytes())
      .and_then(Self::get_schema_from_body)
  }

  async fn get_schema_from_bytes(&self, body: Bytes) -> ServerResult<GetSchemaResponse> {
    let req = common::parse_pb::<GetSchemaRequest>(body)?;
    self.get_schema(req).await
  }

  async fn get_schema_from_body(server: Server, body: Bytes) -> Result<impl Reply, Infallible> {
    Self::log_request(ROUTE_NAME, &body);
    common::pancake_result_into_warp(
      server.get_schema_from_bytes(body).await,
      ROUTE_NAME,
    )
  }
}

