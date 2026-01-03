use linera::{ConwayBets, ConwayBetsAbi, ConwayBetsMessage, Operation};
use linera_sdk::{
    abi::WithContractAbi,
    Contract, ContractRuntime,
};
use serde::{Deserialize, Serialize};
// Import Batch and WriteOperation correctly
use linera_sdk::views::linera_views::batch::{Batch, WriteOperation}; 
use linera_sdk::views::linera_views::store::{ReadableKeyValueStore, WritableKeyValueStore};

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
        // Fix: Read and deserialize ConwayBets directly.
        // read_value returns Result<Option<T>, ...>
        let state = runtime.key_value_store()
            .read_value::<ConwayBets>(STATE_KEY)
            .await
            .expect("Failed to read state")
            .unwrap_or_default();
            
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
        // Fix: Use correct Batch and WriteOperation types
        let bytes = bcs::to_bytes(&self.state).expect("Failed to serialize state");
        
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