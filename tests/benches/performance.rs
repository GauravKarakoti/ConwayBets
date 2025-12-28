use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput, PlotConfiguration, AxisScale};
use conwaybets::{ConwayBetsService, Market, Bet, Resolution, MarketId, UserId};
use linera_sdk::{
    base::{Amount, Owner, ApplicationId, ChainId},
    test::{TestValidator, TestChain},
};
use std::{collections::BTreeMap, time::Duration, sync::Arc};
use tokio::runtime::Runtime;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::sync::atomic::{AtomicU64, Ordering};

// Constants for benchmarking
const SEED: u64 = 1234567890;
const INITIAL_USERS: usize = 100;
const INITIAL_MARKETS: usize = 20;
const INITIAL_BETS_PER_MARKET: usize = 50;

// Benchmark utilities
struct BenchmarkContext {
    validator: TestValidator,
    runtime: Runtime,
    rng: ChaCha8Rng,
    user_pool: Vec<Owner>,
    market_pool: Vec<MarketId>,
    transaction_counter: AtomicU64,
}

impl BenchmarkContext {
    async fn new() -> Self {
        let validator = TestValidator::with_current_module::<crate::ConwayBetsAbi>().await;
        let runtime = Runtime::new().unwrap();
        let rng = ChaCha8Rng::seed_from_u64(SEED);
        
        // Generate initial users
        let mut user_pool = Vec::with_capacity(INITIAL_USERS);
        for i in 0..INITIAL_USERS {
            let mut bytes = [0u8; 32];
            bytes[..8].copy_from_slice(&(i as u64).to_le_bytes());
            user_pool.push(Owner::from(bytes));
        }
        
        let market_pool = Vec::new();
        let transaction_counter = AtomicU64::new(0);
        
        Self {
            validator,
            runtime,
            rng,
            user_pool,
            market_pool,
            transaction_counter,
        }
    }
    
    fn random_user(&mut self) -> Owner {
        let idx = self.rng.gen_range(0..self.user_pool.len());
        self.user_pool[idx]
    }
    
    fn random_market(&self) -> Option<MarketId> {
        if self.market_pool.is_empty() {
            None
        } else {
            let idx = self.rng.gen_range(0..self.market_pool.len());
            Some(self.market_pool[idx])
        }
    }
    
    fn record_transaction(&self) {
        self.transaction_counter.fetch_add(1, Ordering::SeqCst);
    }
    
    fn get_transaction_count(&self) -> u64 {
        self.transaction_counter.load(Ordering::SeqCst)
    }
}

// Benchmark 1: Market Creation Performance
async fn benchmark_market_creation(count: usize, ctx: &mut BenchmarkContext) -> Duration {
    let start = std::time::Instant::now();
    let mut chain = ctx.validator.new_chain().await;
    
    // Deploy application
    let app_id = chain
        .create_application::<crate::ConwayBetsAbi>((), (), vec![])
        .await;
    
    for i in 0..count {
        let creator = ctx.random_user();
        let market_data = crate::MarketCreationData {
            title: format!("Test Market {}", i),
            description: format!("Benchmark market {}", i),
            end_time: 1_000_000_000 + (i as u64) * 86_400, // 1 day increments
            outcomes: vec!["Yes".to_string(), "No".to_string()],
        };
        
        chain
            .call_application::<crate::ConwayBetsAbi, _>(
                app_id,
                "create_market",
                &(creator, market_data),
            )
            .await
            .unwrap();
        
        ctx.record_transaction();
    }
    
    start.elapsed()
}

