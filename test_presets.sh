#!/bin/bash

echo "Testing CBC Genesis Config Presets..."
echo "====================================="

# Test development preset
echo "1. Testing development preset..."
./target/release/cbc-node build-spec --chain development --raw > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ Development preset: PASSED"
else
    echo "❌ Development preset: FAILED"
fi

# Test local preset
echo "2. Testing local preset..."
./target/release/cbc-node build-spec --chain local --raw > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ Local preset: PASSED"
else
    echo "❌ Local preset: FAILED"
fi

# Test multi_validator preset
echo "3. Testing multi_validator preset..."
./target/release/cbc-node build-spec --chain multi_validator --raw > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ Multi-validator preset: PASSED"
else
    echo "❌ Multi-validator preset: FAILED"
fi

# Test high_stake preset
echo "4. Testing high_stake preset..."
./target/release/cbc-node build-spec --chain high_stake --raw > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ High-stake preset: PASSED"
else
    echo "❌ High-stake preset: FAILED"
fi

echo ""
echo "All preset tests completed!"