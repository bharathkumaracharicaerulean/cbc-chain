# PowerShell script for resetting the node

Write-Host "Starting node reset process..." -ForegroundColor Green

# Kill any existing node processes
Write-Host "Stopping any running node processes..."
Get-Process | Where-Object { $_.ProcessName -like "*cbc-node*" } | Stop-Process -Force -ErrorAction SilentlyContinue

# Remove the database
Write-Host "Removing node database..."
$dbPath = "$env:USERPROFILE\.local\share\cbc-node\chains\*\db"
if (Test-Path $dbPath) {
    Remove-Item -Path $dbPath -Recurse -Force
}

# Build the node if needed
Write-Host "Building node..."
cargo build --release

# Start the node in development mode
Write-Host "Starting node in development mode..." -ForegroundColor Green
.\target\release\cbc-node.exe --dev --tmp 