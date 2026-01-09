#!/bin/bash

# Start ACE Step and open in browser

set -e

ACE_STEP_DIR="/home/k/game-engine/ACE-Step"
PORT=7865
MAX_RETRIES=30
RETRY_DELAY=1

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}Starting ACE-Step music generation service...${NC}"

# Check if ACE-Step directory exists
if [ ! -d "$ACE_STEP_DIR" ]; then
    echo -e "${RED}Error: ACE-Step directory not found at $ACE_STEP_DIR${NC}"
    exit 1
fi

cd "$ACE_STEP_DIR"

# Check if virtual environment exists
if [ ! -d "venv" ]; then
    echo -e "${RED}Error: Python virtual environment not found at $ACE_STEP_DIR/venv${NC}"
    echo "Please set up the virtual environment first:"
    echo "  cd $ACE_STEP_DIR"
    echo "  python -m venv venv"
    echo "  source venv/bin/activate"
    echo "  pip install -e ."
    exit 1
fi

# Activate virtual environment
source venv/bin/activate

# Configure PyTorch memory allocator for better memory management
export PYTORCH_CUDA_ALLOC_CONF=expandable_segments:True,max_split_size_mb:512
export PYTORCH_NO_CUDA_MEMORY_CACHING=0

# Enable memory efficient attention
export ATTN_BACKEND=flash-attn

# Start ACE-Step service in background with full GPU mode
# For best performance, use APG mode (not dual CFG guidance)
acestep --port $PORT --device_id 0 --torch_compile false --cpu_offload false &
ACE_STEP_PID=$!

echo -e "${BLUE}ACE-Step started with PID $ACE_STEP_PID${NC}"

# Wait for service to be ready
echo -e "${BLUE}Waiting for ACE-Step to be ready...${NC}"
RETRY_COUNT=0

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -s "http://localhost:$PORT/health" > /dev/null 2>&1; then
        echo -e "${GREEN}ACE-Step is ready!${NC}"
        break
    fi
    RETRY_COUNT=$((RETRY_COUNT + 1))
    sleep $RETRY_DELAY
done

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
    echo -e "${RED}Warning: ACE-Step didn't respond within ${MAX_RETRIES}s, but attempting to open browser anyway${NC}"
fi

# Open in browser
BROWSER_URL="http://localhost:$PORT"
echo -e "${BLUE}Opening $BROWSER_URL in browser...${NC}"

# Try different methods to open browser
if command -v xdg-open > /dev/null; then
    xdg-open "$BROWSER_URL"
elif command -v open > /dev/null; then
    open "$BROWSER_URL"
elif command -v firefox > /dev/null; then
    firefox "$BROWSER_URL" &
elif command -v google-chrome > /dev/null; then
    google-chrome "$BROWSER_URL" &
else
    echo -e "${BLUE}No browser found, but ACE-Step is available at: $BROWSER_URL${NC}"
fi

echo -e "${GREEN}ACE-Step is running. Press Ctrl+C to stop.${NC}"
echo -e "${BLUE}API Documentation: $BROWSER_URL/docs${NC}"
echo ""
echo "To stop ACE-Step:"
echo "  kill $ACE_STEP_PID"
echo "  or press Ctrl+C"

# Wait for the process
wait $ACE_STEP_PID
