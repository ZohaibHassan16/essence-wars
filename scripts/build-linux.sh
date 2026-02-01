#!/bin/bash
#
# Build Essence Wars for Linux
#
# Usage:
#   ./scripts/build-linux.sh              # Build AppImage
#   ./scripts/build-linux.sh --deb        # Build .deb package
#   ./scripts/build-linux.sh --all        # Build both
#
# Prerequisites:
#   - Rust
#   - pnpm
#   - webkit2gtk (for Tauri)
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
UI_DIR="$PROJECT_ROOT/crates/essence-wars-ui"
TARGET_DIR="$PROJECT_ROOT/target/release"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

check_prerequisites() {
    log_info "Checking prerequisites..."

    local missing=()

    # Check pnpm
    if ! command -v pnpm &> /dev/null; then
        missing+=("pnpm (npm install -g pnpm)")
    fi

    # Check webkit2gtk
    if ! pkg-config --exists webkit2gtk-4.1; then
        if ! pkg-config --exists webkit2gtk-4.0; then
            missing+=("webkit2gtk-4.1 or webkit2gtk-4.0 (sudo apt install libwebkit2gtk-4.1-dev)")
        fi
    fi

    # Check other Tauri dependencies
    if ! pkg-config --exists gtk+-3.0; then
        missing+=("gtk+-3.0 (sudo apt install libgtk-3-dev)")
    fi

    if [ ${#missing[@]} -gt 0 ]; then
        log_error "Missing prerequisites:"
        for item in "${missing[@]}"; do
            echo "  - $item"
        done
        log_info "\nInstall all Tauri dependencies with:"
        echo "  sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev"
        exit 1
    fi

    log_success "All prerequisites satisfied"
}

build_frontend() {
    log_info "Building frontend..."
    cd "$UI_DIR"
    
    if [ ! -d "node_modules" ]; then
        log_info "Installing pnpm dependencies..."
        pnpm install
    fi

    pnpm build
    log_success "Frontend built successfully"
}

build_tauri() {
    local bundle_type="$1"
    
    log_info "Building Tauri application..."
    cd "$UI_DIR"

    case "$bundle_type" in
        "appimage")
            log_info "Building AppImage..."
            cargo tauri build --bundles appimage
            ;;
        "deb")
            log_info "Building .deb package..."
            cargo tauri build --bundles deb
            ;;
        "all")
            log_info "Building all bundles..."
            cargo tauri build
            ;;
        *)
            log_info "Building default bundles..."
            cargo tauri build
            ;;
    esac

    log_success "Tauri build completed"
}

show_artifacts() {
    log_info "Build artifacts:"
    echo ""
    
    local bundle_dir="$UI_DIR/src-tauri/target/release/bundle"
    
    if [ -d "$bundle_dir/appimage" ]; then
        echo "AppImage:"
        find "$bundle_dir/appimage" -name "*.AppImage" -exec echo "  {}" \;
    fi
    
    if [ -d "$bundle_dir/deb" ]; then
        echo "Debian package:"
        find "$bundle_dir/deb" -name "*.deb" -exec echo "  {}" \;
    fi
    
    echo ""
    log_info "To test the application, run:"
    if [ -d "$bundle_dir/appimage" ]; then
        local appimage=$(find "$bundle_dir/appimage" -name "*.AppImage" | head -n1)
        if [ -n "$appimage" ]; then
            echo "  chmod +x \"$appimage\""
            echo "  \"$appimage\""
        fi
    fi
}

reveal_artifacts() {
    local bundle_dir="$UI_DIR/src-tauri/target/release/bundle"
    
    if command -v xdg-open &> /dev/null; then
        log_info "Opening bundle directory..."
        xdg-open "$bundle_dir" 2>/dev/null || true
    else
        log_info "Bundle directory: $bundle_dir"
    fi
}

main() {
    local bundle_type="all"
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --appimage)
                bundle_type="appimage"
                shift
                ;;
            --deb)
                bundle_type="deb"
                shift
                ;;
            --all)
                bundle_type="all"
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --appimage    Build AppImage only"
                echo "  --deb         Build .deb package only"
                echo "  --all         Build all bundles (default)"
                echo "  --help        Show this help"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done

    log_info "Starting Linux build process..."
    echo ""

    check_prerequisites
    build_frontend
    build_tauri "$bundle_type"
    
    echo ""
    log_success "Build completed successfully!"
    echo ""
    
    show_artifacts
    reveal_artifacts
}

main "$@"
