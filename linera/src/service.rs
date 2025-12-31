#![cfg_attr(target_arch = "wasm32", no_main)]

use async_graphql::{EmptyMutation, EmptySubscription, Object, Request, Response, Schema};
use linera::{ConwayBets, ConwayBetsAbi};
use linera_sdk::{
    abi::WithServiceAbi,
    Service, ServiceRuntime,
};

pub struct ConwayBetsService {
    state: ConwayBets,
}

impl WithServiceAbi for ConwayBetsService {
    type Abi = ConwayBetsAbi;
}

linera_sdk::service!(ConwayBetsService);

impl Service for ConwayBetsService {
    type Parameters = ();

    async fn new(_runtime: ServiceRuntime<Self>) -> Self {
        ConwayBetsService {
            state: ConwayBets::default(),
        }
    }

    async fn handle_query(&self, query: Request) -> Response {   
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