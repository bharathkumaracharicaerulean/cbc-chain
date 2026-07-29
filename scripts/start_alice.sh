#!/bin/bash
# Start Alice — bootnode and primary RPC endpoint.
# No manual key generation or directory setup needed; the node handles everything automatically.

set -e

BINARY="${BINARY:-./target/release/cbc-node}"
BASE_PATH="${BASE_PATH:-$HOME/.local/share/cbc-node/alice}"
LOG_FILE="${LOG_FILE:-node_output_alice.log}"

if [ ! -f "$BINARY" ]; then
    echo "ERROR: Binary not found at $BINARY. Run: cargo build --release"
    exit 1
fi

echo "Starting Alice (bootnode)..."
echo "  Data dir  : $BASE_PATH"
echo "  RPC port  : 9944"
echo "  Log file  : $LOG_FILE"

# Derive other peers
BOB_PEER_ID=$("$BINARY" key inspect-node-key --file "$HOME/.local/share/cbc-node/bob/chains/cbc_local/network/secret_ed25519" 2>/dev/null | tail -n 1)
CHARLIE_PEER_ID=$("$BINARY" key inspect-node-key --file "$HOME/.local/share/cbc-node/charlie/chains/cbc_local/network/secret_ed25519" 2>/dev/null | tail -n 1)
BHARATH_PEER_ID=$("$BINARY" key inspect-node-key --file "$HOME/.local/share/cbc-node/bharath/chains/cbc_local/network/secret_ed25519" 2>/dev/null | tail -n 1)

exec "$BINARY" \
    --base-path "$BASE_PATH" \
    --chain local \
    --alice \
    --rpc-port 9944 \
    --prometheus-port 9615 \
    --prometheus-external \
    --unsafe-rpc-external \
    --rpc-methods unsafe \
    --rpc-cors all \
    --no-mdns \
    --rpc-max-connections 5000 \
    --listen-addr /ip4/127.0.0.1/tcp/30333 \
    --allow-private-ip \
    --enable-cbc-extensions \
    --validator \
    --name Alice \
    --reserved-nodes "/ip4/127.0.0.1/tcp/30334/p2p/$BOB_PEER_ID" "/ip4/127.0.0.1/tcp/30335/p2p/$CHARLIE_PEER_ID" "/ip4/127.0.0.1/tcp/30336/p2p/$BHARATH_PEER_ID" \
    --bootnodes "/ip4/127.0.0.1/tcp/30334/p2p/$BOB_PEER_ID" \
    >> "$LOG_FILE" 2>&1