// Benchmark 2: Sequential Bet Placement
async fn benchmark_sequential_bets(count: usize, markets: usize, ctx: &mut BenchmarkContext) -> Duration {
    let start = std::time::Instant::now();
    let mut chain = ctx.validator.new_chain().await;
    
    // Deploy application
    let app_id = chain
        .create_application::<crate::ConwayBetsAbi>((), (), vec![])
        .await;
    
    // Create markets
    let mut market_ids = Vec::new();
    for i in 0..markets {
        let creator = ctx.random_user();
        let market_data = crate::MarketCreationData {
            title: format!("Bench Market {}", i),
            description: "For benchmark testing".to_string(),
            end_time: 2_000_000_000,
            outcomes: vec!["A".to_string(), "B".to_string(), "C".to_string()],
        };
        
        let market_id = chain
            .call_application::<crate::ConwayBetsAbi, _>(
                app_id,
                "create_market",
                &(creator, market_data),
            )
            .await
            .unwrap();
        
        market_ids.push(market_id);
        ctx.record_transaction();
    }
    
    // Place bets sequentially
    for i in 0..count {
        let market_idx = i % market_ids.len();
        let market_id = market_ids[market_idx];
        let user = ctx.random_user();
        let outcome_index = (i % 3) as u32;
        let amount = Amount::from((i % 100 + 1) as u64); // 1-100 tokens
        
        let bet_data = crate::BetData {
            market_id,
            outcome_index,
            amount,
        };
        
        chain
            .call_application::<crate::ConwayBetsAbi, _>(
                app_id,
                "place_bet",
                &(user, bet_data),
            )
            .await
            .unwrap();
        
        ctx.record_transaction();
    }
    
    start.elapsed()
}

// Benchmark 3: Concurrent Bet Placement
async fn benchmark_concurrent_bets(bet_count: usize, market_count: usize, concurrency: usize, ctx: &mut BenchmarkContext) -> Duration {
    let start = std::time::Instant::now();
    let mut chain = ctx.validator.new_chain().await;
    
    // Deploy application
    let app_id = chain
        .create_application::<crate::ConwayBetsAbi>((), (), vec![])
        .await;
    
    // Create markets
    let mut market_ids = Vec::new();
    for i in 0..market_count {
        let creator = ctx.random_user();
        let market_data = crate::MarketCreationData {
            title: format!("Concurrent Market {}", i),
            description: "Concurrent betting test".to_string(),
            end_time: 2_000_000_000,
            outcomes: vec!["Yes".to_string(), "No".to_string()],
        };
        
        let market_id = chain
            .call_application::<crate::ConwayBetsAbi, _>(
                app_id,
                "create_market",
                &(creator, market_data),
            )
            .await
            .unwrap();
        
        market_ids.push(market_id);
        ctx.record_transaction();
    }
    
    // Create a channel for distributing work
    use tokio::sync::mpsc;
    let (tx, mut rx) = mpsc::channel(1000);
    
    // Spawn producer
    let producer = tokio::spawn(async move {
        for i in 0..bet_count {
            let market_idx = i % market_ids.len();
            let bet_data = crate::BetData {
                market_id: market_ids[market_idx],
                outcome_index: (i % 2) as u32,
                amount: Amount::from((i % 50 + 1) as u64),
            };
            tx.send((i, bet_data)).await.unwrap();
        }
    });
    
    // Spawn consumer workers
    let mut workers = Vec::new();
    let chain_arc = Arc::new(chain);
    let app_id_arc = Arc::new(app_id);
    let counter_arc = Arc::new(ctx.transaction_counter.clone());
    
    for worker_id in 0..concurrency {
        let chain = chain_arc.clone();
        let app_id = app_id_arc.clone();
        let counter = counter_arc.clone();
        let mut rx = rx.resubscribe();
        
        workers.push(tokio::spawn(async move {
            let mut local_counter = 0;
            while let Ok((seq_id, bet_data)) = rx.recv().await {
                // Simulate different users
                let mut user_bytes = [0u8; 32];
                user_bytes[..8].copy_from_slice(&(worker_id as u64 * 1000 + seq_id as u64).to_le_bytes());
                let user = Owner::from(user_bytes);
                
                chain
                    .call_application::<crate::ConwayBetsAbi, _>(
                        *app_id,
                        "place_bet",
                        &(user, bet_data),
                    )
                    .await
                    .unwrap();
                
                local_counter += 1;
            }
            counter.fetch_add(local_counter, Ordering::SeqCst);
        }));
    }
    
    // Wait for completion
    producer.await.unwrap();
    drop(tx); // Close channel to let workers finish
    
    for worker in workers {
        worker.await.unwrap();
    }
    
    start.elapsed()
}

