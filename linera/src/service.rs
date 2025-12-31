#![cfg_attr(target_arch = "wasm32", no_main)]

mod state; // Changed from mod lib;

use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema}; // Imported EmptyMutation
use linera_sdk::{
    ServiceRuntime,
    Service
};
use linera_sdk::abi::WithServiceAbi;
use state::ConwayBets; // Changed from linera::ConwayBets
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
        state::ConwayBets::default() // Changed to state::ConwayBets
    }

    async fn handle_query(&self, query: Request) -> Response {   
        // Use the imported EmptyMutation from async_graphql
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