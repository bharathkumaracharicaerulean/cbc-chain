#!/bin/bash

LOG_FILE="detailed_resource_usage.log"
INTERVAL=3

echo "=== Detailed Resource Monitoring for Rust Compilation ===" | tee $LOG_FILE
echo "Started at: $(date)" | tee -a $LOG_FILE
echo "" | tee -a $LOG_FILE

# Function to log system state
log_system_state() {
    echo "=== $(date) ===" | tee -a $LOG_FILE
    
    # Memory usage
    echo "Memory Usage:" | tee -a $LOG_FILE
    free -h | tee -a $LOG_FILE
    echo "" | tee -a $LOG_FILE
    
    # CPU and Load
    echo "CPU and Load:" | tee -a $LOG_FILE
    uptime | tee -a $LOG_FILE
    echo "" | tee -a $LOG_FILE
    
    # Top processes by CPU
    echo "Top CPU consumers:" | tee -a $LOG_FILE
    ps aux --sort=-%cpu | head -10 | tee -a $LOG_FILE
    echo "" | tee -a $LOG_FILE
    
    # Top processes by Memory
    echo "Top Memory consumers:" | tee -a $LOG_FILE
    ps aux --sort=-%mem | head -10 | tee -a $LOG_FILE
    echo "" | tee -a $LOG_FILE
    
    # Rust-specific processes
    echo "Rust/Cargo processes:" | tee -a $LOG_FILE
    ps aux | grep -E "(cargo|rustc|cc1plus|wasm-opt)" | grep -v grep | tee -a $LOG_FILE
    echo "" | tee -a $LOG_FILE
    
    # Disk I/O if available
    if command -v iostat &> /dev/null; then
        echo "Disk I/O:" | tee -a $LOG_FILE
        iostat -x 1 1 | tee -a $LOG_FILE
        echo "" | tee -a $LOG_FILE
    fi
    
    echo "----------------------------------------" | tee -a $LOG_FILE
    echo "" | tee -a $LOG_FILE
}

# Initial state
log_system_state

# Monitor continuously
while true; do
    sleep $INTERVAL
    log_system_state
done