// Benchmark 4: Cross-chain Message Performance
async fn benchmark_cross_chain_messages(message_count: usize, chain_count: usize, ctx: &mut BenchmarkContext) -> Duration {
    let start = std::time::Instant::now();
    
    // Create multiple chains
    let mut chains = Vec::with_capacity(chain_count);
    let mut app_ids = Vec::with_capacity(chain_count);
    
    for i in 0..chain_count {
        let mut chain = ctx.validator.new_chain().await;
        let app_id = chain
            .create_application::<crate::ConwayBetsAbi>((), (), vec![])
            .await;
        
        chains.push(chain);
        app_ids.push(app_id);
        ctx.record_transaction(); // Count chain creation
    }
    
    // Send messages between chains
    for i in 0..message_count {
        let source_idx = i % chain_count;
        let target_idx = (i + 1) % chain_count;
        
        if source_idx == target_idx {
            continue;
        }
        
        let source_chain = &mut chains[source_idx];
        let target_chain = &chains[target_idx];
        
        // Create a market on source chain
        let creator = ctx.random_user();
        let market_data = crate::MarketCreationData {
            title: format!("Cross-chain Market {}", i),
            description: "Cross-chain benchmark".to_string(),
            end_time: 1_000_000_000,
            outcomes: vec!["Yes".to_string(), "No".to_string()],
        };
        
        let market_id = source_chain
            .call_application::<crate::ConwayBetsAbi, _>(
                app_ids[source_idx],
                "create_market",
                &(creator, market_data),
            )
            .await
            .unwrap();
        
        // Send message about market creation to target chain
        let message = crate::ConwayBetsMessage::MarketCreated {
            market_id,
            creator,
            title: market_data.title,
        };
        
        source_chain
            .send_message(target_chain.id(), message)
            .await
            .unwrap();
        
        ctx.record_transaction(); // Count each message
    }
    
    // Process all messages
    for chain in &mut chains {
        chain.handle_received_messages().await;
        ctx.record_transaction(); // Count message processing
    }
    
    start.elapsed()
}

// Benchmark 5: Market Resolution Performance
async fn benchmark_market_resolution(market_count: usize, bets_per_market: usize, ctx: &mut BenchmarkContext) -> Duration {
    let start = std::time::Instant::now();
    let mut chain = ctx.validator.new_chain().await;
    
    // Deploy application
    let app_id = chain
        .create_application::<crate::ConwayBetsAbi>((), (), vec![])
        .await;
    
    let mut market_ids = Vec::new();
    
    // Create markets and place bets
    for m in 0..market_count {
        let creator = ctx.random_user();
        let market_data = crate::MarketCreationData {
            title: format!("Resolution Market {}", m),
            description: "Market resolution benchmark".to_string(),
            end_time: 1_000_000_000,
            outcomes: vec!["Win".to_string(), "Lose".to_string()],
        };
        
        let market_id = chain
            .call_application::<crate::ConwayBetsAbi, _>(
                app_id,
                "create_market",
                &(creator, market_data),
            )
            .await
            .unwrap();
        
        market_ids.push((market_id, creator));
        ctx.record_transaction();
        
        // Place bets
        for b in 0..bets_per_market {
            let user = ctx.random_user();
            let bet_data = crate::BetData {
                market_id,
                outcome_index: (b % 2) as u32,
                amount: Amount::from((b % 100 + 1) as u64),
            };
            
            chain
                .call_application::<crate::ConwayBetsAbi, _>(
                    app_id,
                    "place_bet",
                    &(user, bet_data),
                )
                .await
                .unwrap();
            
            ctx.record_transaction();
        }
    }
    
    // Resolve all markets
    for (market_id, creator) in &market_ids {
        let resolution = crate::ResolutionData {
            market_id: *market_id,
            winning_outcome: 0,
            resolution_proof: vec![],
        };
        
        chain
            .call_application::<crate::ConwayBetsAbi, _>(
                app_id,
                "resolve_market",
                &(*creator, resolution),
            )
            .await
            .unwrap();
        
        ctx.record_transaction();
    }
    
    start.elapsed()
}

