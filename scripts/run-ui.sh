#!/bin/bash
#
# Run Essence Wars UI from release build
# Automatically detects platform and runs the appropriate binary
#
# Usage:
#   ./scripts/run-ui.sh          # Run from latest build
#   ./scripts/run-ui.sh --build  # Build first, then run
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Parse arguments
BUILD_FIRST=false
if [ "$1" == "--build" ]; then
    BUILD_FIRST=true
fi

# Detect platform
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    PLATFORM="linux"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    PLATFORM="windows"
else
    log_warn "Unknown platform: $OSTYPE, assuming Linux"
    PLATFORM="linux"
fi

log_info "Detected platform: $PLATFORM"

# Build if requested
if [ "$BUILD_FIRST" == true ]; then
    log_info "Building application..."
    if [ "$PLATFORM" == "linux" ]; then
        "$SCRIPT_DIR/build-linux.sh" --appimage
    else
        "$SCRIPT_DIR/build-windows.sh"
    fi
fi

# Run application
if [ "$PLATFORM" == "linux" ]; then
    APPIMAGE="$PROJECT_ROOT/target/release/bundle/appimage/essence-wars-ui_0.1.0_amd64.AppImage"
    
    if [ ! -f "$APPIMAGE" ]; then
        log_error "AppImage not found at: $APPIMAGE"
        log_info "Build with: ./scripts/build-linux.sh --appimage"
        exit 1
    fi
    
    log_info "Running AppImage..."
    chmod +x "$APPIMAGE"
    "$APPIMAGE"
    
elif [ "$PLATFORM" == "windows" ]; then
    EXE="$PROJECT_ROOT/target/x86_64-pc-windows-gnu/release/essence-wars-ui.exe"
    
    if [ ! -f "$EXE" ]; then
        log_error "Windows executable not found at: $EXE"
        log_info "Build with: ./scripts/build-windows.sh"
        exit 1
    fi
    
    log_info "Running Windows executable..."
    "$EXE"
fi
