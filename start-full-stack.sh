#!/bin/bash

# Start the full game engine stack:
# 1. ACE-Step music generation service
# 2. MCP Server
# 3. Game Engine Editor

set -e

PROJECT_DIR="/home/k/game-engine"
ACE_STEP_DIR="$PROJECT_DIR/ACE-Step"
PORT=7865

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

cleanup() {
    echo -e "${YELLOW}Stopping all services...${NC}"
    jobs -p | xargs -r kill 2>/dev/null || true
    exit 0
}

trap cleanup SIGINT SIGTERM

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Starting Game Engine Full Stack           ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""

cd "$PROJECT_DIR"

# Start ACE-Step
echo -e "${BLUE}[1/3] Starting ACE-Step music generation service...${NC}"
if [ -d "$ACE_STEP_DIR/venv" ]; then
    (
        cd "$ACE_STEP_DIR"
        source venv/bin/activate
        acestep --port $PORT --device_id 0 --torch_compile true
    ) &
    ACE_STEP_PID=$!
    echo -e "${GREEN}✓ ACE-Step started (PID: $ACE_STEP_PID)${NC}"
    sleep 2
else
    echo -e "${YELLOW}⚠ ACE-Step venv not found, skipping...${NC}"
    echo "  Setup with: cd $ACE_STEP_DIR && python -m venv venv && source venv/bin/activate && pip install -e ."
fi

echo ""

# Start MCP Server
echo -e "${BLUE}[2/3] Starting MCP Server...${NC}"
cargo run --bin engine-mcp-server --quiet &
MCP_PID=$!
echo -e "${GREEN}✓ MCP Server started (PID: $MCP_PID)${NC}"
sleep 2

echo ""

# Start Editor
echo -e "${BLUE}[3/3] Starting Game Engine Editor...${NC}"
cargo run --bin editor &
EDITOR_PID=$!
echo -e "${GREEN}✓ Editor started (PID: $EDITOR_PID)${NC}"

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  All services running!                     ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
echo ""
echo "Services:"
echo "  • Editor:          Running locally"
echo "  • MCP Server:      Listening on stdio"
echo "  • ACE-Step:        http://localhost:$PORT"
echo ""
echo "Press Ctrl+C to stop all services"
echo ""

# Wait for all processes
wait
