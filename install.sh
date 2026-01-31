#!/usr/bin/env bash
#
# RevGame Engine Installer for Linux
# Installs dependencies and builds the Bevy game client
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

# State
MISSING_DEPS=()
PKG_MANAGER=""
BUILD_TYPE="graphics"

log_info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step()  { echo -e "\n${BLUE}${BOLD}[$1]${NC} $2"; }

prompt_yn() {
    local prompt="$1"
    local default="${2:-y}"
    local response

    if [[ "$default" == "y" ]]; then
        read -p "$prompt [Y/n]: " -r response
        [[ -z "$response" || "$response" =~ ^[Yy]$ ]]
    else
        read -p "$prompt [y/N]: " -r response
        [[ "$response" =~ ^[Yy]$ ]]
    fi
}

prompt_choice() {
    local prompt="$1"
    local default="$2"
    shift 2
    local options=("$@")
    local choice

    echo ""
    for i in "${!options[@]}"; do
        echo "  [$((i+1))] ${options[$i]}"
    done
    echo ""

    while true; do
        read -p "$prompt [$default]: " -r choice
        choice="${choice:-$default}"
        if [[ "$choice" =~ ^[0-9]+$ ]] && (( choice >= 1 && choice <= ${#options[@]} )); then
            echo "$choice"
            return
        fi
        echo "Please enter a number between 1 and ${#options[@]}"
    done
}

show_banner() {
    echo -e "${BOLD}"
    echo "================================================================================"
    echo "                       RevGame Engine Installer"
    echo "================================================================================"
    echo -e "${NC}"
    echo "This script will install dependencies and build RevGame for Linux."
    echo ""
}

detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        case "$ID" in
            ubuntu|debian|linuxmint|pop)
                PKG_MANAGER="apt"
                ;;
            fedora|rhel|centos|rocky|alma)
                PKG_MANAGER="dnf"
                ;;
            arch|manjaro|endeavouros)
                PKG_MANAGER="pacman"
                ;;
            opensuse*)
                PKG_MANAGER="zypper"
                ;;
            *)
                PKG_MANAGER="unknown"
                ;;
        esac
        log_info "Detected: $PRETTY_NAME (package manager: $PKG_MANAGER)"
    else
        PKG_MANAGER="unknown"
        log_warn "Could not detect Linux distribution"
    fi
}

check_command() {
    local name="$1"
    local cmd="$2"

    echo -n "  $name: "
    if command -v "$cmd" &> /dev/null; then
        local version
        version=$("$cmd" --version 2>&1 | head -1 || echo "unknown")
        echo -e "${GREEN}FOUND${NC} ($version)"
        return 0
    else
        echo -e "${RED}NOT FOUND${NC}"
        return 1
    fi
}

check_dependencies() {
    log_step "1/4" "Checking dependencies..."

    MISSING_DEPS=()

    if ! check_command "Git" "git"; then
        MISSING_DEPS+=("git")
    fi

    if ! check_command "Rust" "rustc"; then
        MISSING_DEPS+=("rust")
    fi

    if ! check_command "Cargo" "cargo"; then
        if [[ ! " ${MISSING_DEPS[*]} " =~ " rust " ]]; then
            MISSING_DEPS+=("rust")
        fi
    fi

    # Check Bevy dependencies for graphics builds
    if [[ "$BUILD_TYPE" == "graphics" ]]; then
        echo ""
        echo "  Bevy system libraries:"

        local bevy_missing=false

        # Check for pkg-config
        if ! command -v pkg-config &> /dev/null; then
            echo -e "    pkg-config: ${RED}NOT FOUND${NC}"
            bevy_missing=true
        else
            echo -e "    pkg-config: ${GREEN}FOUND${NC}"
        fi

        # Check for alsa
        if ! pkg-config --exists alsa 2>/dev/null; then
            echo -e "    alsa: ${RED}NOT FOUND${NC}"
            bevy_missing=true
        else
            echo -e "    alsa: ${GREEN}FOUND${NC}"
        fi

        # Check for libudev
        if ! pkg-config --exists libudev 2>/dev/null; then
            echo -e "    libudev: ${RED}NOT FOUND${NC}"
            bevy_missing=true
        else
            echo -e "    libudev: ${GREEN}FOUND${NC}"
        fi

        if $bevy_missing; then
            MISSING_DEPS+=("bevy-deps")
        fi
    fi

    echo ""
    if [ ${#MISSING_DEPS[@]} -eq 0 ]; then
        log_info "All dependencies are installed!"
        return 0
    else
        log_warn "Missing dependencies: ${MISSING_DEPS[*]}"
        return 1
    fi
}

install_rust() {
    log_info "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

    # Source cargo env
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi

    # Verify installation
    if command -v rustc &> /dev/null; then
        log_info "Rust installed successfully: $(rustc --version)"
    else
        log_error "Rust installation failed"
        exit 1
    fi
}

install_bevy_deps() {
    log_info "Installing Bevy system dependencies..."

    case "$PKG_MANAGER" in
        apt)
            sudo apt update
            sudo apt install -y \
                libasound2-dev \
                libudev-dev \
                pkg-config \
                libwayland-dev \
                libxkbcommon-dev \
                libx11-dev
            ;;
        dnf)
            sudo dnf install -y \
                alsa-lib-devel \
                systemd-devel \
                pkgconf \
                wayland-devel \
                libxkbcommon-devel \
                libX11-devel
            ;;
        pacman)
            sudo pacman -S --noconfirm \
                alsa-lib \
                systemd \
                pkgconf \
                wayland \
                libxkbcommon
            ;;
        zypper)
            sudo zypper install -y \
                alsa-devel \
                systemd-devel \
                pkg-config \
                wayland-devel \
                libxkbcommon-devel
            ;;
        *)
            log_error "Unknown package manager. Please install Bevy dependencies manually:"
            echo "  - ALSA development libraries"
            echo "  - udev development libraries"
            echo "  - pkg-config"
            echo "  - Wayland development libraries"
            echo "  - xkbcommon development libraries"
            exit 1
            ;;
    esac

    log_info "Bevy dependencies installed successfully"
}

