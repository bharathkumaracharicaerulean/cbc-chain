#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
RPC_URL="http://localhost:9933"
FORMAT="text"

# Help function
show_help() {
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -u, --url      RPC URL (default: http://localhost:9933)"
    echo "  -f, --format   Output format (text/json) (default: text)"
    echo "  -p, --peers    Show only peer information"
    echo "  -s, --status   Show only node status"
    exit 0
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            ;;
        -u|--url)
            RPC_URL="$2"
            shift 2
            ;;
        -f|--format)
            FORMAT="$2"
            shift 2
            ;;
        -p|--peers)
            PEERS_ONLY=true
            shift
            ;;
        -s|--status)
            STATUS_ONLY=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            ;;
    esac
done

# Function to make RPC calls
make_rpc_call() {
    local method=$1
    local params=$2
    curl -s -H "Content-Type: application/json" -d "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":1}" $RPC_URL
}

# Function to format latency
format_latency() {
    local latency=$1
    if (( $(echo "$latency < 100" | bc -l) )); then
        echo -e "${GREEN}${latency}ms${NC}"
    elif (( $(echo "$latency < 500" | bc -l) )); then
        echo -e "${YELLOW}${latency}ms${NC}"
    else
        echo -e "${RED}${latency}ms${NC}"
    fi
}

# Get system information
if [ -z "$PEERS_ONLY" ]; then
    echo -e "${GREEN}Node Status:${NC}"
    system_info=$(make_rpc_call "system_health" "[]")
    if [ "$FORMAT" = "json" ]; then
        echo "$system_info"
    else
        peers=$(echo $system_info | jq -r '.result.peers')
        is_syncing=$(echo $system_info | jq -r '.result.isSyncing')
        should_have_peers=$(echo $system_info | jq -r '.result.shouldHavePeers')
        
        echo "Peers: $peers"
        echo "Syncing: $is_syncing"
        echo "Should Have Peers: $should_have_peers"
    fi
fi

# Get peer information
if [ -z "$STATUS_ONLY" ]; then
    echo -e "\n${GREEN}Peer Information:${NC}"
    peers_info=$(make_rpc_call "system_peers" "[]")
    
    if [ "$FORMAT" = "json" ]; then
        echo "$peers_info"
    else
        echo "$peers_info" | jq -r '.result[] | "Peer ID: \(.peerId)\nRoles: \(.roles)\nProtocol Version: \(.protocolVersion)\nBest Hash: \(.bestHash)\nBest Number: \(.bestNumber)\nLatency: \(.latency)ms\n\n"'
    fi
fi 