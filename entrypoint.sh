#!/bin/bash

# Read env file inside Docker
if [ -f ".env" ]; then
  export $(grep -v '^#' .env | xargs)
fi

echo "Starting container with network: $NETWORK"

# Apply solana config based on NETWORK env
if [ "$NETWORK" == "mainnet" ]; then
  solana config set --url https://api.mainnet-beta.solana.com
elif [ "$NETWORK" == "devnet" ]; then
  solana config set --url https://api.devnet.solana.com
elif [ "$NETWORK" == "testnet" ]; then
  solana config set --url https://api.testnet.solana.com
else
  echo "Unknown network: $NETWORK"
  exit 1
fi

# Set keypair path
solana config set --keypair /root/.config/solana/id.json

# Start your application (or keep alive)
exec "$@"
