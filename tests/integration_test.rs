use conwaybets::*;
use linera_sdk::{
    base::{Amount, ApplicationId, Owner, ChainId},
    test::{TestValidator, TestChain},
};
use std::sync::Arc;

#[tokio::test]
async fn test_market_creation() {
    // Setup validator with Conway testnet config
    let validator = TestValidator::with_current_module::<ConwayBetsAbi>().await;
    let mut chain = validator.new_chain().await;
    
    // Deploy application
    let app_id = chain
        .create_application::<ConwayBetsAbi>((), (), vec![])
        .await;
    
    // Test market creation
    let creator = Owner::from([1u8; 32]);
    let market_data = MarketCreationData {
        title: "Test Market".to_string(),
        description: "Integration test market".to_string(),
        end_time: 1_000_000_000,
        outcomes: vec!["Yes".to_string(), "No".to_string()],
    };
    
    let market_id = chain
        .call_application::<ConwayBetsAbi, _>(app_id, "create_market", &(creator, market_data))
        .await
        .unwrap();
    
    assert!(market_id.is_valid());
    
    // Verify market exists
    let markets = chain
        .query_application::<ConwayBetsAbi, _>(app_id, "get_all_markets", &())
        .await
        .unwrap();
    
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].title, "Test Market");
}

#[tokio::test]
async fn test_bet_placement() {
    let validator = TestValidator::with_current_module::<ConwayBetsAbi>().await;
    let mut chain = validator.new_chain().await;
    
    let app_id = chain
        .create_application::<ConwayBetsAbi>((), (), vec![])
        .await;
    
    // Create market first
    let creator = Owner::from([1u8; 32]);
    let market_data = MarketCreationData {
        title: "Bet Test Market".to_string(),
        description: "For bet testing".to_string(),
        end_time: 1_000_000_000,
        outcomes: vec!["Team A".to_string(), "Team B".to_string()],
    };
    
    let market_id = chain
        .call_application::<ConwayBetsAbi, _>(app_id, "create_market", &(creator, market_data))
        .await
        .unwrap();
    
    // Place a bet
    let bettor = Owner::from([2u8; 32]);
    let bet_data = BetData {
        market_id,
        outcome_index: 0,
        amount: Amount::from(100), // 100 tokens
    };
    
    let receipt = chain
        .call_application::<ConwayBetsAbi, _>(app_id, "place_bet", &(bettor, bet_data))
        .await
        .unwrap();
    
    assert!(receipt.is_finalized());
    
    // Verify bet is recorded
    let user_bets = chain
        .query_application::<ConwayBetsAbi, _>(app_id, "get_user_bets", &bettor)
        .await
        .unwrap();
    
    assert_eq!(user_bets.len(), 1);
    assert_eq!(user_bets[0].amount, Amount::from(100));
}

#[tokio::test]
async fn test_cross_chain_bet_synchronization() {
    // Test betting across user and market microchains
    let validator = TestValidator::with_current_module::<ConwayBetsAbi>().await;
    
    // Create two chains: user chain and market chain
    let mut user_chain = validator.new_chain().await;
    let mut market_chain = validator.new_chain().await;
    
    // Deploy application on both chains
    let app_id = user_chain
        .create_application::<ConwayBetsAbi>((), (), vec![])
        .await;
    
    // Create market on market chain
    let creator = Owner::from([1u8; 32]);
    let market_data = MarketCreationData {
        title: "Cross-chain Market".to_string(),
        description: "Testing cross-chain bets".to_string(),
        end_time: 1_000_000_000,
        outcomes: vec!["Yes".to_string(), "No".to_string()],
    };
    
    let market_id = market_chain
        .call_application::<ConwayBetsAbi, _>(app_id, "create_market", &(creator, market_data))
        .await
        .unwrap();
    
    // Simulate cross-chain message for bet placement
    let bet_message = ConwayBetsMessage::Bet {
        market_id,
        user: Owner::from([2u8; 32]),
        outcome_index: 1,
        amount: Amount::from(50),
    };
    
    // Send message from user chain to market chain
    user_chain
        .send_message(market_chain.id(), bet_message.clone())
        .await
        .unwrap();
    
    // Process the message on market chain
    market_chain.handle_received_messages().await;
    
    // Verify bet was processed on market chain
    let market_state = market_chain
        .query_application::<ConwayBetsAbi, _>(app_id, "get_market_state", &market_id)
        .await
        .unwrap();
    
    assert!(market_state.total_liquidity > Amount::from(0));
}