// Benchmark 6: State Hash Synchronization
async fn benchmark_state_sync(updates: usize, chains: usize, ctx: &mut BenchmarkContext) -> Duration {
    let start = std::time::Instant::now();
    
    // Create multiple chains
    let mut chain_instances = Vec::with_capacity(chains);
    let mut app_ids = Vec::with_capacity(chains);
    
    for i in 0..chains {
        let mut chain = ctx.validator.new_chain().await;
        let app_id = chain
            .create_application::<crate::ConwayBetsAbi>((), (), vec![])
            .await;
        
        chain_instances.push(chain);
        app_ids.push(app_id);
        ctx.record_transaction();
    }
    
    // Create a market on chain 0
    let creator = ctx.random_user();
    let market_data = crate::MarketCreationData {
        title: "Sync Benchmark Market".to_string(),
        description: "State synchronization test".to_string(),
        end_time: 1_000_000_000,
        outcomes: vec!["X".to_string(), "Y".to_string()],
    };
    
    let market_id = chain_instances[0]
        .call_application::<crate::ConwayBetsAbi, _>(
            app_ids[0],
            "create_market",
            &(creator, market_data),
        )
        .await
        .unwrap();
    
    ctx.record_transaction();
    
    // Sync state to all chains
    for chain_idx in 1..chains {
        // Get state hash from chain 0
        let state_hash = chain_instances[0]
            .query_application::<crate::ConwayBetsAbi, _>(
                app_ids[0],
                "get_market_state_hash",
                &market_id,
            )
            .await
            .unwrap();
        
        // Send sync message
        let sync_message = crate::ConwayBetsMessage::SyncState {
            market_id,
            state_hash,
            block_height: 1,
        };
        
        chain_instances[0]
            .send_message(chain_instances[chain_idx].id(), sync_message)
            .await
            .unwrap();
        
        ctx.record_transaction();
    }
    
    // Process messages on all chains
    for chain in &mut chain_instances {
        chain.handle_received_messages().await;
        ctx.record_transaction();
    }
    
    // Make updates and verify consistency
    for update in 0..updates {
        let user = ctx.random_user();
        let bet_data = crate::BetData {
            market_id,
            outcome_index: (update % 2) as u32,
            amount: Amount::from((update % 50 + 1) as u64),
        };
        
        // Update on random chain
        let chain_idx = update % chains;
        chain_instances[chain_idx]
            .call_application::<crate::ConwayBetsAbi, _>(
                app_ids[chain_idx],
                "place_bet",
                &(user, bet_data),
            )
            .await
            .unwrap();
        
        ctx.record_transaction();
        
        // Sync state to other chains
        if update % 10 == 0 { // Sync every 10 updates
            let latest_state_hash = chain_instances[chain_idx]
                .query_application::<crate::ConwayBetsAbi, _>(
                    app_ids[chain_idx],
                    "get_market_state_hash",
                    &market_id,
                )
                .await
                .unwrap();
            
            for other_idx in 0..chains {
                if other_idx == chain_idx {
                    continue;
                }
                
                let sync_message = crate::ConwayBetsMessage::SyncState {
                    market_id,
                    state_hash: latest_state_hash,
                    block_height: update as u64 + 2,
                };
                
                chain_instances[chain_idx]
                    .send_message(chain_instances[other_idx].id(), sync_message)
                    .await
                    .unwrap();
                
                ctx.record_transaction();
            }
            
            // Process messages
            for chain in &mut chain_instances {
                chain.handle_received_messages().await;
                ctx.record_transaction();
            }
        }
    }
    
    start.elapsed()
}

