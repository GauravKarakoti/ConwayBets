pub mod state;
pub use state::*;

use linera_sdk::abi::{ContractAbi, ServiceAbi};
use async_graphql::{Request, Response};

// --- ABI Definition ---

pub struct ConwayBetsAbi;

impl ContractAbi for ConwayBetsAbi {
    type Operation = Operation;
    type Response = (); 
}

impl ServiceAbi for ConwayBetsAbi {
    type Query = Request;
    type QueryResponse = Response;
}