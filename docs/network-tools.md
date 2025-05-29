# Network Tools Documentation

This document describes the network tools available for monitoring and managing the CBC node.

## Node Reset Scripts

### Linux (reset-node.sh)

The `reset-node.sh` script resets the node database and restarts it in development mode.

```bash
./scripts/reset-node.sh
```

This script will:
1. Stop any running node processes
2. Remove the node database
3. Rebuild the node
4. Start the node in development mode

### Windows (reset-node.ps1)

The `reset-node.ps1` script provides the same functionality for Windows systems.

```powershell
.\scripts\reset-node.ps1
```

## Network Information Tools

### Linux (network-info.sh)

The `network-info.sh` script provides detailed information about node status and peer connections.

```bash
./scripts/network-info.sh [options]
```

Options:
- `-h, --help`: Show help message
- `-u, --url`: RPC URL (default: http://localhost:9933)
- `-f, --format`: Output format (text/json) (default: text)
- `-p, --peers`: Show only peer information
- `-s, --status`: Show only node status

Examples:
```bash
# Show all information in text format
./scripts/network-info.sh

# Show only peer information in JSON format
./scripts/network-info.sh -f json -p

# Connect to a remote node
./scripts/network-info.sh -u http://remote-node:9933
```

### Windows (network-info.ps1)

The PowerShell version provides the same functionality:

```powershell
.\scripts\network-info.ps1 [options]
```

Parameters:
- `-RpcUrl`: RPC URL (default: http://localhost:9933)
- `-Format`: Output format (text/json) (default: text)
- `-PeersOnly`: Show only peer information
- `-StatusOnly`: Show only node status

Examples:
```powershell
# Show all information in text format
.\scripts\network-info.ps1

# Show only peer information in JSON format
.\scripts\network-info.ps1 -Format json -PeersOnly

# Connect to a remote node
.\scripts\network-info.ps1 -RpcUrl http://remote-node:9933
```

## Output Format

### Text Format
The text format provides human-readable output with color-coded latency information:
- Green: < 100ms
- Yellow: 100-500ms
- Red: > 500ms

### JSON Format
The JSON format provides structured data suitable for programmatic processing:
```json
{
  "result": {
    "peers": 5,
    "isSyncing": false,
    "shouldHavePeers": true
  }
}
```

## Performance Monitoring

The network tools can be used to monitor node performance:

1. Check peer connections:
   ```bash
   ./scripts/network-info.sh -p
   ```

2. Monitor node health:
   ```bash
   ./scripts/network-info.sh -s
   ```

3. Export metrics for monitoring:
   ```bash
   ./scripts/network-info.sh -f json > metrics.json
   ```

## Troubleshooting

Common issues and solutions:

1. Connection refused:
   - Verify the node is running
   - Check RPC port (default: 9933)
   - Ensure firewall allows the connection

2. No peers:
   - Check network connectivity
   - Verify bootnode configuration
   - Check peer discovery settings

3. High latency:
   - Check network conditions
   - Verify peer locations
   - Consider using closer peers 