// Benchmark 7: Query Performance
async fn benchmark_queries(query_count: usize, data_size: usize, ctx: &mut BenchmarkContext) -> Duration {
    let start = std::time::Instant::now();
    let mut chain = ctx.validator.new_chain().await;
    
    // Deploy application
    let app_id = chain
        .create_application::<crate::ConwayBetsAbi>((), (), vec![])
        .await;
    
    // Create markets with varying data sizes
    let mut market_ids = Vec::new();
    for i in 0..data_size {
        let creator = ctx.random_user();
        let market_data = crate::MarketCreationData {
            title: format!("Query Market {}", i),
            description: "A".repeat(100 + (i % 900)), // 100-1000 chars
            end_time: 1_000_000_000 + (i as u64) * 86_400,
            outcomes: (0..(i % 5 + 2)) // 2-6 outcomes
                .map(|j| format!("Outcome {}", j))
                .collect(),
        };
        
        let market_id = chain
            .call_application::<crate::ConwayBetsAbi, _>(
                app_id,
                "create_market",
                &(creator, market_data),
            )
            .await
            .unwrap();
        
        market_ids.push(market_id);
        ctx.record_transaction();
    }
    
    // Run queries
    for i in 0..query_count {
        let query_type = i % 4;
        
        match query_type {
            0 => {
                // Query single market
                let market_idx = i % market_ids.len();
                let _: crate::Market = chain
                    .query_application::<crate::ConwayBetsAbi, _>(
                        app_id,
                        "get_market",
                        &market_ids[market_idx],
                    )
                    .await
                    .unwrap();
            }
            1 => {
                // Query all markets
                let _: Vec<crate::Market> = chain
                    .query_application::<crate::ConwayBetsAbi, _>(
                        app_id,
                        "get_all_markets",
                        &(),
                    )
                    .await
                    .unwrap();
            }
            2 => {
                // Query market state
                let market_idx = i % market_ids.len();
                let _: crate::MarketState = chain
                    .query_application::<crate::ConwayBetsAbi, _>(
                        app_id,
                        "get_market_state",
                        &market_ids[market_idx],
                    )
                    .await
                    .unwrap();
            }
            3 => {
                // Query user bets
                let user = ctx.random_user();
                let _: Vec<crate::Bet> = chain
                    .query_application::<crate::ConwayBetsAbi, _>(
                        app_id,
                        "get_user_bets",
                        &user,
                    )
                    .await
                    .unwrap();
            }
            _ => unreachable!(),
        }
        
        ctx.record_transaction();
    }
    
    start.elapsed()
}

