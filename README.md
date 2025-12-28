# ConwayBets 🚀
**Real-Time Prediction Markets on Linera Conway Testnet**

## Overview
ConwayBets is a next-generation prediction market platform leveraging Linera's microchain architecture to deliver instant trading, real-time resolution, and scalable markets for live events.

## Features
- ⚡ **Sub-second trade execution** on Linera microchains
- 🎯 **Real-time event integration** for automatic market resolution
- 🤖 **AI Market Makers** providing continuous liquidity
- 📱 **Web2-like UX** with instant updates
- 🔒 **Trustless resolution** via TEE-based oracles
- 📊 **Social trading** and strategy copying

## Prerequisites
- Rust 1.70+
- Node.js 18+
- Linera CLI v0.7.0+
- Docker (for TEE oracle)
- Conway testnet access

## Quick Start

### 1. Install Dependencies
```bash
# Install Linera CLI
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/linera-io/linera-protocol/raw/main/install-linera.sh | sh

# Clone repository
git clone https://github.com/your-org/conwaybets
cd conwaybets
```

### 2. Set Up Local Network
```bash
# Start local Linera network
linera net up --testing-prng-seed 37

# Deploy ConwayBets service
linera service deploy-with-arguments \
    target/wasm32-unknown-unknown/release/conwaybets_{service,contract}.wasm \
    --service-arguments '{"initial_markets": []}'
```

### 3. Configure Conway Testnet
```bash
# Set testnet endpoint
export LINERA_ENDPOINT="https://conway.linera.io:443"

# Fund wallet
linera wallet request-faucet-funds

# Publish to testnet
linera publish-and-create \
    target/wasm32-unknown-unknown/release/conwaybets_{service,contract}.wasm \
    --service-arguments '{"initial_markets": []}'
```

### 4. Run Frontend
```bash
cd frontend
npm install
npm run dev
```

## Project Structure
```text
conwaybets/
├── service/           # Linera service (Rust)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── market.rs
│   │   └── resolution.rs
│   └── Cargo.toml
├── contract/          # Linera contract (Rust)
├── frontend/          # React application
│   ├── src/
│   │   ├── components/
│   │   ├── hooks/
│   │   └── utils/
├── oracle/            # TEE-based resolution service
├── docker-compose.yml
└── README.md
```

## Key Linera Features Used

### 1. Microchain Per Market
```rust
// Each market gets isolated microchain
let market_chain = self.create_microchain().await?;
```

### 2. Instant Finality
```rust
// Transactions confirm in <1 second
let receipt = self.place_bet(market_id, bet).await;
assert!(receipt.is_finalized());
```

### 3. Cross-Chain Messaging
```rust
// Send resolution across chains
self.send_message(target_chain, ResolutionMessage::new(outcome));
```

### 4. GraphQL Subscriptions
```javascript
// Real-time updates in frontend
const { data, loading } = useSubscription(MARKET_UPDATES, {
  variables: { marketId }
});
```

## Testing

### Unit Tests
```bash
cargo test --package conwaybets-service
```

### Integration Tests
```bash
# Test with local network
./scripts/test_integration.sh
```

### Load Testing
```bash
# Simulate high-frequency trading
cargo run --bin load_test -- --tps 1000
```

## Deployment to Conway Testnet

1. **Build WASM artifacts:**
```bash
./scripts/build_release.sh
```

2. **Deploy service:**
```bash
linera --with-conway publish-and-create \
    target/wasm32-unknown-unknown/release/conwaybets_{service,contract}.wasm
```

3. **Verify deployment:**
```bash
linera --with-conway query service <SERVICE_ID>
```

## API Reference

### Service Methods
- `create_market(event: EventData) → MarketId`
- `place_bet(market_id: MarketId, side: Side, amount: u64) → Receipt`
- `resolve_market(market_id: MarketId, outcome: Outcome) → bool`
- `get_market_state(market_id: MarketId) → MarketState`

### GraphQL Queries
```graphql
subscription MarketUpdates($marketId: ID!) {
  marketState(marketId: $marketId) {
    price
    volume
    openInterest
  }
}
```

## Team
- Lead Developer: Gaurav Karakoti
- Contact: Telegram: @GauravKarakoti | X: @GauravKara_Koti

## Development Progress
- Wave 1: Basic market creation and betting (Complete)
- Wave 2: Real-time resolution and AI market makers (In Progress)
- Wave 3: Social features and mobile app (Planned)

## Resources
- [Linera Documentation](https://linera.io/docs)
- [Conway Testnet Guide](https://docs.linera.io/testnet/conway)
- [GitHub Issues](https://github.com/GauravKarakoti/conwaybets/issues)
