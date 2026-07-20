#!/bin/bash
# stop-monitoring.sh - Tear down the CBC Monitoring Stack (Prometheus, Grafana, Discovery)

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}      Stopping CBC Monitoring Stack     ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if docker-compose or docker compose is available
if command -v docker-compose &> /dev/null; then
    DOCKER_COMPOSE="docker-compose"
elif docker compose version &> /dev/null; then
    DOCKER_COMPOSE="docker compose"
else
    echo "ERROR: Docker Compose is not installed. Please install docker-compose or the docker-compose-plugin first."
    exit 1
fi

echo "Stopping services and cleaning up containers..."
$DOCKER_COMPOSE -f docker-compose.monitoring.yml down "$@"

# Reset targets file to clean empty state
if [ -f "monitoring/targets.json" ]; then
    echo "[]" > monitoring/targets.json
    echo "Reset targets database."
fi

echo ""
echo -e "${GREEN}✓ Monitoring stack stopped successfully!${NC}"
echo ""