install_git() {
    log_info "Installing Git..."

    case "$PKG_MANAGER" in
        apt)
            sudo apt update && sudo apt install -y git
            ;;
        dnf)
            sudo dnf install -y git
            ;;
        pacman)
            sudo pacman -S --noconfirm git
            ;;
        zypper)
            sudo zypper install -y git
            ;;
        *)
            log_error "Unknown package manager. Please install Git manually."
            exit 1
            ;;
    esac
}

install_dependencies() {
    log_step "2/4" "Installing missing dependencies..."

    if [ ${#MISSING_DEPS[@]} -eq 0 ]; then
        log_info "No dependencies to install"
        return
    fi

    echo ""
    echo "The following will be installed:"
    for dep in "${MISSING_DEPS[@]}"; do
        case "$dep" in
            git)
                echo "  - Git (version control)"
                ;;
            rust)
                echo "  - Rust toolchain (via rustup)"
                ;;
            bevy-deps)
                echo "  - Bevy system libraries (audio, graphics)"
                ;;
        esac
    done
    echo ""

    if ! prompt_yn "Proceed with installation?"; then
        log_warn "Installation cancelled"
        exit 0
    fi

    echo ""

    for dep in "${MISSING_DEPS[@]}"; do
        case "$dep" in
            git)
                install_git
                ;;
            rust)
                install_rust
                ;;
            bevy-deps)
                install_bevy_deps
                ;;
        esac
    done

    log_info "All dependencies installed!"
}

choose_build_type() {
    log_step "3/4" "Build configuration"

    local choice
    choice=$(prompt_choice "Select build type:" "1" \
        "Full build with graphics (Bevy) - for playing/testing" \
        "Headless build - API testing only")

    if [[ "$choice" == "1" ]]; then
        BUILD_TYPE="graphics"
    else
        BUILD_TYPE="headless"
    fi

    log_info "Selected: $BUILD_TYPE build"
}

build_revgame() {
    echo ""
    log_info "Building RevGame ($BUILD_TYPE)..."
    echo ""

    # Ensure we're in the right directory
    cd "$SCRIPT_DIR"

    # Check Cargo.toml exists
    if [ ! -f "Cargo.toml" ]; then
        log_error "Cargo.toml not found. Are you in the revgame directory?"
        exit 1
    fi

    # Build
    if [[ "$BUILD_TYPE" == "graphics" ]]; then
        cargo build --release --features graphics
    else
        cargo build --release
    fi

    log_info "Build complete!"
}

verify_build() {
    log_step "4/4" "Verification"

    local binary=""
    if [[ "$BUILD_TYPE" == "graphics" ]]; then
        binary="$SCRIPT_DIR/target/release/revgame"
    else
        # For headless, we check if the library was built
        binary="$SCRIPT_DIR/target/release/librevgame.rlib"
    fi

    echo ""
    if [[ "$BUILD_TYPE" == "graphics" ]]; then
        if [ -f "$binary" ]; then
            echo -e "  Binary: ${GREEN}$binary${NC}"
            log_info "Verification passed!"
        else
            echo -e "  Binary: ${RED}NOT FOUND${NC}"
            log_error "Build verification failed"
            exit 1
        fi
    else
        if [ -d "$SCRIPT_DIR/target/release" ]; then
            echo -e "  Build directory: ${GREEN}$SCRIPT_DIR/target/release${NC}"
            log_info "Headless build verification passed!"
        else
            log_error "Build verification failed"
            exit 1
        fi
    fi
}

show_next_steps() {
    echo ""
    echo -e "${BOLD}================================================================================"
    echo "                       Installation Complete!"
    echo -e "================================================================================${NC}"
    echo ""

    if [[ "$BUILD_TYPE" == "graphics" ]]; then
        echo "Run the game:"
        echo "  cd $SCRIPT_DIR"
        echo "  ./target/release/revgame"
        echo ""
        echo "Or with cargo:"
        echo "  cargo run --release --features graphics"
    else
        echo "The headless library has been built."
        echo ""
        echo "Run tests:"
        echo "  cargo test"
        echo ""
        echo "Use in your code:"
        echo "  use revgame::api::*;"
    fi

    echo ""
    echo "Configuration:"
    echo "  Edit config.json to set backend URL"
    echo ""
}

main() {
    show_banner
    detect_distro

    # First pass: check what's needed without Bevy deps
    BUILD_TYPE="graphics"  # Assume graphics for initial check
    check_dependencies || true

    # If Rust is missing, install it before asking about build type
    if [[ " ${MISSING_DEPS[*]} " =~ " rust " ]]; then
        echo ""
        if prompt_yn "Rust is not installed. Install it now?"; then
            install_rust
            # Remove rust from missing deps
            MISSING_DEPS=("${MISSING_DEPS[@]/rust/}")
        else
            log_error "Rust is required to build RevGame"
            exit 1
        fi
    fi

    # Now ask about build type
    choose_build_type

    # Re-check dependencies with the selected build type
    MISSING_DEPS=()
    check_dependencies || true

    # Install remaining dependencies
    install_dependencies

    # Build
    build_revgame

    # Verify
    verify_build

    # Done
    show_next_steps
}

main "$@"
