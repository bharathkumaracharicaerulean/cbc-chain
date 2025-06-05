#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting node reset process...${NC}"

# Kill any existing node processes
echo "Stopping any running node processes..."
pkill -f cbc-node || true

# Remove the database
echo "Removing node database..."
rm -rf ~/.local/share/cbc-node/chains/*/db

# Build the node if needed
echo "Building node..."
cargo build --release

# Start the node in development mode
echo -e "${GREEN}Starting node in development mode...${NC}"
./target/release/cbc-node --dev --tmp 