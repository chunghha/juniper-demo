extern crate dotenv;
#[macro_use]
extern crate dotenv_codegen;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use dotenv::dotenv;
use juniper::http::graphiql::graphiql_source;
use std::sync::Arc;
use warp::Filter;

mod db;
mod graphql;

#[tokio::main]
async fn main() {
  dotenv().ok();
  pretty_env_logger::init();

  let schema = Arc::new(graphql::Schema::new(graphql::QueryRoot, graphql::MutationRoot));
  // Create a warp filter for the schema
  let schema = warp::any().map(move || Arc::clone(&schema));

  let db_client = db::get_db_client().await;

  let ctx = Arc::new(graphql::Context { db_client });
  // Create a warp filter for the context
  let ctx = warp::any().map(move || Arc::clone(&ctx));

  let graphql_route = warp::post()
    .and(warp::path!("graphql"))
    .and(schema.clone())
    .and(ctx.clone())
    .and(warp::body::json())
    .and_then(graphql::graphql_resolve);

  let graphiql_route = warp::get().and(warp::path!("graphiql")).map(|| warp::reply::html(graphiql_source("graphql")));

  let routes = graphql_route.or(graphiql_route);

  warp::serve(routes).run(([0, 0, 0, 0], 6060)).await;
}
