#!/bin/bash

# CBC Monitoring Stack Startup Script

set -e

echo "Starting CBC Monitoring Stack..."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "ERROR: Docker is not running. Please start Docker first."
    exit 1
fi

# Check if docker-compose or docker compose is available
if command -v docker-compose &> /dev/null; then
    DOCKER_COMPOSE="docker-compose"
elif docker compose version &> /dev/null; then
    DOCKER_COMPOSE="docker compose"
else
    echo "ERROR: Docker Compose is not installed. Please install docker-compose or the docker-compose-plugin first."
    exit 1
fi

# Create monitoring directories and initialize targets list if they don't exist
mkdir -p monitoring/grafana/provisioning/{dashboards,datasources}
if [ ! -f "monitoring/targets.json" ]; then
    echo "[]" > monitoring/targets.json
fi

# Parse arguments
FRESH=false
for arg in "$@"; do
  case $arg in
    --fresh|-f)
      FRESH=true
      shift
      ;;
  esac
done

if [ "$FRESH" = true ]; then
    echo "Fresh start requested. Removing old monitoring data (volumes)..."
    $DOCKER_COMPOSE -f docker-compose.monitoring.yml down -v || true
fi

# Clean up any existing stopped containers to avoid conflicts
echo "Cleaning up old containers..."
docker ps -a --filter name=cbc-prometheus --filter name=cbc-grafana -q | xargs -r docker rm 2>/dev/null || true

echo "Starting Prometheus and Grafana..."
$DOCKER_COMPOSE -f docker-compose.monitoring.yml up -d

echo "Waiting for services to start..."
sleep 10

# Check if services are running
if $DOCKER_COMPOSE -f docker-compose.monitoring.yml ps | grep -q "Up"; then
    echo "SUCCESS: Monitoring stack started successfully!"
    echo ""
    echo "Access URLs:"
    echo "   Grafana:    http://localhost:3000 (admin/admin)"
    echo "   Prometheus: http://localhost:9090"
    echo ""
    echo "Next steps:"
    echo "   1. Start your CBC node with: ./target/release/cbc-node --dev --prometheus-external"
    echo "   2. Open Grafana and navigate to 'CBC Consensus Dashboard'"
    echo "   3. If no data appears, check the troubleshooting section in docs/grafana-setup-guide.md"
    echo "   4. To reset all data (e.g. after restarting chain), use: ./start-monitoring.sh --fresh"
    echo ""
    echo "To stop: $DOCKER_COMPOSE -f docker-compose.monitoring.yml down"
else
    echo "ERROR: Failed to start monitoring stack. Check logs with:"
    echo "   $DOCKER_COMPOSE -f docker-compose.monitoring.yml logs"
    exit 1
fi