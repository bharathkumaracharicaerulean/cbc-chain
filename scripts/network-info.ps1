# PowerShell script for network information

param(
    [string]$RpcUrl = "http://localhost:9933",
    [ValidateSet("text", "json")]
    [string]$Format = "text",
    [switch]$PeersOnly,
    [switch]$StatusOnly
)

# Function to make RPC calls
function Make-RpcCall {
    param(
        [string]$Method,
        [string]$Params
    )
    
    $body = @{
        jsonrpc = "2.0"
        method = $Method
        params = $Params
        id = 1
    } | ConvertTo-Json
    
    $response = Invoke-RestMethod -Uri $RpcUrl -Method Post -Body $body -ContentType "application/json"
    return $response
}

# Function to format latency with colors
function Format-Latency {
    param([int]$Latency)
    
    if ($Latency -lt 100) {
        Write-Host "$Latency`ms" -ForegroundColor Green
    }
    elseif ($Latency -lt 500) {
        Write-Host "$Latency`ms" -ForegroundColor Yellow
    }
    else {
        Write-Host "$Latency`ms" -ForegroundColor Red
    }
}

# Get system information
if (-not $PeersOnly) {
    Write-Host "Node Status:" -ForegroundColor Green
    $systemInfo = Make-RpcCall -Method "system_health" -Params "[]"
    
    if ($Format -eq "json") {
        $systemInfo | ConvertTo-Json -Depth 10
    }
    else {
        Write-Host "Peers: $($systemInfo.result.peers)"
        Write-Host "Syncing: $($systemInfo.result.isSyncing)"
        Write-Host "Should Have Peers: $($systemInfo.result.shouldHavePeers)"
    }
}

# Get peer information
if (-not $StatusOnly) {
    Write-Host "`nPeer Information:" -ForegroundColor Green
    $peersInfo = Make-RpcCall -Method "system_peers" -Params "[]"
    
    if ($Format -eq "json") {
        $peersInfo | ConvertTo-Json -Depth 10
    }
    else {
        foreach ($peer in $peersInfo.result) {
            Write-Host "Peer ID: $($peer.peerId)"
            Write-Host "Roles: $($peer.roles)"
            Write-Host "Protocol Version: $($peer.protocolVersion)"
            Write-Host "Best Hash: $($peer.bestHash)"
            Write-Host "Best Number: $($peer.bestNumber)"
            Write-Host "Latency: " -NoNewline
            Format-Latency -Latency $peer.latency
            Write-Host ""
        }
    }
} 