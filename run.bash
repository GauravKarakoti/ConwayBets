#!/usr/bin/env bash

set -eu

export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"

# 1. Initialize Wallet
export LINERA_FAUCET_URL=https://faucet.testnet-conway.linera.net
linera wallet init --faucet="$LINERA_FAUCET_URL"
linera wallet request-chain --faucet="$LINERA_FAUCET_URL"

# 2. Build Backend
echo "Building ConwayBets backend..."
cd linera
cargo build --release --target wasm32-unknown-unknown
cd ..

# 3. Publish Application
echo "Publishing application..."
# APP_ID requires TWO files: contract and service
APP_ID=$(linera publish-and-create \
    linera/target/wasm32-unknown-unknown/release/linera_contract.wasm \
    linera/target/wasm32-unknown-unknown/release/linera_service.wasm \
    --json-argument '{"initial_markets": []}')

echo "Application Published with ID: $APP_ID"

# 4. Start Linera Service (GraphQL API)
echo "Starting Linera Service on port 8081..."
linera service --port 8081 &

# 5. Setup and Run Frontend
echo "Setting up Frontend..."
cd frontend
echo "VITE_LINERA_ENDPOINT=https://conway.linera.io" > .env
echo "VITE_APPLICATION_ID=$APP_ID" >> .env
npm install
echo "Starting Frontend..."
npm run dev -- --host 0.0.0.0