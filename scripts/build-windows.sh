#!/bin/bash
#
# Build Essence Wars for Windows from WSL2/Linux
#
# Usage:
#   ./scripts/build-windows.sh              # Build unsigned
#   ./scripts/build-windows.sh --sign       # Build and sign (requires certificate)
#   ./scripts/build-windows.sh --setup-cert # Generate self-signed certificate
#
# Prerequisites:
#   - Rust with x86_64-pc-windows-gnu target
#   - MinGW cross-compiler (x86_64-w64-mingw32-gcc)
#   - NSIS (makensis)
#   - pnpm
#   - osslsigncode (for signing)
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
UI_DIR="$PROJECT_ROOT/crates/essence-wars-ui"
TARGET_DIR="$PROJECT_ROOT/target/x86_64-pc-windows-gnu/release"
CERT_DIR="$PROJECT_ROOT/.certs"
CERT_FILE="$CERT_DIR/codesign.pfx"
CERT_PASS_FILE="$CERT_DIR/codesign.pass"

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

    # Check Rust target
    if ! rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
        missing+=("rustup target add x86_64-pc-windows-gnu")
    fi

    # Check MinGW
    if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        missing+=("sudo apt install mingw-w64")
    fi

    # Check NSIS
    if ! command -v makensis &> /dev/null; then
        missing+=("sudo apt install nsis")
    fi

    # Check pnpm
    if ! command -v pnpm &> /dev/null; then
        missing+=("npm install -g pnpm")
    fi

    if [ ${#missing[@]} -gt 0 ]; then
        log_error "Missing prerequisites. Run these commands:"
        for cmd in "${missing[@]}"; do
            echo "  $cmd"
        done
        exit 1
    fi

    log_success "All prerequisites satisfied"
}

check_signing_prerequisites() {
    if ! command -v osslsigncode &> /dev/null; then
        log_warn "osslsigncode not installed. Installing..."
        sudo apt-get update && sudo apt-get install -y osslsigncode
    fi

    if [ ! -f "$CERT_FILE" ]; then
        log_error "Certificate not found at $CERT_FILE"
        log_info "Run './scripts/build-windows.sh --setup-cert' to create a self-signed certificate"
        log_info "Or place your purchased certificate at $CERT_FILE"
        exit 1
    fi

    if [ ! -f "$CERT_PASS_FILE" ]; then
        log_error "Certificate password file not found at $CERT_PASS_FILE"
        exit 1
    fi
}

setup_self_signed_cert() {
    log_info "Setting up self-signed code signing certificate..."

    mkdir -p "$CERT_DIR"

    # Generate private key and certificate
    log_info "Generating certificate (valid for 10 years)..."

    openssl req -x509 -newkey rsa:4096 \
        -keyout "$CERT_DIR/codesign.key" \
        -out "$CERT_DIR/codesign.crt" \
        -sha256 -days 3650 \
        -nodes \
        -subj "/CN=Essence Wars/O=Chris/C=DE" \
        -addext "extendedKeyUsage=codeSigning" \
        -addext "keyUsage=digitalSignature"

    # Generate random password
    local password=$(openssl rand -base64 32)
    echo "$password" > "$CERT_PASS_FILE"
    chmod 600 "$CERT_PASS_FILE"

    # Convert to PFX
    log_info "Converting to PFX format..."
    openssl pkcs12 -export \
        -out "$CERT_FILE" \
        -inkey "$CERT_DIR/codesign.key" \
        -in "$CERT_DIR/codesign.crt" \
        -password "pass:$password"

    # Secure the files
    chmod 600 "$CERT_DIR"/*

    # Add to .gitignore if not already there
    if ! grep -q "^\.certs/" "$PROJECT_ROOT/.gitignore" 2>/dev/null; then
        echo -e "\n# Code signing certificates\n.certs/" >> "$PROJECT_ROOT/.gitignore"
        log_info "Added .certs/ to .gitignore"
    fi

    log_success "Self-signed certificate created at $CERT_FILE"
    log_warn "Note: Self-signed certificates will still trigger Windows SmartScreen warnings"
    log_info "For trusted signing, purchase a certificate from DigiCert, Sectigo, etc."
}

sign_executable() {
    local exe_path="$1"
    local password=$(cat "$CERT_PASS_FILE")

    log_info "Signing $exe_path..."

    osslsigncode sign \
        -pkcs12 "$CERT_FILE" \
        -pass "$password" \
        -n "Essence Wars" \
        -i "https://github.com/your-repo/essence-wars" \
        -t "http://timestamp.digicert.com" \
        -in "$exe_path" \
        -out "${exe_path}.signed"

    mv "${exe_path}.signed" "$exe_path"
    log_success "Signed: $exe_path"
}

build_windows() {
    local do_sign=false

    if [ "$1" == "--sign" ]; then
        do_sign=true
        check_signing_prerequisites
    fi

    check_prerequisites

    cd "$UI_DIR"

    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        log_info "Installing npm dependencies..."
        pnpm install
    fi

    log_info "Building Essence Wars for Windows..."
    log_info "This will compile: Frontend (Svelte) → Rust Backend → NSIS Installer"
    echo ""

    # Set NSIS path for Linux
    export TAURI_NSIS_MAKENSIS_PATH=/usr/bin/makensis

    # Build
    pnpm tauri build --target x86_64-pc-windows-gnu

    if [ "$do_sign" == true ]; then
        log_info "Signing executables..."

        # Sign the main executable
        sign_executable "$TARGET_DIR/essence-wars-ui.exe"

        # The NSIS installer embeds the exe, so we need to rebuild after signing
        # Or sign the installer itself
        local installer="$TARGET_DIR/bundle/nsis/essence-wars-ui_0.1.0_x64-setup.exe"
        if [ -f "$installer" ]; then
            sign_executable "$installer"
        fi
    fi

    echo ""
    log_success "Build complete!"
    echo ""
    echo "Output files:"
    echo "  Installer: $TARGET_DIR/bundle/nsis/essence-wars-ui_0.1.0_x64-setup.exe"
    echo "  Executable: $TARGET_DIR/essence-wars-ui.exe"
    echo ""

    if [ "$do_sign" == false ]; then
        log_warn "Executable is unsigned. Windows Defender may show warnings."
        log_info "Run with --sign to sign the executable (requires certificate)"
    fi
}

show_help() {
    echo "Build Essence Wars for Windows from WSL2/Linux"
    echo ""
    echo "Usage:"
    echo "  $0              Build unsigned Windows installer"
    echo "  $0 --sign       Build and sign with certificate"
    echo "  $0 --setup-cert Generate self-signed certificate"
    echo "  $0 --help       Show this help"
    echo ""
    echo "Certificate location: $CERT_DIR/"
    echo ""
    echo "For trusted signing (no SmartScreen warnings):"
    echo "  1. Purchase a code signing certificate from DigiCert, Sectigo, etc."
    echo "  2. Export as PFX and save to $CERT_FILE"
    echo "  3. Save password to $CERT_PASS_FILE"
}

# Main
case "${1:-}" in
    --help|-h)
        show_help
        ;;
    --setup-cert)
        setup_self_signed_cert
        ;;
    --sign)
        build_windows --sign
        ;;
    "")
        build_windows
        ;;
    *)
        log_error "Unknown option: $1"
        show_help
        exit 1
        ;;
esac
