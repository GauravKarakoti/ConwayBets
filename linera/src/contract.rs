use linera::{ConwayBets, ConwayBetsAbi, ConwayBetsMessage, Operation};
use linera_sdk::{
    abi::WithContractAbi,
    Contract, ContractRuntime,
};
use serde::{Deserialize, Serialize};
use linera_sdk::bcs;
use linera_sdk::views::linera_views::store::ReadableKeyValueStore;
use linera_sdk::views::linera_views::store::WritableKeyValueStore;

pub struct ConwayBetsContract {
    state: ConwayBets,
    runtime: ContractRuntime<Self>, // Added runtime field
}

impl WithContractAbi for ConwayBetsContract {
    type Abi = ConwayBetsAbi;
}

linera_sdk::contract!(ConwayBetsContract);

#[derive(Deserialize, Serialize, Debug)]
pub struct InstantiationArgument {
    pub initial_markets: Vec<String>,
}

const STATE_KEY: &[u8] = b"conway_bets_state"; // Define a key for storage

impl Contract for ConwayBetsContract {
    type Message = ConwayBetsMessage;
    type InstantiationArgument = InstantiationArgument;
    type Parameters = ();
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        // Fix: Read bytes from the key-value store and deserialize
        let state = match runtime.key_value_store().read_value(STATE_KEY).await {     
            Ok(Some(bytes)) => bcs::from_bytes(&bytes).unwrap_or_default(),
            _ => ConwayBets::default(),
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
        // Fix: Serialize and write the state back to storage
        let bytes = bcs::to_bytes(&self.state).expect("Failed to serialize state");
        self.runtime.key_value_store().write_batch(vec![
            (STATE_KEY.to_vec(), linera_sdk::linera_base_types::BatchOperation::Put(bytes))
        ]).await.expect("Failed to store state");
    }
}