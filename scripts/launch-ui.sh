#!/usr/bin/env bash
# Launch Essence Wars UI (Tauri app) for development/debugging

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

UI_DIR="crates/essence-wars-ui"

echo -e "${BLUE}🚀 Launching Essence Wars UI (Tauri)${NC}"

# Check if we're in the project root
if [ ! -d "$UI_DIR" ]; then
    echo -e "${RED}❌ Error: Must run from project root (ai-cardgame)${NC}"
    exit 1
fi

# Navigate to UI directory
cd "$UI_DIR"

# Check if pnpm is installed
if ! command -v pnpm &> /dev/null; then
    echo -e "${RED}❌ Error: pnpm not found. Install with: npm install -g pnpm${NC}"
    exit 1
fi

# Install dependencies if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    echo -e "${BLUE}📦 Installing dependencies...${NC}"
    pnpm install
fi

# Launch Tauri in dev mode
echo -e "${GREEN}✨ Starting Tauri dev server...${NC}"
echo -e "${BLUE}Press Ctrl+C to stop${NC}"
echo ""

pnpm tauri dev
