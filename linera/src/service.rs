#![cfg_attr(target_arch = "wasm32", no_main)]

mod lib;

use async_graphql::{EmptySubscription, Object, Schema};
use linera_sdk::{
    ServiceRuntime,
    Service
};
use linera_sdk::abi::WithServiceAbi;
use linera::ConwayBets;
use std::sync::Arc;
use linera_sdk::http::{Request, Response};

pub struct ConwayBetsAbi;

impl WithServiceAbi for ConwayBets {
    type Abi = ConwayBetsAbi;
}

linera_sdk::service!(ConwayBets);

impl Service for ConwayBets {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        // Load state for view
        linera::ConwayBets::default()
    }

    async fn handle_query(&self, _runtime: ServiceRuntime<Self>, query: Request) -> Response {   
        // Minimal GraphQL schema setup
        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
        schema.execute(query).await
    }
}

struct QueryRoot;
#[Object]
impl QueryRoot {
    async fn hello(&self) -> String {
        "Hello from ConwayBets".to_string()
    }
}

struct EmptyMutation;
#[Object]
impl EmptyMutation {}