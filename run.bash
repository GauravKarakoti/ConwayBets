#!/usr/bin/env bash

set -eu

# 1. Start Local Network
eval "$(linera net helper)"
linera_spawn linera net up --with-faucet

# 2. Initialize Wallet
export LINERA_FAUCET_URL=https://conway.linera.io
linera wallet init --faucet="$LINERA_FAUCET_URL"
linera wallet request-chain --faucet="$LINERA_FAUCET_URL"

# 3. Build Backend
echo "Building ConwayBets backend..."
cd linera
cargo build --release --target wasm32-unknown-unknown
cd ..

# 4. Publish Application
echo "Publishing application..."
# Note: Using 'linera' as the prefix based on your Cargo.toml package name.
# If you renamed the package to 'conwaybets', update the filenames below.
APP_ID=$(linera publish-and-create \
    linera/target/wasm32-unknown-unknown/release/linera_contract.wasm \
    linera/target/wasm32-unknown-unknown/release/linera_service.wasm \
    --service-arguments '{"initial_markets": []}')

echo "Application Published with ID: $APP_ID"

# 5. Start Linera Service (GraphQL API)
echo "Starting Linera Service on port 8081..."
linera service --port 8081 &

# 6. Setup and Run Frontend
echo "Setting up Frontend..."
cd frontend

# Create .env file for the frontend
# Note: You need to provide your VITE_DYNAMIC_ENVIRONMENT_ID manually or set it here
echo "VITE_LINERA_ENDPOINT=https://conway.linera.io" > .env
echo "VITE_APPLICATION_ID=$APP_ID" >> .env

npm install

echo "Starting Frontend..."
# --host 0.0.0.0 is required to expose the vite server outside the container
npm run dev -- --host 0.0.0.0