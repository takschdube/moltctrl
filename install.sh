#!/usr/bin/env bash
# moltctrl installer/uninstaller
# Usage:
#   Install (latest):  curl -fsSL https://raw.githubusercontent.com/takschdube/moltctrl/main/install.sh | sh
#   Install (version): curl -fsSL https://raw.githubusercontent.com/takschdube/moltctrl/main/install.sh | sh -s -- v0.2.0
#   Uninstall:         curl -fsSL https://raw.githubusercontent.com/takschdube/moltctrl/main/install.sh | sh -s -- --uninstall

set -euo pipefail

REPO="takschdube/moltctrl"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
DATA_DIR="${HOME}/.moltctrl"

# Colors
if [ -t 1 ]; then
    RED='\033[1;31m'
    GREEN='\033[1;32m'
    BLUE='\033[1;34m'
    NC='\033[0m'
else
    RED='' GREEN='' BLUE='' NC=''
fi

info()    { printf "${BLUE}::${NC} %s\n" "$*"; }
success() { printf "${GREEN}✓${NC} %s\n" "$*"; }
error()   { printf "${RED}error:${NC} %s\n" "$*" >&2; }
die()     { error "$@"; exit 1; }

# --- Uninstall ---
uninstall() {
    info "Uninstalling moltctrl..."

    if [ -f "${INSTALL_DIR}/moltctrl" ]; then
        sudo rm -f "${INSTALL_DIR}/moltctrl"
        success "Removed ${INSTALL_DIR}/moltctrl"
    else
        info "Binary not found at ${INSTALL_DIR}/moltctrl (already removed?)"
    fi

    if [ -d "${DATA_DIR}" ]; then
        printf "  Remove all instance data at %s? [y/N] " "${DATA_DIR}"
        read -r answer < /dev/tty 2>/dev/null || answer="n"
        if echo "$answer" | grep -qi "^y"; then
            rm -rf "${DATA_DIR}"
            success "Removed ${DATA_DIR}"
        else
            info "Kept ${DATA_DIR}"
        fi
    fi

    success "moltctrl uninstalled"
    exit 0
}

# --- Detect platform ---
detect_target() {
    local os arch

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)
            case "$arch" in
                x86_64)  echo "x86_64-unknown-linux-musl" ;;
                aarch64) echo "aarch64-unknown-linux-musl" ;;
                arm64)   echo "aarch64-unknown-linux-musl" ;;
                *)       die "Unsupported architecture: ${arch}" ;;
            esac
            ;;
        Darwin)
            case "$arch" in
                x86_64)  echo "x86_64-apple-darwin" ;;
                arm64)   echo "aarch64-apple-darwin" ;;
                aarch64) echo "aarch64-apple-darwin" ;;
                *)       die "Unsupported architecture: ${arch}" ;;
            esac
            ;;
        MINGW*|MSYS*|CYGWIN*)
            echo "x86_64-pc-windows-msvc"
            ;;
        *)
            die "Unsupported OS: ${os}"
            ;;
    esac
}

# --- Parse args ---
VERSION=""
for arg in "$@"; do
    case "$arg" in
        --uninstall) uninstall ;;
        v*)          VERSION="$arg" ;;
        *)           die "Unknown argument: ${arg}" ;;
    esac
done

# --- Resolve version ---
if [ -z "$VERSION" ]; then
    info "Fetching latest release..."
    VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')" \
        || die "Failed to fetch latest version. Check https://github.com/${REPO}/releases"
fi

TARGET="$(detect_target)"
ARCHIVE="moltctrl-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

echo ""
echo "  moltctrl installer"
echo "  ==================="
echo ""
info "Version:  ${VERSION}"
info "Platform: ${TARGET}"
info "Target:   ${INSTALL_DIR}/moltctrl"
echo ""

# --- Download ---
TMPDIR="$(mktemp -d)"
trap 'rm -rf "${TMPDIR}"' EXIT

info "Downloading ${URL}..."
if ! curl -fsSL -o "${TMPDIR}/${ARCHIVE}" "${URL}"; then
    die "Download failed. Check that ${VERSION} exists at https://github.com/${REPO}/releases"
fi

# --- Extract ---
info "Extracting..."
if echo "$ARCHIVE" | grep -q '\.zip$'; then
    unzip -q "${TMPDIR}/${ARCHIVE}" -d "${TMPDIR}"
else
    tar xzf "${TMPDIR}/${ARCHIVE}" -C "${TMPDIR}"
fi

# --- Install ---
info "Installing to ${INSTALL_DIR}..."
if [ -w "${INSTALL_DIR}" ]; then
    install -m 755 "${TMPDIR}/moltctrl" "${INSTALL_DIR}/moltctrl"
else
    sudo install -m 755 "${TMPDIR}/moltctrl" "${INSTALL_DIR}/moltctrl"
fi

# --- Verify ---
if command -v moltctrl >/dev/null 2>&1; then
    INSTALLED_VERSION="$(moltctrl version 2>/dev/null || echo "unknown")"
    echo ""
    success "moltctrl installed! (${INSTALLED_VERSION})"
else
    echo ""
    success "moltctrl installed to ${INSTALL_DIR}/moltctrl"
    if ! echo "$PATH" | grep -q "${INSTALL_DIR}"; then
        echo ""
        info "Add ${INSTALL_DIR} to your PATH:"
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi
fi

echo ""
echo "  Get started:"
echo "    moltctrl"
echo ""
