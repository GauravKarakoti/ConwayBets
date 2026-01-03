#![cfg_attr(target_arch = "wasm32", no_main)] // <--- ADD THIS LINE

use async_graphql::{EmptyMutation, EmptySubscription, Object, Request, Response, Schema, SimpleObject};
use linera::{ConwayBets, Market};
use linera_sdk::{
    abi::WithServiceAbi,
    Service, ServiceRuntime,
};
use std::sync::Arc;
use linera_sdk::views::linera_views::store::ReadableKeyValueStore;

pub struct ConwayBetsService {
    state: Arc<ConwayBets>,
}

impl WithServiceAbi for ConwayBetsService {
    type Abi = linera::ConwayBetsAbi;
}

linera_sdk::service!(ConwayBetsService);

const STATE_KEY: &[u8] = b"conway_bets_state";

impl Service for ConwayBetsService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = runtime.key_value_store()
            .read_value_bytes::<>(STATE_KEY)
            .await
            .expect("Failed to read state")
            .unwrap_or_default();
            
        ConwayBetsService {
            state: Arc::new(state),
        }
    }

    async fn handle_query(&self, query: Request) -> Response {
        let schema = Schema::build(
            QueryRoot { state: self.state.clone() }, 
            EmptyMutation, 
            EmptySubscription
        ).finish();
        schema.execute(query).await
    }
}

struct QueryRoot {
    state: Arc<ConwayBets>,
}

#[Object]
impl QueryRoot {
    async fn markets(&self, limit: Option<usize>, offset: Option<usize>) -> Vec<MarketGql> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        
        self.state.markets.values()
            .skip(offset)
            .take(limit)
            .map(|m| MarketGql::from(m))
            .collect()
    }

    async fn market(&self, id: String) -> Option<MarketGql> {
        self.state.markets.values()
            .find(|m| m.id.id.to_string() == id) 
            .map(|m| MarketGql::from(m))
    }
}

#[derive(SimpleObject)]
struct MarketGql {
    id: String,
    title: String,
    description: String,
    creator: String,
    end_time: u64,
    outcomes: Vec<String>,
    total_liquidity: String,
    is_resolved: bool,
    winning_outcome: Option<u32>,
    state_hash: String,
    created_at: u64,
}

impl From<&Market> for MarketGql {
    fn from(m: &Market) -> Self {
        MarketGql {
            id: m.id.id.to_string(),
            title: m.title.clone(),
            description: m.description.clone(),
            creator: m.creator.to_string(),
            end_time: m.end_time,
            outcomes: m.outcomes.clone(),
            total_liquidity: m.total_liquidity.to_string(),
            is_resolved: m.is_resolved,
            winning_outcome: m.winning_outcome,
            state_hash: m.state_hash.iter().map(|b| format!("{:02x}", b)).collect(),
            created_at: 0,
        }
    }
}