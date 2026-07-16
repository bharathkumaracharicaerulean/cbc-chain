#!/bin/bash
# Script to build separate modular dependency graphs for each crate/module

set -e

REPO_ROOT="/home/bharat/projects/cbc-chain"
D3_SRC="$REPO_ROOT/.code-review-graph/d3.v7.min.js"
CRG_BIN="/home/bharat/.local/bin/code-review-graph"

# Check if d3.js source is available
if [ ! -f "$D3_SRC" ]; then
    echo "Downloading d3.v7.min.js to root directory first..."
    mkdir -p "$REPO_ROOT/.code-review-graph"
    curl -sSL https://d3js.org/d3.v7.min.js -o "$D3_SRC"
fi

# List of modules to build
MODULES=("cbc-node" "cbc-pallets" "cbc-runtime")

for MODULE in "${MODULES[@]}"; do
    MODULE_PATH="$REPO_ROOT/$MODULE"
    
    if [ -d "$MODULE_PATH" ]; then
        echo "=========================================="
        echo "Building graph for module: $MODULE"
        echo "=========================================="
        
        # 1. Create the local configuration folder
        mkdir -p "$MODULE_PATH/.code-review-graph"
        
        # 2. Build the database for this module specifically
        $CRG_BIN build --repo "$MODULE_PATH"
        
        # 3. Generate the graph.html visualization
        $CRG_BIN visualize --repo "$MODULE_PATH"
        
        # 4. Copy local d3.js to enable offline/local loading
        cp "$D3_SRC" "$MODULE_PATH/.code-review-graph/d3.v7.min.js"
        
        echo "Successfully generated visualization: $MODULE_PATH/.code-review-graph/graph.html"
    else
        echo "Skipping $MODULE (directory not found)"
    fi
done

echo "=========================================="
echo "All modular graphs generated successfully!"
echo "=========================================="
echo "To serve them on separate ports, run:"
echo "  - cbc-node:    python3 -m http.server 8766 --directory $REPO_ROOT/cbc-node/.code-review-graph/"
echo "  - cbc-pallets: python3 -m http.server 8767 --directory $REPO_ROOT/cbc-pallets/.code-review-graph/"
echo "  - cbc-runtime: python3 -m http.server 8768 --directory $REPO_ROOT/cbc-runtime/.code-review-graph/"
