#!/usr/bin/env bash

set -eu

export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"

export LINERA_FAUCET_URL=https://faucet.testnet-conway.linera.net
linera wallet init --faucet="$LINERA_FAUCET_URL"
linera wallet request-chain --faucet="$LINERA_FAUCET_URL"

echo "Building ConwayBets backend..."
cd linera
cargo build --release --target wasm32-unknown-unknown
cd ..

echo "Publishing application..."
APP_ID=$(linera publish-and-create \
    linera/target/wasm32-unknown-unknown/release/linera_contract.wasm \
    linera/target/wasm32-unknown-unknown/release/linera_service.wasm \
    --json-argument '{"initial_markets": []}')

echo "Application Published with ID: $APP_ID"

echo "Starting Linera Service on port 8081..."
linera service --port 8081 &

echo "Setting up Frontend..."
cd frontend
echo "VITE_LINERA_ENDPOINT=https://faucet.testnet-conway.linera.net" > .env
echo "VITE_APPLICATION_ID=$APP_ID" >> .env
npm install
echo "Starting Frontend..."
npm run dev -- --host 0.0.0.0