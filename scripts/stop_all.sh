#!/bin/bash
# Stop cbc-node processes and optionally clean data directories and logs.

# Base path for local node data
DATA_DIR="$HOME/.local/share/cbc-node"

# Function to stop nodes
stop_nodes() {
    echo "Stopping all CBC Chain nodes..."
    if pkill -f "cbc-node"; then
        echo "Sent SIGTERM to all cbc-node processes."
        sleep 2
        # Force-kill anything still alive
        if pkill -9 -f "cbc-node" 2>/dev/null; then
            echo "Force-killed remaining cbc-node processes."
        fi
    else
        echo "No cbc-node processes found."
    fi
}

# Function to clean data and temporary node files
clean_all() {
    echo "Cleaning node temporary files and database..."
    if [ -d "$DATA_DIR" ]; then
        rm -rf "$DATA_DIR"
        echo "Successfully deleted data directory (temp files & database): $DATA_DIR"
    else
        echo "Data directory not found: $DATA_DIR"
    fi
}

# Parse command line options
CLEAN_MODE=""
while [[ "$#" -gt 0 ]]; do
    case $1 in
        -c|--clean) CLEAN_MODE="clean"; shift ;;
        -s|--stop-only) CLEAN_MODE="stop"; shift ;;
        -h|--help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  -s, --stop-only  Stop running cbc-node processes only"
            echo "  -c, --clean      Stop nodes and clean all node data & temporary files"
            echo "  -h, --help       Show this help message"
            exit 0
            ;;
        *) echo "Unknown parameter passed: $1"; exit 1 ;;
    esac
done

if [ -z "$CLEAN_MODE" ]; then
    # Interactive mode
    echo "============================================="
    echo "          CBC Chain Node Control             "
    echo "============================================="
    echo "Please choose an action:"
    echo " 1) Stop nodes only"
    echo " 2) Stop nodes and clean data/temp files (Fresh Start)"
    echo " 3) Cancel"
    echo "============================================="
    read -rp "Enter choice [1-3]: " choice

    case $choice in
        1)
            CLEAN_MODE="stop"
            ;;
        2)
            CLEAN_MODE="clean"
            ;;
        *)
            echo "Operation cancelled."
            exit 0
            ;;
    esac
fi

# Execute based on selection
if [ "$CLEAN_MODE" == "stop" ]; then
    stop_nodes
    echo "Done."
elif [ "$CLEAN_MODE" == "clean" ]; then
    stop_nodes
    clean_all
    echo "Done. Node temporary files and database are deleted. Next start will begin a new chain!"
fi