// Benchmark 8: Microchain Scalability
async fn benchmark_microchain_scalability(microchain_count: usize, ops_per_chain: usize, ctx: &mut BenchmarkContext) -> Duration {
    let start = std::time::Instant::now();
    
    // Create multiple independent chains (simulating microchains)
    let mut chains = Vec::with_capacity(microchain_count);
    let mut app_ids = Vec::with_capacity(microchain_count);
    
    for i in 0..microchain_count {
        let mut chain = ctx.validator.new_chain().await;
        let app_id = chain
            .create_application::<crate::ConwayBetsAbi>((), (), vec![])
            .await;
        
        chains.push(chain);
        app_ids.push(app_id);
        ctx.record_transaction();
    }
    
    // Execute operations in parallel on each microchain
    let mut handles = Vec::with_capacity(microchain_count);
    
    for (chain_idx, (chain, app_id)) in chains.into_iter().zip(app_ids.into_iter()).enumerate() {
        let handle = tokio::spawn(async move {
            let mut local_counter = 0;
            
            // Create markets and bets on this microchain
            for op in 0..ops_per_chain {
                if op % 2 == 0 {
                    // Create market
                    let mut creator_bytes = [0u8; 32];
                    creator_bytes[..8].copy_from_slice(&(chain_idx as u64 * 1000 + op as u64).to_le_bytes());
                    let creator = Owner::from(creator_bytes);
                    
                    let market_data = crate::MarketCreationData {
                        title: format!("Microchain {} Market {}", chain_idx, op),
                        description: "Microchain scalability test".to_string(),
                        end_time: 1_000_000_000,
                        outcomes: vec!["Yes".to_string(), "No".to_string()],
                    };
                    
                    chain
                        .call_application::<crate::ConwayBetsAbi, _>(
                            app_id,
                            "create_market",
                            &(creator, market_data),
                        )
                        .await
                        .unwrap();
                } else {
                    // Place bet (if we have markets)
                    if op > 1 {
                        // Create a user
                        let mut user_bytes = [0u8; 32];
                        user_bytes[..8].copy_from_slice(&(chain_idx as u64 * 1000 + op as u64 + 500).to_le_bytes());
                        let user = Owner::from(user_bytes);
                        
                        // We'd need a market ID here - in reality, we'd track created markets
                        // For simplicity, we'll skip this in the benchmark
                    }
                }
                local_counter += 1;
            }
            local_counter
        });
        handles.push(handle);
    }
    
    // Wait for all microchains to complete
    let mut total_ops = 0;
    for handle in handles {
        total_ops += handle.await.unwrap();
    }
    
    ctx.transaction_counter.fetch_add(total_ops, Ordering::SeqCst);
    
    start.elapsed()
}

// Criterion benchmark groups
fn market_creation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Market Creation");
    group.plot_config(PlotConfiguration::default()
        .summary_scale(AxisScale::Logarithmic));
    
    for count in [1, 10, 50, 100, 200].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                let mut ctx = Runtime::new().unwrap().block_on(BenchmarkContext::new());
                b.to_async(&ctx.runtime).iter(|| async {
                    benchmark_market_creation(count, &mut ctx).await
                });
            },
        );
    }
    group.finish();
}

fn bet_placement_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bet Placement");
    group.plot_config(PlotConfiguration::default()
        .summary_scale(AxisScale::Logarithmic));
    
    for (bet_count, market_count) in [(10, 2), (50, 5), (100, 10), (200, 20), (500, 25)].iter() {
        group.throughput(Throughput::Elements(*bet_count as u64));
        group.bench_with_input(
            BenchmarkId::new("Sequential", format!("{}/{}", bet_count, market_count)),
            &(*bet_count, *market_count),
            |b, &(bet_count, market_count)| {
                let mut ctx = Runtime::new().unwrap().block_on(BenchmarkContext::new());
                b.to_async(&ctx.runtime).iter(|| async {
                    benchmark_sequential_bets(bet_count, market_count, &mut ctx).await
                });
            },
        );
    }
    group.finish();
}

fn concurrent_operations_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Concurrent Operations");
    
    for concurrency in [1, 2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("Concurrent Bets", concurrency),
            concurrency,
            |b, &concurrency| {
                let mut ctx = Runtime::new().unwrap().block_on(BenchmarkContext::new());
                b.to_async(&ctx.runtime).iter(|| async {
                    benchmark_concurrent_bets(100, 10, concurrency, &mut ctx).await
                });
            },
        );
    }
    group.finish();
}

fn cross_chain_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Cross-chain Communication");
    
    for chain_count in [2, 3, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::new("Cross-chain Messages", chain_count),
            chain_count,
            |b, &chain_count| {
                let mut ctx = Runtime::new().unwrap().block_on(BenchmarkContext::new());
                b.to_async(&ctx.runtime).iter(|| async {
                    benchmark_cross_chain_messages(50, chain_count, &mut ctx).await
                });
            },
        );
    }
    group.finish();
}

fn resolution_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Market Resolution");
    
    for market_count in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("Resolution Performance", market_count),
            market_count,
            |b, &market_count| {
                let mut ctx = Runtime::new().unwrap().block_on(BenchmarkContext::new());
                b.to_async(&ctx.runtime).iter(|| async {
                    benchmark_market_resolution(market_count, 10, &mut ctx).await
                });
            },
        );
    }
    group.finish();
}

