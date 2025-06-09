# Validator Score Checker Tool

## Purpose
The Validator Score Checker is a command-line tool that helps analyze and monitor validator performance in the CBC-Chain network. It provides a comprehensive view of validator scores by:

1. Retrieving PoS (Proof of Stake) scores from the blockchain
2. Retrieving PoI (Proof of Inference) scores from the blockchain
3. Calculating combined scores based on both metrics
4. Presenting the data in a clear, tabular format

This tool is particularly useful for:
- Monitoring validator performance
- Debugging score calculation issues
- Testing validator ranking logic
- Analyzing network health

## Example Output
```
+------------------+------------+------------+---------------+
| Validator ID     | PoS Score  | PoI Score  | Combined Score|
+------------------+------------+------------+---------------+
| 5GrwvaEF5zXb... | 1000       | 800        | 1800         |
| 5FHneW46xGXg... | 900        | 700        | 1600         |
| 5FLSigC9HGRK... | 800        | 600        | 1400         |
+------------------+------------+------------+---------------+
```

## Installation
1. Ensure you have Rust installed
2. Navigate to the project root
3. Build the tool:
```bash
cargo build --release
```

## Usage
1. Start your CBC-Chain node
2. Run the tool:
```bash
./target/release/score-checker
```

The tool will:
1. Connect to your local node (default: ws://127.0.0.1:9944)
2. Fetch validator scores from the blockchain
3. Calculate combined scores
4. Display the results in a sorted table

## Configuration
You can modify the following in the source code:
- Node connection URL
- Score calculation weights
- Output format

## Future Improvements
- Add command-line arguments for configuration
- Support for historical score analysis
- Export to CSV/JSON
- Real-time monitoring mode
- Custom scoring algorithms 