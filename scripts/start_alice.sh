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

# Helper function to get valid Peer ID
get_peer_id() {
    local key_file="$1"
    if [ -f "$key_file" ]; then
        "$BINARY" key inspect-node-key --file "$key_file" 2>/dev/null | grep -E '^12D3K[a-zA-Z0-9]+' | tail -n 1
    fi
}

# Helper function to ensure network key exists
ensure_net_key() {
    local key_file="$1"
    if [ ! -f "$key_file" ]; then
        mkdir -p "$(dirname "$key_file")"
        "$BINARY" key generate-node-key --file "$key_file" 2>/dev/null || true
        chmod 600 "$key_file" 2>/dev/null || true
    fi
}

# Ensure network keys exist for local nodes
ensure_net_key "$BASE_PATH/chains/cbc_local/network/secret_ed25519"
ensure_net_key "$HOME/.local/share/cbc-node/bob/chains/cbc_local/network/secret_ed25519"
ensure_net_key "$HOME/.local/share/cbc-node/charlie/chains/cbc_local/network/secret_ed25519"

BOB_PEER_ID=$(get_peer_id "$HOME/.local/share/cbc-node/bob/chains/cbc_local/network/secret_ed25519")
CHARLIE_PEER_ID=$(get_peer_id "$HOME/.local/share/cbc-node/charlie/chains/cbc_local/network/secret_ed25519")
BHARATH_PEER_ID=$(get_peer_id "$HOME/.local/share/cbc-node/bharath/chains/cbc_local/network/secret_ed25519")

RESERVED_NODES=()
BOOTNODES=()

if [ -n "$BOB_PEER_ID" ]; then
    RESERVED_NODES+=("/ip4/127.0.0.1/tcp/30334/p2p/$BOB_PEER_ID")
    BOOTNODES+=("/ip4/127.0.0.1/tcp/30334/p2p/$BOB_PEER_ID")
fi
if [ -n "$CHARLIE_PEER_ID" ]; then
    RESERVED_NODES+=("/ip4/127.0.0.1/tcp/30335/p2p/$CHARLIE_PEER_ID")
fi
if [ -n "$BHARATH_PEER_ID" ]; then
    RESERVED_NODES+=("/ip4/127.0.0.1/tcp/30336/p2p/$BHARATH_PEER_ID")
fi

EXTRA_ARGS=()
if [ ${#RESERVED_NODES[@]} -gt 0 ]; then
    EXTRA_ARGS+=(--reserved-nodes "${RESERVED_NODES[@]}")
fi
if [ ${#BOOTNODES[@]} -gt 0 ]; then
    EXTRA_ARGS+=(--bootnodes "${BOOTNODES[@]}")
fi

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
    "${EXTRA_ARGS[@]}" \
    >> "$LOG_FILE" 2>&1