#[tokio::test]
async fn test_market_resolution() {
    let validator = TestValidator::with_current_module::<ConwayBetsAbi>().await;
    let mut chain = validator.new_chain().await;
    
    let app_id = chain
        .create_application::<ConwayBetsAbi>((), (), vec![])
        .await;
    
    // Create and fund a market
    let creator = Owner::from([1u8; 32]);
    let market_data = MarketCreationData {
        title: "Resolution Test".to_string(),
        description: "Testing market resolution".to_string(),
        end_time: 1_000_000_000,
        outcomes: vec!["Win".to_string(), "Lose".to_string()],
    };
    
    let market_id = chain
        .call_application::<ConwayBetsAbi, _>(app_id, "create_market", &(creator, market_data))
        .await
        .unwrap();
    
    // Place some bets
    let bet_data = BetData {
        market_id,
        outcome_index: 0,
        amount: Amount::from(100),
    };
    
    chain
        .call_application::<ConwayBetsAbi, _>(app_id, "place_bet", &(Owner::from([2u8; 32]), bet_data))
        .await
        .unwrap();
    
    // Resolve the market
    let resolution = ResolutionData {
        market_id,
        winning_outcome: 0,
        resolution_proof: vec![], // Empty for test
    };
    
    let success = chain
        .call_application::<ConwayBetsAbi, _>(app_id, "resolve_market", &(creator, resolution))
        .await
        .unwrap();
    
    assert!(success);
    
    // Verify market is resolved
    let market = chain
        .query_application::<ConwayBetsAbi, _>(app_id, "get_market", &market_id)
        .await
        .unwrap();
    
    assert!(market.is_resolved);
    assert_eq!(market.winning_outcome, Some(0));
}

#[tokio::test]
async fn test_concurrent_bet_placement() {
    // Test handling multiple simultaneous bets
    let validator = TestValidator::with_current_module::<ConwayBetsAbi>().await;
    let mut chain = validator.new_chain().await;
    
    let app_id = chain
        .create_application::<ConwayBetsAbi>((), (), vec![])
        .await;
    
    // Create market
    let creator = Owner::from([1u8; 32]);
    let market_data = MarketCreationData {
        title: "Concurrency Test".to_string(),
        description: "Testing concurrent bets".to_string(),
        end_time: 1_000_000_000,
        outcomes: vec!["A".to_string(), "B".to_string(), "C".to_string()],
    };
    
    let market_id = chain
        .call_application::<ConwayBetsAbi, _>(app_id, "create_market", &(creator, market_data))
        .await
        .unwrap();
    
    // Place multiple bets concurrently
    let mut handles = vec![];
    
    for i in 0..10 {
        let chain_clone = chain.clone();
        let app_id_clone = app_id;
        let market_id_clone = market_id;
        
        handles.push(tokio::spawn(async move {
            let bettor = Owner::from([(i as u8) + 10; 32]);
            let bet_data = BetData {
                market_id: market_id_clone,
                outcome_index: (i % 3) as u32,
                amount: Amount::from((i + 1) * 10),
            };
            
            chain_clone
                .call_application::<ConwayBetsAbi, _>(app_id_clone, "place_bet", &(bettor, bet_data))
                .await
                .unwrap()
        }));
    }
    
    // Wait for all bets to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify total liquidity
    let market = chain
        .query_application::<ConwayBetsAbi, _>(app_id, "get_market", &market_id)
        .await
        .unwrap();
    
    assert!(market.total_liquidity >= Amount::from(550)); // Sum of 10+20+...+100
}

#[tokio::test]
async fn test_state_hash_consistency() {
    // Test the state hash synchronization mechanism
    let validator = TestValidator::with_current_module::<ConwayBetsAbi>().await;
    let mut chain1 = validator.new_chain().await;
    let mut chain2 = validator.new_chain().await;
    
    let app_id = chain1
        .create_application::<ConwayBetsAbi>((), (), vec![])
        .await;
    
    // Create market on chain1
    let creator = Owner::from([1u8; 32]);
    let market_data = MarketCreationData {
        title: "State Hash Test".to_string(),
        description: "Testing state hash consistency".to_string(),
        end_time: 1_000_000_000,
        outcomes: vec!["X".to_string(), "Y".to_string()],
    };
    
    let market_id = chain1
        .call_application::<ConwayBetsAbi, _>(app_id, "create_market", &(creator, market_data))
        .await
        .unwrap();
    
    // Get state hash from chain1
    let state_hash1 = chain1
        .query_application::<ConwayBetsAbi, _>(app_id, "get_market_state_hash", &market_id)
        .await
        .unwrap();
    
    // Sync chain2 with chain1
    let sync_message = ConwayBetsMessage::SyncState {
        market_id,
        state_hash: state_hash1,
        block_height: 1,
    };
    
    chain1.send_message(chain2.id(), sync_message.clone()).await.unwrap();
    chain2.handle_received_messages().await;
    
    // Verify state hash matches on chain2
    let state_hash2 = chain2
        .query_application::<ConwayBetsAbi, _>(app_id, "get_market_state_hash", &market_id)
        .await
        .unwrap();
    
    assert_eq!(state_hash1, state_hash2);
    
    // Make update on chain1 and verify chain2 detects inconsistency
    let bet_data = BetData {
        market_id,
        outcome_index: 0,
        amount: Amount::from(100),
    };
    
    chain1
        .call_application::<ConwayBetsAbi, _>(app_id, "place_bet", &(Owner::from([2u8; 32]), bet_data))
        .await
        .unwrap();
    
    let state_hash1_updated = chain1
        .query_application::<ConwayBetsAbi, _>(app_id, "get_market_state_hash", &market_id)
        .await
        .unwrap();
    
    assert_ne!(state_hash1, state_hash1_updated);
    
    // Chain2 should have old hash until synced
    let state_hash2_current = chain2
        .query_application::<ConwayBetsAbi, _>(app_id, "get_market_state_hash", &market_id)
        .await
        .unwrap();
    
    assert_eq!(state_hash2, state_hash2_current); // Still old hash
    assert_ne!(state_hash1_updated, state_hash2_current);
}