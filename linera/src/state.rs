use linera_sdk::linera_base_types::{AccountOwner, Amount, ChainId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;

// --- Definitions ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MarketId {
    pub chain_id: ChainId,
    pub id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Operation {
    CreateMarket {
        creator: AccountOwner,
        title: String,
        description: String,
        end_time: u64,
        outcomes: Vec<String>,
    },
    PlaceBet {
        market_id: MarketId,
        user: AccountOwner,
        outcome_index: u32,
        amount: Amount,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BetMessage {
    pub market_id: MarketId,
    pub user: AccountOwner,
    pub outcome_index: u32,
    pub amount: Amount,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ConwayBetsMessage {
    Initialize,
    Bet(BetMessage),
    SyncState {
        market_id: MarketId,
        state_hash: [u8; 32],
        block_height: u64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Status {
    Finalized,
    Pending,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub id: u64,
    pub status: Status,
}

impl Receipt {
    pub fn new(id: u64, status: Status) -> Self {
        Self { id, status }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct ConwayBets {
    pub markets: BTreeMap<MarketId, Market>,
    pub user_positions: BTreeMap<AccountOwner, Vec<UserPosition>>,
    #[serde(skip)] 
    pub next_market_id: u64,
    #[serde(skip)]
    pub next_bet_id: u64,
}

// --------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Market {
    pub id: MarketId,
    pub creator: AccountOwner,
    pub title: String,
    pub description: String,
    pub end_time: u64, // Unix timestamp
    pub outcomes: Vec<String>,
    pub total_liquidity: Amount,
    pub is_resolved: bool,
    pub winning_outcome: Option<u32>,
    pub state_hash: [u8; 32],
}

impl Market {
    pub fn new(chain_id: ChainId) -> MarketId {
        MarketId { chain_id, id: 0 } 
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPosition {
    pub market_id: MarketId,
    pub outcome_index: u32,
    pub amount: Amount,
    pub state_hash: [u8; 32],
}

impl ConwayBets {
    // Helper to access chain_id
    fn context(&self) -> ContextStub {
        ContextStub { chain_id: ChainId([0; 4].into()) }
    }

    // Helper to send messages
    fn send_message(&self, _dest: ChainId, _msg: ConwayBetsMessage) {
        // Placeholder
    }

    // Helper to lock funds
    async fn lock_funds(&self, _user: AccountOwner, _amount: Amount) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    // Helper to init state
    async fn initialize_market_state(&self, _market_id: &MarketId) -> [u8; 32] {
        [0; 32]
    }

    pub async fn create_market(
        &mut self,
        creator: AccountOwner,
        title: String,
        description: String,
        end_time: u64,
        outcomes: Vec<String>,
    ) {
        self.next_market_id += 1;
        let market_id = MarketId { 
            chain_id: self.context().chain_id, 
            id: self.next_market_id 
        };
        
        let state_hash = self.initialize_market_state(&market_id).await;

        let market = Market {
            id: market_id,
            creator,
            title,
            description,
            end_time,
            outcomes,
            total_liquidity: Amount::from(0),
            is_resolved: false,
            winning_outcome: None,
            state_hash,
        };

        self.markets.insert(market_id, market);
        self.send_message(market_id.chain_id, ConwayBetsMessage::Initialize);
    }

    pub async fn place_bet(
        &mut self,
        market_id: MarketId,
        user: AccountOwner,
        outcome_index: u32,
        amount: Amount,
    ) -> Result<Receipt, Box<dyn Error>> {
        let state_hash = self.markets.get(&market_id)
            .ok_or("MarketNotFound")?
            .state_hash;

        self.lock_funds(user, amount).await?;

        let bet_message = BetMessage {
            market_id,
            user,
            outcome_index,
            amount,
        };
        self.send_message(market_id.chain_id, ConwayBetsMessage::Bet(bet_message));

        let position = UserPosition {
            market_id,
            outcome_index,
            amount,
            state_hash,
        };
        self.user_positions.entry(user).or_insert(Vec::new()).push(position);

        self.next_bet_id += 1;
        Ok(Receipt::new(self.next_bet_id, Status::Finalized))
    }
}

struct ContextStub {
    chain_id: ChainId,
}
impl ContextStub {
    fn chain_id(&self) -> ChainId {
        self.chain_id
    }
}