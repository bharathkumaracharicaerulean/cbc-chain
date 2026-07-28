#!/bin/bash
# Start Bharath — connects to Alice as bootnode.
# Generates a random keypair dynamically inside the keystore on first start.

set -e

BINARY="${BINARY:-./target/release/cbc-node}"
BASE_PATH="${BASE_PATH:-$HOME/.local/share/cbc-node/bharath}"
LOG_FILE="${LOG_FILE:-node_output_bharath.log}"
ALICE_BASE_PATH="${ALICE_BASE_PATH:-$HOME/.local/share/cbc-node/alice}"

if [ ! -f "$BINARY" ]; then
    echo "ERROR: Binary not found at $BINARY. Run: cargo build --release"
    exit 1
fi

ALICE_NET_KEY="$ALICE_BASE_PATH/chains/cbc_local/network/secret_ed25519"
MAX_RETRIES=15
RETRY=0

echo "Waiting for Alice's network key at $ALICE_NET_KEY..."
while [ ! -f "$ALICE_NET_KEY" ] && [ $RETRY -lt $MAX_RETRIES ]; do
    sleep 2
    RETRY=$((RETRY + 1))
    echo "  ...attempt $RETRY/$MAX_RETRIES"
done

if [ ! -f "$ALICE_NET_KEY" ]; then
    echo "ERROR: Alice's network key not found after ${MAX_RETRIES} attempts."
    exit 1
fi

ALICE_PEER_ID=$("$BINARY" key inspect-node-key --file "$ALICE_NET_KEY" 2>/dev/null | tail -n 1)
if [ -z "$ALICE_PEER_ID" ]; then
    echo "ERROR: Could not derive Alice's peer ID from $ALICE_NET_KEY"
    exit 1
fi

echo "Starting Bharath..."
echo "  Data dir   : $BASE_PATH"
echo "  RPC port   : 9947"
echo "  Bootnode   : /ip4/127.0.0.1/tcp/30333/p2p/$ALICE_PEER_ID"
echo "  Log file   : $LOG_FILE"

exec "$BINARY" \
    --base-path "$BASE_PATH" \
    --chain local \
    --rpc-port 9947 \
    --prometheus-port 9618 \
    --prometheus-external \
    --unsafe-rpc-external \
    --rpc-cors all \
    --no-mdns \
    --rpc-max-connections 5000 \
    --listen-addr /ip4/127.0.0.1/tcp/30336 \
    --allow-private-ip \
    --enable-cbc-extensions \
    --validator \
    --name bharath \
    --bootnodes "/ip4/127.0.0.1/tcp/30333/p2p/$ALICE_PEER_ID" \
    --reserved-nodes "/ip4/127.0.0.1/tcp/30333/p2p/$ALICE_PEER_ID" \
    >> "$LOG_FILE" 2>&1
