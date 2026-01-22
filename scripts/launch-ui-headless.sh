#!/usr/bin/env bash
# Launch Essence Wars UI headlessly using Xvfb
#
# This script starts the Tauri app in a virtual X server, allowing:
# - Screenshots to be captured without a visible window
# - Claude Code to playtest games autonomously
# - Running on servers without a display
#
# Usage:
#   ./scripts/launch-ui-headless.sh          # Start headless UI
#   ./scripts/launch-ui-headless.sh --build  # Build release and start
#
# Requirements:
#   - Xvfb (sudo apt install xvfb)
#   - fluxbox or similar WM (sudo apt install fluxbox)
#   - pnpm (npm install -g pnpm)

set -e

# Configuration
DISPLAY_NUM="${DISPLAY_NUM:-99}"
export DISPLAY=":${DISPLAY_NUM}"  # Export immediately to override any inherited DISPLAY
unset WAYLAND_DISPLAY  # Ensure we don't use Wayland
XVFB_ARGS="-screen 0 1024x768x24"
UI_DIR="crates/essence-wars-ui"
PORT=9999
TIMEOUT="${TIMEOUT:-300}"  # 5 minute default timeout

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Track PIDs for cleanup
XVFB_PID=""
WM_PID=""
APP_PID=""

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Cleaning up...${NC}"

    # Kill app first
    if [ -n "$APP_PID" ] && kill -0 "$APP_PID" 2>/dev/null; then
        echo "Stopping app (PID $APP_PID)"
        kill "$APP_PID" 2>/dev/null || true
        wait "$APP_PID" 2>/dev/null || true
    fi

    # Kill window manager
    if [ -n "$WM_PID" ] && kill -0 "$WM_PID" 2>/dev/null; then
        echo "Stopping window manager (PID $WM_PID)"
        kill "$WM_PID" 2>/dev/null || true
    fi

    # Kill Xvfb
    if [ -n "$XVFB_PID" ] && kill -0 "$XVFB_PID" 2>/dev/null; then
        echo "Stopping Xvfb (PID $XVFB_PID)"
        kill "$XVFB_PID" 2>/dev/null || true
    fi

    # Clean up lock file
    rm -f "/tmp/.X${DISPLAY_NUM}-lock" 2>/dev/null || true

    echo -e "${GREEN}Cleanup complete${NC}"
}

trap cleanup EXIT INT TERM

# Check if we're in the project root
if [ ! -d "$UI_DIR" ]; then
    echo -e "${RED}Error: Must run from project root (ai-cardgame)${NC}"
    exit 1
fi

# Check dependencies
check_dependency() {
    if ! command -v "$1" &>/dev/null; then
        echo -e "${RED}Error: $1 not found. Install with: $2${NC}"
        exit 1
    fi
}

check_dependency Xvfb "sudo apt install xvfb"
check_dependency pnpm "npm install -g pnpm"

# Check for a window manager
WM_CMD=""
if command -v fluxbox &>/dev/null; then
    WM_CMD="fluxbox"
elif command -v openbox &>/dev/null; then
    WM_CMD="openbox"
elif command -v fvwm &>/dev/null; then
    WM_CMD="fvwm"
else
    echo -e "${YELLOW}Warning: No window manager found (fluxbox, openbox, fvwm).${NC}"
    echo -e "${YELLOW}Install one with: sudo apt install fluxbox${NC}"
    echo -e "${YELLOW}Continuing without WM (may cause issues)...${NC}"
fi

# Kill any existing Xvfb on our display
pkill -f "Xvfb.*:${DISPLAY_NUM}" 2>/dev/null || true
rm -f "/tmp/.X${DISPLAY_NUM}-lock" 2>/dev/null || true
sleep 1

echo -e "${BLUE}Starting Essence Wars UI (headless)${NC}"
echo -e "Display: ${DISPLAY}"
echo -e "Screenshot port: ${PORT}"
echo ""

# Start Xvfb
echo -e "${GREEN}Starting Xvfb on ${DISPLAY}...${NC}"
Xvfb "$DISPLAY" $XVFB_ARGS &
XVFB_PID=$!
sleep 1

# Verify Xvfb started
if ! kill -0 "$XVFB_PID" 2>/dev/null; then
    echo -e "${RED}Failed to start Xvfb${NC}"
    exit 1
fi
echo "Xvfb running (PID $XVFB_PID)"

# DISPLAY already exported at top of script

# Start window manager if available
if [ -n "$WM_CMD" ]; then
    echo -e "${GREEN}Starting window manager ($WM_CMD)...${NC}"
    $WM_CMD &
    WM_PID=$!
    sleep 1
    echo "Window manager running (PID $WM_PID)"
fi

# Navigate to UI directory
cd "$UI_DIR"

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo -e "${BLUE}Installing dependencies...${NC}"
    pnpm install
fi

# Build if requested or if no release binary exists
# Note: tauri builds to workspace root target/, not src-tauri/target/
RELEASE_BINARY="../../target/release/essence-wars-ui"
if [ "$1" = "--build" ] || [ ! -f "$RELEASE_BINARY" ]; then
    echo -e "${BLUE}Building release binary...${NC}"
    pnpm tauri build 2>&1 | tail -20
    echo -e "${GREEN}Build complete${NC}"
else
    echo -e "${GREEN}Using existing binary: $RELEASE_BINARY${NC}"
fi

# Find the binary (tauri builds to workspace root target/, not src-tauri/target/)
BINARY=""
if [ -f "../../target/release/essence-wars-ui" ]; then
    BINARY="../../target/release/essence-wars-ui"
elif [ -f "src-tauri/target/release/essence-wars-ui" ]; then
    BINARY="src-tauri/target/release/essence-wars-ui"
elif [ -f "../../target/debug/essence-wars-ui" ]; then
    BINARY="../../target/debug/essence-wars-ui"
elif [ -f "src-tauri/target/debug/essence-wars-ui" ]; then
    BINARY="src-tauri/target/debug/essence-wars-ui"
else
    echo -e "${RED}Error: Could not find essence-wars-ui binary${NC}"
    echo "Searched in: ../../target/release/, src-tauri/target/release/"
    echo "Try running with --build flag"
    exit 1
fi

# Start the app
echo -e "${GREEN}Starting Essence Wars UI...${NC}"
"$BINARY" &
APP_PID=$!
echo "App starting (PID $APP_PID)"

# Wait for screenshot server to be ready
echo -e "${BLUE}Waiting for screenshot server on port ${PORT}...${NC}"
READY=0
for i in {1..30}; do
    if curl -s "http://127.0.0.1:${PORT}/health" >/dev/null 2>&1; then
        READY=1
        break
    fi
    echo -n "."
    sleep 1
done
echo ""

if [ $READY -eq 0 ]; then
    echo -e "${RED}Screenshot server failed to start${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}=== Essence Wars UI is running headlessly ===${NC}"
echo ""
echo "Endpoints:"
echo "  - Health:     http://127.0.0.1:${PORT}/health"
echo "  - Screenshot: http://127.0.0.1:${PORT}/screenshot"
echo "  - Sync State: http://127.0.0.1:${PORT}/sync_state (POST)"
echo "  - Get State:  http://127.0.0.1:${PORT}/synced_state (GET)"
echo ""
echo "Test screenshot: curl http://127.0.0.1:${PORT}/screenshot -o test.png"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
echo ""

# Keep running
wait "$APP_PID"
