#![cfg_attr(target_arch = "wasm32", no_main)]

use linera::{ConwayBets, ConwayBetsAbi, ConwayBetsMessage, Operation};
use linera_sdk::{
    abi::WithContractAbi,
    Contract, ContractRuntime,
};
use serde::{Deserialize, Serialize};
// FIX: Import Batch and WriteOperation from linera_views
use linera_sdk::views::linera_views::batch::{Batch, WriteOperation};
use linera_sdk::views::linera_views::store::WritableKeyValueStore;
use linera_sdk::views::linera_views::store::ReadableKeyValueStore; 

pub struct ConwayBetsContract {
    state: ConwayBets,
    runtime: ContractRuntime<Self>,
}

impl WithContractAbi for ConwayBetsContract {
    type Abi = ConwayBetsAbi;
}

linera_sdk::contract!(ConwayBetsContract);

#[derive(Deserialize, Serialize, Debug)]
pub struct InstantiationArgument {
    pub initial_markets: Vec<String>,
}

const STATE_KEY: &[u8] = b"conway_bets_state";

impl Contract for ConwayBetsContract {
    type Message = ConwayBetsMessage;
    type InstantiationArgument = InstantiationArgument;
    type Parameters = ();
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        // FIX: Read bytes and deserialize them
        let state_bytes = runtime.key_value_store()
            .read_value_bytes(STATE_KEY)
            .await
            .expect("Failed to read state");
            
        let state: ConwayBets = match state_bytes {
            Some(bytes) => bcs::from_bytes(&bytes).expect("Failed to deserialize state"),
            None => ConwayBets::default(),
        };
            
        ConwayBetsContract { state, runtime }
    }

    async fn instantiate(
        &mut self,
        _argument: Self::InstantiationArgument,
    ) {
        // Initialization logic
    }

    async fn execute_operation(
        &mut self,
        operation: Operation,
    ) -> Self::Response {
        match operation {
            Operation::CreateMarket { creator, title, description, end_time, outcomes } => {
                self.state.create_market(creator, title, description, end_time, outcomes).await;
            }
            Operation::PlaceBet { market_id, user, outcome_index, amount } => {
                let _ = self.state.place_bet(market_id, user, outcome_index, amount).await;
            }
        }
    }

    async fn execute_message(
        &mut self,
        _message: Self::Message,
    ) {
        // Handle cross-chain messages
    }

    async fn store(self) {
        let bytes = bcs::to_bytes(&self.state).expect("Failed to serialize state");
        
        // FIX: Use the correct Batch structure from linera_views
        let batch = Batch {
            operations: vec![
                WriteOperation::Put {
                    key: STATE_KEY.to_vec(),
                    value: bytes,
                }
            ],
        };

        self.runtime.key_value_store().write_batch(batch).await.expect("Failed to store state");
    }
}