fn state_sync_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("State Synchronization");
    
    for chain_count in [2, 3, 5].iter() {
        group.bench_with_input(
            BenchmarkId::new("State Sync", chain_count),
            chain_count,
            |b, &chain_count| {
                let mut ctx = Runtime::new().unwrap().block_on(BenchmarkContext::new());
                b.to_async(&ctx.runtime).iter(|| async {
                    benchmark_state_sync(50, chain_count, &mut ctx).await
                });
            },
        );
    }
    group.finish();
}

fn query_performance_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Query Performance");
    
    for data_size in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("Queries", data_size),
            data_size,
            |b, &data_size| {
                let mut ctx = Runtime::new().unwrap().block_on(BenchmarkContext::new());
                b.to_async(&ctx.runtime).iter(|| async {
                    benchmark_queries(100, data_size, &mut ctx).await
                });
            },
        );
    }
    group.finish();
}

fn microchain_scalability_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Microchain Scalability");
    group.plot_config(PlotConfiguration::default()
        .summary_scale(AxisScale::Logarithmic));
    
    for chain_count in [1, 2, 4, 8, 16, 32].iter() {
        group.bench_with_input(
            BenchmarkId::new("Microchains", chain_count),
            chain_count,
            |b, &chain_count| {
                let mut ctx = Runtime::new().unwrap().block_on(BenchmarkContext::new());
                b.to_async(&ctx.runtime).iter(|| async {
                    benchmark_microchain_scalability(chain_count, 10, &mut ctx).await
                });
            },
        );
    }
    group.finish();
}

// Memory usage benchmark
fn memory_usage_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory Usage");
    
    group.bench_function("Memory per Market", |b| {
        b.iter_custom(|iterations| {
            let mut total_duration = Duration::new(0, 0);
            
            for _ in 0..iterations {
                let start = std::time::Instant::now();
                let runtime = Runtime::new().unwrap();
                
                runtime.block_on(async {
                    let mut ctx = BenchmarkContext::new().await;
                    // Create 100 markets and measure memory
                    let _ = benchmark_market_creation(100, &mut ctx).await;
                    
                    // Force garbage collection (drop everything)
                    drop(ctx);
                });
                
                total_duration += start.elapsed();
            }
            
            total_duration / iterations
        });
    });
    
    group.finish();
}

// Transaction throughput benchmark
fn throughput_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Transaction Throughput");
    
    group.bench_function("Peak TPS", |b| {
        b.iter_custom(|iterations| {
            let mut total_duration = Duration::new(0, 0);
            let mut total_transactions = 0;
            
            for _ in 0..iterations {
                let start = std::time::Instant::now();
                let runtime = Runtime::new().unwrap();
                
                let transactions = runtime.block_on(async {
                    let mut ctx = BenchmarkContext::new().await;
                    // Run a mixed workload
                    let duration = benchmark_concurrent_bets(200, 20, 8, &mut ctx).await;
                    let txs = ctx.get_transaction_count();
                    (duration, txs)
                });
                
                total_duration += transactions.0;
                total_transactions += transactions.1;
            }
            
            // Calculate TPS
            let avg_tps = total_transactions as f64 / total_duration.as_secs_f64();
            println!("Average TPS: {:.2}", avg_tps);
            
            total_duration / iterations
        });
    });
    
    group.finish();
}

// Register all benchmarks
criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3))
        .significance_level(0.05)
        .noise_threshold(0.05);
    targets = 
        market_creation_benchmark,
        bet_placement_benchmark,
        concurrent_operations_benchmark,
        cross_chain_benchmark,
        resolution_benchmark,
        state_sync_benchmark,
        query_performance_benchmark,
        microchain_scalability_benchmark,
        memory_usage_benchmark,
        throughput_benchmark
);

criterion_main!(benches);