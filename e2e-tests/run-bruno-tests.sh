#!/bin/bash
# Run Bruno E2E tests against all Oicana example services

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
SERVICES="${1:-aspnet,axum,fastapi,nestjs}"
BRUNO_DIR="$(dirname "$0")/bruno"

echo "🚀 Running Bruno E2E tests for Oicana example services"
echo ""

# Check if Bruno CLI is installed
if ! command -v bru &> /dev/null; then
    echo -e "${RED}✗ Bruno CLI not found!${NC}"
    echo "Please install Bruno CLI: npm install -g @usebruno/cli"
    exit 1
fi

echo -e "${GREEN}✓ Bruno CLI found${NC}"
echo ""

# Parse services
IFS=',' read -ra SERVICE_ARRAY <<< "$SERVICES"

# Track results
PASSED=()
FAILED=()

for service in "${SERVICE_ARRAY[@]}"; do
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "Testing ${YELLOW}${service}${NC} service"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Check if environment exists
    if [ ! -f "$BRUNO_DIR/environments/${service}.bru" ]; then
        echo -e "${RED}✗ Environment file not found for ${service}${NC}"
        FAILED+=("$service")
        continue
    fi

    # Run Bruno tests
    pushd $BRUNO_DIR
    if bru run --env "$service" --reporter-html results-${service}.html; then
        echo ""
        echo -e "${GREEN}✓ All tests passed for ${service}${NC}"
        PASSED+=("$service")
    else
        echo ""
        echo -e "${RED}✗ Tests failed for ${service}${NC}"
        FAILED+=("$service")
    fi
    popd

    echo ""
done

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ ${#PASSED[@]} -gt 0 ]; then
    echo -e "${GREEN}✓ Passed (${#PASSED[@]}):${NC} ${PASSED[*]}"
fi

if [ ${#FAILED[@]} -gt 0 ]; then
    echo -e "${RED}✗ Failed (${#FAILED[@]}):${NC} ${FAILED[*]}"
    echo ""
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 All tests passed!${NC}"
