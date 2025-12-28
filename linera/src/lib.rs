use linera_sdk::base::Owner;
use linera_sdk::linera_base_types::{Amount, ApplicationId, ChainId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Market {
    pub id: MarketId,
    pub creator: Owner,
    pub title: String,
    pub description: String,
    pub end_time: u64, // Unix timestamp
    pub outcomes: Vec<String>,
    pub total_liquidity: Amount,
    pub is_resolved: bool,
    pub winning_outcome: Option<u32>,
    // Hash of the canonical state stored on the market's microchain
    pub state_hash: [u8; 32],
}

// A lightweight reference stored on user chains
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPosition {
    pub market_id: Market,
    pub outcome_index: u32,
    pub amount: Amount,
    pub state_hash: [u8; 32], // To verify against market chain
}

impl<Receipt: Bit> ConwayBets {
    // Operation to create a new market
    pub async fn create_market(
        &mut self,
        creator: Owner,
        title: String,
        description: String,
        end_time: u64,
        outcomes: Vec<String>,
    ) {
        let market_id = Market::new(self.context().chain_id());
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

        // Cross-chain message: Notify the new market's microchain to activate
        self.send_message(market_id.chain_id, MarketMessage::Initialize);
    }

    // Operation to place a bet
    pub async fn place_bet(
        &mut self,
        market_id: Market,
        user: Owner,
        outcome_index: u32,
        amount: Amount,
    ) -> Result<Receipt, dyn Error> {
        // 1. Verify market exists and is open
        let market = self.markets.get_mut(&market_id).ok_or(<dyn Error>::MarketNotFound)?;

        // 2. Lock funds from the user's chain (user chain operation)
        self.lock_funds(user, amount).await?;

        // 3. Create a cross-chain message to the market's microchain
        let bet_message = BetMessage {
            market_id,
            user,
            outcome_index,
            amount,
        };
        self.send_message(market_id.chain_id, ConwayBetsMessage::Bet(bet_message));

        // 4. Update local user portfolio reference
        let position = UserPosition {
            market_id,
            outcome_index,
            amount,
            state_hash: market.state_hash,
        };
        self.user_positions.entry(user).or_insert(Vec::new()).push(position);

        Ok(Receipt::new(bet_id, Status::Finalized))
    }
}