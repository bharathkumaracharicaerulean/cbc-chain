#!/usr/bin/env python3
import time
import json
import urllib.request
import urllib.error
import os

PROM_PORT_START = 9615
PROM_PORT_END = 9635
TARGETS_FILE = "/monitoring/targets.json"
HOST = "host.docker.internal"

def check_port(port):
    url = f"http://{HOST}:{port}/metrics"
    try:
        req = urllib.request.Request(url, method="GET")
        with urllib.request.urlopen(req, timeout=0.5) as response:
            if response.status == 200:
                return True
    except Exception:
        pass
    return False

def get_node_name(port):
    # Try to resolve node name via RPC (port + 329)
    rpc_port = port + 329
    url = f"http://{HOST}:{rpc_port}"
    data = json.dumps({"jsonrpc": "2.0", "method": "system_localPeerId", "params": [], "id": 1}).encode("utf-8")
    try:
        req = urllib.request.Request(
            url, 
            data=data, 
            headers={"Content-Type": "application/json"},
            method="POST"
        )
        with urllib.request.urlopen(req, timeout=0.5) as response:
            res_data = json.loads(response.read().decode("utf-8"))
            if "result" in res_data:
                peer_id = res_data["result"]
                short_peer = peer_id[-6:]
                
                # Check for standard defaults
                if port == 9615: return "alice"
                if port == 9616: return "bob"
                if port == 9617: return "charlie"
                return f"node_{port}_{short_peer}"
    except Exception:
        pass
    
    # Fallback mappings for default nodes if RPC fails
    if port == 9615: return "alice"
    if port == 9616: return "bob"
    if port == 9617: return "charlie"
    return f"node_{port}"

def main():
    print("Starting CBC node discovery service...", flush=True)
    # Ensure directory exists
    os.makedirs(os.path.dirname(TARGETS_FILE), exist_ok=True)
    
    while True:
        active_targets = []
        for port in range(PROM_PORT_START, PROM_PORT_END + 1):
            if check_port(port):
                node_name = get_node_name(port)
                target_entry = {
                    "targets": [f"{HOST}:{port}"],
                    "labels": {
                        "job": "cbc-node",
                        "node": node_name,
                        "port": str(port)
                    }
                }
                active_targets.append(target_entry)
        
        # Write to target file
        try:
            with open(TARGETS_FILE, "w") as f:
                json.dump(active_targets, f, indent=2)
            # print(f"Discovered {len(active_targets)} active nodes.", flush=True)
        except Exception as e:
            print(f"Error writing targets file: {e}", flush=True)
            
        time.sleep(5)

if __name__ == "__main__":
    main()
