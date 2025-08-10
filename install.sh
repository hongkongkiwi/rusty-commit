#!/usr/bin/env bash
#
# Rusty Commit Installation Script
#
# This script automatically detects your OS and installs rusty-commit using the best available method.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh | bash
#   wget -qO- https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh | bash
#
# Environment variables:
#   INSTALL_METHOD - Force a specific installation method (homebrew, deb, rpm, binary, cargo)
#   INSTALL_VERSION - Install a specific version instead of latest
#   INSTALL_DIR - Custom installation directory for binary method (default: /usr/local/bin)
#   NO_SUDO - Don't use sudo for installation (may fail if permissions required)
#   VERIFY_CHECKSUM - Set to 'false' to skip checksum verification (not recommended)
#   VERIFY_SIGNATURE - Set to 'false' to skip GPG signature verification (not recommended)
#   ACCEPT_RISKS - Set to 'true' to bypass security warnings
#   GPG_KEY_ID - GPG key ID to use for verification (auto-detected if not set)

set -euo pipefail

# Trap errors and clean up
trap 'error "Installation failed on line $LINENO"' ERR
trap cleanup EXIT

# Configuration
REPO="hongkongkiwi/rusty-commit"
BINARY_NAME="rco"
CRATE_NAME="rusty-commit"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
VERIFY_CHECKSUM="${VERIFY_CHECKSUM:-true}"
VERIFY_SIGNATURE="${VERIFY_SIGNATURE:-true}"
ACCEPT_RISKS="${ACCEPT_RISKS:-false}"
TEMP_DIR=""

# Colors for output
if [[ -t 1 ]] && command -v tput >/dev/null 2>&1; then
    RED=$(tput setaf 1)
    GREEN=$(tput setaf 2)
    YELLOW=$(tput setaf 3)
    BLUE=$(tput setaf 4)
    NC=$(tput sgr0)
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

# Helper functions
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1" >&2
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

die() {
    error "$1"
    cleanup
    exit 1
}

cleanup() {
    if [[ -n "${TEMP_DIR:-}" ]] && [[ -d "${TEMP_DIR}" ]]; then
        rm -rf "${TEMP_DIR}"
    fi
}

# Security check
security_check() {
    # Check if running as root without explicit permission
    if [[ "${EUID}" -eq 0 ]] && [[ "${ACCEPT_RISKS}" != "true" ]]; then
        warn "Running as root is not recommended for security reasons."
        warn "Set ACCEPT_RISKS=true if you really want to proceed as root."
        die "Aborting installation for security reasons"
    fi

    # Check for suspicious environment
    if [[ -n "${LD_PRELOAD:-}" ]] || [[ -n "${LD_LIBRARY_PATH:-}" ]]; then
        warn "Suspicious environment variables detected (LD_PRELOAD/LD_LIBRARY_PATH)"
        if [[ "${ACCEPT_RISKS}" != "true" ]]; then
            die "Aborting installation for security reasons. Set ACCEPT_RISKS=true to override."
        fi
    fi
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Get system information
get_os() {
    local os
    os="$(uname -s)"
    case "${os}" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        CYGWIN*|MINGW*|MSYS*) echo "windows";;
        FreeBSD*)   echo "freebsd";;
        OpenBSD*)   echo "openbsd";;
        NetBSD*)    echo "netbsd";;
        *)          echo "unknown";;
    esac
}

get_arch() {
    local arch
    arch="$(uname -m)"
    case "${arch}" in
        x86_64|amd64)    echo "x86_64";;
        aarch64|arm64)   echo "aarch64";;
        armv7l|armhf)    echo "armv7";;
        i686|i386)       echo "i686";;
        riscv64)         echo "riscv64";;
        *)               echo "unknown";;
    esac
}
get_libc() {
    # Detect libc on Linux (gnu vs musl)
    if [[ "$(get_os)" != "linux" ]]; then
        echo "unknown"
        return
    fi
    if [[ -f /etc/alpine-release ]]; then
        echo "musl"
        return
    fi
    if ldd --version 2>&1 | grep -qi musl; then
        echo "musl"
    else
        echo "gnu"
    fi
}

get_distro() {
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        echo "${ID:-unknown}"
    elif [[ -f /etc/redhat-release ]]; then
        echo "rhel"
    elif [[ -f /etc/debian_version ]]; then
        echo "debian"
    else
        echo "unknown"
    fi
}

# Check if running as root or with sudo
can_sudo() {
    if [[ -z "${NO_SUDO:-}" ]]; then
        if [[ "${EUID}" -eq 0 ]]; then
            return 0
        elif command_exists sudo; then
            # Test sudo access
            if sudo -n true 2>/dev/null; then
                return 0
            elif [[ -t 0 ]]; then
                # Interactive terminal, can prompt for password
                return 0
            fi
        fi
    fi
    return 1
}

# Run command with sudo if needed
run_privileged() {
    if [[ "${EUID}" -eq 0 ]]; then
        "$@"
    elif [[ -z "${NO_SUDO:-}" ]] && command_exists sudo; then
        sudo "$@"
    else
        "$@"
    fi
}

# Get latest version from GitHub
get_latest_version() {
    local version
    info "Fetching latest version from GitHub..."

    if ! command_exists curl; then
        die "curl is required but not installed"
    fi

    version=$(curl -fsSL --retry 3 --retry-delay 2 \
        "https://api.github.com/repos/${REPO}/releases/latest" | \
        grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/') || {
        die "Failed to fetch latest version from GitHub"
    }

    if [[ -z "${version}" ]]; then
        die "Failed to determine latest version"
    fi

    echo "${version}"
}

# Download and verify checksum
download_and_verify() {
    local url="$1"
    local output_file="$2"
    local expected_checksum="${3:-}"

    info "Downloading from: ${url}"

    # Download with retry logic
    local retry=0
    local max_retries=3

    while [[ ${retry} -lt ${max_retries} ]]; do
        if curl -fsSL --retry 3 --retry-delay 2 "${url}" -o "${output_file}"; then
            break
        fi
        retry=$((retry + 1))
        warn "Download failed (attempt ${retry}/${max_retries})"
        sleep 2
    done

    if [[ ${retry} -eq ${max_retries} ]]; then
        return 1
    fi

    # Verify checksum if provided
    if [[ -n "${expected_checksum}" ]] && [[ "${VERIFY_CHECKSUM}" == "true" ]]; then
        info "Verifying SHA256 checksum..."
        local actual_checksum

        if command_exists sha256sum; then
            actual_checksum=$(sha256sum "${output_file}" | cut -d' ' -f1)
        elif command_exists shasum; then
            actual_checksum=$(shasum -a 256 "${output_file}" | cut -d' ' -f1)
        else
            warn "Cannot verify checksum: neither sha256sum nor shasum found"
            if [[ "${ACCEPT_RISKS}" != "true" ]]; then
                die "Checksum verification required but tools not available"
            fi
            return 0
        fi

        if [[ "${actual_checksum}" != "${expected_checksum}" ]]; then
            error "Checksum verification failed!"
            error "Expected: ${expected_checksum}"
            error "Actual:   ${actual_checksum}"
            return 1
        fi

        success "Checksum verified"
    fi

    # Try Cosign signature verification first (preferred for Rust binaries)
    if check_cosign; then
        verify_cosign_signature "${output_file}" || {
            if [[ "${ACCEPT_RISKS}" != "true" ]]; then
                error "Cosign signature verification failed"
                return 1
            fi
        }
    else
        # Fall back to GPG signature verification
        verify_gpg_signature "${output_file}" || {
            if [[ "${ACCEPT_RISKS}" != "true" ]]; then
                error "GPG signature verification failed"
                return 1
            fi
        }
    fi

    return 0
}

# Get checksum for a file from GitHub release
get_checksum_for_file() {
    local version="$1"
    local filename="$2"
    local checksums_url="https://github.com/${REPO}/releases/download/${version}/SHA256SUMS.txt"

    if [[ "${VERIFY_CHECKSUM}" != "true" ]]; then
        echo ""
        return
    fi

    # Download checksums file
    local checksums
    checksums=$(curl -fsSL --retry 2 "${checksums_url}" 2>/dev/null) || {
        warn "Could not download checksums file"
        echo ""
        return
    }

    # Extract checksum for specific file
    echo "${checksums}" | grep "${filename}" | cut -d' ' -f1 || echo ""
}

# Check if Cosign is available
check_cosign() {
    if command_exists cosign; then
        return 0
    fi
    return 1
}

# Verify Cosign signature of a file
verify_cosign_signature() {
    local file="$1"
    local bundle_file="${file}.cosign.bundle"

    if [[ "${VERIFY_SIGNATURE:-true}" != "true" ]]; then
        return 0
    fi

    if ! check_cosign; then
        # Cosign not installed, fall back to GPG verification
        return 0
    fi

    # Check if bundle exists
    local bundle_url="${file/https:\/\/github.com/https:\/\/github.com}.cosign.bundle"
    if [[ "${file}" == *.tar.gz ]] || [[ "${file}" == *.zip ]]; then
        bundle_url="${file}.cosign.bundle"
    fi

    info "Checking for Cosign signature bundle..."
    if curl -fsSL "${bundle_url}" -o "${bundle_file}" 2>/dev/null; then
        info "Verifying Cosign signature..."

        # Verify using the bundle (includes certificate and signature)
        if cosign verify-blob \
            --bundle "${bundle_file}" \
            --certificate-identity-regexp "https://github.com/${REPO}/.github/workflows/release.yml@.*" \
            --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
            "${file}" 2>/dev/null; then
            success "Cosign signature verified successfully"
            rm -f "${bundle_file}"
            return 0
        else
            warn "Cosign signature verification failed"
            rm -f "${bundle_file}"
            if [[ "${ACCEPT_RISKS}" != "true" ]]; then
                return 1
            fi
        fi
    else
        # No Cosign bundle available, that's okay
        return 0
    fi
}

# Import GPG public key for signature verification
import_gpg_key() {
    if ! command_exists gpg; then
        warn "GPG not installed, skipping GPG signature verification"
        return 1
    fi

    info "Importing Rusty Commit GPG public key..."

    # Try to import from GitHub repository first
    local key_url="https://raw.githubusercontent.com/${REPO}/main/.github/rusty-commit-public-key.asc"

    if curl -fsSL "${key_url}" 2>/dev/null | gpg --import 2>/dev/null; then
        success "GPG public key imported successfully"
        return 0
    fi

    # Fallback to keyserver
    info "Trying to fetch GPG key from keyserver..."
    local keyservers=(
        "hkps://keys.openpgp.org"
        "hkps://keyserver.ubuntu.com"
        "hkps://pgp.mit.edu"
    )

    for keyserver in "${keyservers[@]}"; do
        if gpg --keyserver "${keyserver}" --recv-keys "${GPG_KEY_ID:-}" 2>/dev/null; then
            success "GPG key imported from ${keyserver}"
            return 0
        fi
    done

    warn "Could not import GPG public key"
    return 1
}

# Verify GPG signature of a file
verify_gpg_signature() {
    local file="$1"
    local signature_file="${file}.asc"

    if [[ "${VERIFY_SIGNATURE:-true}" != "true" ]]; then
        return 0
    fi

    if ! command_exists gpg; then
        warn "GPG not installed, skipping signature verification"
        return 0
    fi

    # Download signature file
    local signature_url="${file/https:\/\/github.com/https:\/\/github.com}.asc"
    if [[ "${file}" == *.tar.gz ]] || [[ "${file}" == *.zip ]]; then
        # For direct file downloads, construct signature URL
        signature_url="${file}.asc"
    fi

    info "Downloading signature file..."
    if ! curl -fsSL "${signature_url}" -o "${signature_file}" 2>/dev/null; then
        warn "Signature file not available, skipping verification"
        return 0
    fi

    # Import key if not already present
    import_gpg_key || return 0

    info "Verifying GPG signature..."
    if gpg --verify "${signature_file}" "${file}" 2>/dev/null; then
        success "GPG signature verified successfully"
        rm -f "${signature_file}"
        return 0
    else
        error "GPG signature verification failed!"
        if [[ "${ACCEPT_RISKS}" != "true" ]]; then
            rm -f "${signature_file}"
            return 1
        fi
        warn "Continuing despite failed signature (ACCEPT_RISKS=true)"
        rm -f "${signature_file}"
        return 0
    fi
}

# Verify package signatures (for .deb and .rpm)
verify_package_signature() {
    local package_file="$1"
    local package_type="${package_file##*.}"

    if [[ "${VERIFY_SIGNATURE:-true}" != "true" ]]; then
        return 0
    fi

    case "${package_type}" in
        deb)
            if command_exists dpkg-sig; then
                info "Verifying .deb package signature..."
                if dpkg-sig --verify "${package_file}" 2>/dev/null | grep -q "GOODSIG"; then
                    success "Package signature verified"
                    return 0
                fi
            fi
            # Fall back to GPG signature
            verify_gpg_signature "${package_file}"
            ;;
        rpm)
            if command_exists rpm; then
                info "Verifying .rpm package signature..."
                # Import GPG key to RPM database
                import_gpg_key
                if rpm --checksig "${package_file}" 2>/dev/null | grep -q "OK"; then
                    success "Package signature verified"
                    return 0
                fi
            fi
            # Fall back to GPG signature
            verify_gpg_signature "${package_file}"
            ;;
        *)
            # For other file types, use GPG signature
            verify_gpg_signature "${package_file}"
            ;;
    esac
}

# Install using Homebrew (macOS/Linux)
install_homebrew() {
    info "Attempting Homebrew installation..."

    if ! command_exists brew; then
        warn "Homebrew not found"
        return 1
    fi

    # Ensure tap exists
    if ! brew tap | grep -q "hongkongkiwi/tap"; then
        info "Adding hongkongkiwi/tap..."
        if ! brew tap hongkongkiwi/tap 2>/dev/null; then
            warn "Failed to add Homebrew tap"
            return 1
        fi
    fi

    # Install package
    if brew install rusty-commit 2>/dev/null; then
        success "Installed via Homebrew"
        return 0
    else
        warn "Homebrew installation failed"
        return 1
    fi
}

# Install .deb package (Debian/Ubuntu)
install_deb() {
    local version="${INSTALL_VERSION:-$(get_latest_version)}"
    local arch="$(get_arch)"
    local deb_arch

    case "${arch}" in
        x86_64)  deb_arch="amd64";;
        aarch64) deb_arch="arm64";;
        armv7)   deb_arch="armhf";;
        riscv64) deb_arch="riscv64";;
        *)
            warn "Unsupported architecture for .deb: ${arch}"
            return 1
            ;;
    esac

    local deb_filename="rusty-commit_${version#v}_${deb_arch}.deb"
    local deb_url="https://github.com/${REPO}/releases/download/${version}/${deb_filename}"
    local temp_file="${TEMP_DIR}/${deb_filename}"

    # Get checksum
    local expected_checksum
    expected_checksum=$(get_checksum_for_file "${version}" "${deb_filename}")

    info "Downloading .deb package..."
    if ! download_and_verify "${deb_url}" "${temp_file}" "${expected_checksum}"; then
        warn "Failed to download or verify .deb package"
        return 1
    fi

    info "Installing .deb package..."
    if command_exists apt-get; then
        if ! run_privileged apt-get install -y "${temp_file}" 2>/dev/null; then
            warn "Failed to install .deb package"
            return 1
        fi
    elif command_exists dpkg; then
        if ! run_privileged dpkg -i "${temp_file}" 2>/dev/null; then
            # Try to fix dependencies
            run_privileged apt-get install -f -y 2>/dev/null || true
            # Retry installation
            if ! run_privileged dpkg -i "${temp_file}" 2>/dev/null; then
                warn "Failed to install .deb package"
                return 1
            fi
        fi
    else
        warn "No suitable package manager for .deb files"
        return 1
    fi

    success "Installed via .deb package"
    return 0
}

# Install .rpm package (Fedora/RHEL/CentOS)
install_rpm() {
    local version="${INSTALL_VERSION:-$(get_latest_version)}"
    local arch="$(get_arch)"
    local rpm_arch

    case "${arch}" in
        x86_64)  rpm_arch="x86_64";;
        aarch64) rpm_arch="aarch64";;
        riscv64) rpm_arch="riscv64";;
        *)
            warn "Unsupported architecture for .rpm: ${arch}"
            return 1
            ;;
    esac

    local rpm_filename="rusty-commit-${version#v}-1.${rpm_arch}.rpm"
    local rpm_url="https://github.com/${REPO}/releases/download/${version}/${rpm_filename}"
    local temp_file="${TEMP_DIR}/${rpm_filename}"

    # Get checksum
    local expected_checksum
    expected_checksum=$(get_checksum_for_file "${version}" "${rpm_filename}")

    info "Downloading .rpm package..."
    if ! download_and_verify "${rpm_url}" "${temp_file}" "${expected_checksum}"; then
        warn "Failed to download or verify .rpm package"
        return 1
    fi

    info "Installing .rpm package..."
    if command_exists dnf; then
        if ! run_privileged dnf install -y "${temp_file}" 2>/dev/null; then
            warn "Failed to install .rpm package"
            return 1
        fi
    elif command_exists yum; then
        if ! run_privileged yum install -y "${temp_file}" 2>/dev/null; then
            warn "Failed to install .rpm package"
            return 1
        fi
    elif command_exists rpm; then
        if ! run_privileged rpm -Uvh "${temp_file}" 2>/dev/null; then
            warn "Failed to install .rpm package"
            return 1
        fi
    else
        warn "No suitable package manager for .rpm files"
        return 1
    fi

    success "Installed via .rpm package"
    return 0
}

# Install from binary release
install_binary() {
    local version="${INSTALL_VERSION:-$(get_latest_version)}"
    local os="$(get_os)"
    local arch="$(get_arch)"
    local archive_name

    # Determine archive name based on OS, architecture and libc
    local libc="$(get_libc)"
    case "${os}-${arch}-${libc}" in
        linux-x86_64-musl)   archive_name="rustycommit-linux-musl-x86_64.tar.gz";;
        linux-aarch64-musl)  archive_name="rustycommit-linux-musl-aarch64.tar.gz";;
        linux-riscv64-musl)  archive_name="rustycommit-linux-musl-riscv64.tar.gz";;
        linux-x86_64-gnu|linux-x86_64-unknown)   archive_name="rustycommit-linux-x86_64.tar.gz";;
        linux-aarch64-gnu|linux-aarch64-unknown) archive_name="rustycommit-linux-aarch64.tar.gz";;
        linux-armv7-gnu|linux-armv7-unknown)     archive_name="rustycommit-linux-armv7.tar.gz";;
        linux-riscv64-gnu|linux-riscv64-unknown) archive_name="rustycommit-linux-riscv64.tar.gz";;
        macos-x86_64-*)   archive_name="rustycommit-macos-x86_64.tar.gz";;
        macos-aarch64-*)  archive_name="rustycommit-macos-aarch64.tar.gz";;
        windows-x86_64-*) archive_name="rustycommit-windows-x86_64.zip";;
        windows-i686-*)   archive_name="rustycommit-windows-i686.zip";;
        *)
            die "Unsupported OS/architecture combination: ${os}-${arch}-${libc}"
            ;;
    esac

    local download_url="https://github.com/${REPO}/releases/download/${version}/${archive_name}"
    local temp_file="${TEMP_DIR}/${archive_name}"

    # Get checksum
    local expected_checksum
    expected_checksum=$(get_checksum_for_file "${version}" "${archive_name}")

    info "Downloading binary release..."
    if ! download_and_verify "${download_url}" "${temp_file}" "${expected_checksum}"; then
        die "Failed to download or verify binary release"
    fi

    info "Extracting binary..."
    cd "${TEMP_DIR}"

    if [[ "${archive_name}" == *.zip ]]; then
        if ! command_exists unzip; then
            die "unzip is required but not installed"
        fi
        unzip -q "${archive_name}" || die "Failed to extract archive"
    else
        tar xzf "${archive_name}" || die "Failed to extract archive"
    fi

    # Find and install binary
    local binary_file
    if [[ -f "${BINARY_NAME}" ]]; then
        binary_file="${BINARY_NAME}"
    elif [[ -f "${BINARY_NAME}.exe" ]]; then
        binary_file="${BINARY_NAME}.exe"
    else
        die "Binary not found in archive"
    fi

    info "Installing binary to ${INSTALL_DIR}..."
    chmod +x "${binary_file}"

    # Create install directory if needed
    if ! run_privileged mkdir -p "${INSTALL_DIR}" 2>/dev/null; then
        die "Failed to create installation directory: ${INSTALL_DIR}"
    fi

    # Install binary
    if ! run_privileged mv "${binary_file}" "${INSTALL_DIR}/" 2>/dev/null; then
        die "Failed to install binary to ${INSTALL_DIR}"
    fi

    # Verify installation
    if [[ ! -f "${INSTALL_DIR}/${BINARY_NAME}" ]] && [[ ! -f "${INSTALL_DIR}/${BINARY_NAME}.exe" ]]; then
        die "Binary installation verification failed"
    fi

    # Add to PATH if needed
    if ! echo "${PATH}" | grep -q "${INSTALL_DIR}"; then
        warn "Please add ${INSTALL_DIR} to your PATH"
        warn "You can do this by adding the following to your shell profile:"
        warn "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi

    success "Installed binary to ${INSTALL_DIR}/${BINARY_NAME}"
    return 0
}

# Install using cargo
install_cargo() {
    info "Attempting cargo installation..."

    if ! command_exists cargo; then
        warn "Cargo not found"
        return 1
    fi

    # Check for required build tools
    if [[ "$(get_os)" == "linux" ]]; then
        if ! command_exists gcc && ! command_exists clang; then
            warn "No C compiler found (gcc or clang required for building)"
            return 1
        fi
    fi

    if cargo install "${CRATE_NAME}" --features secure-storage 2>/dev/null; then
        success "Installed via cargo"
        return 0
    else
        warn "Cargo installation failed"
        return 1
    fi
}

# Detect and run appropriate installation method
install() {
    local os="$(get_os)"
    local arch="$(get_arch)"
    local distro="$(get_distro)"

    info "System detected: OS=${os}, Arch=${arch}, Distro=${distro}"

    # Check if specific method requested
    if [[ -n "${INSTALL_METHOD:-}" ]]; then
        info "Using requested installation method: ${INSTALL_METHOD}"
        case "${INSTALL_METHOD}" in
            homebrew)
                install_homebrew || install_binary
                ;;
            deb)
                install_deb || install_binary
                ;;
            rpm)
                install_rpm || install_binary
                ;;
            binary)
                install_binary
                ;;
            cargo)
                install_cargo || install_binary
                ;;
            *)
                die "Unknown installation method: ${INSTALL_METHOD}"
                ;;
        esac
        return
    fi

    # Auto-detect best installation method
    local success=false

    case "${os}" in
        macos)
            # Try: Homebrew → Cargo → Binary
            if command_exists brew; then
                install_homebrew && success=true
            fi

            if [[ "${success}" == "false" ]] && command_exists cargo; then
                install_cargo && success=true
            fi

            if [[ "${success}" == "false" ]]; then
                install_binary && success=true
            fi
            ;;

        linux)
            case "${distro}" in
                ubuntu|debian|pop|linuxmint|elementary|kali|raspbian)
                    # Try: .deb → Homebrew → Cargo → Binary
                    if command_exists apt-get || command_exists dpkg; then
                        install_deb && success=true
                    fi

                    if [[ "${success}" == "false" ]] && command_exists brew; then
                        install_homebrew && success=true
                    fi

                    if [[ "${success}" == "false" ]] && command_exists cargo; then
                        install_cargo && success=true
                    fi

                    if [[ "${success}" == "false" ]]; then
                        install_binary && success=true
                    fi
                    ;;

                fedora|rhel|centos|rocky|almalinux|opensuse*)
                    # Try: .rpm → Homebrew → Cargo → Binary
                    if command_exists dnf || command_exists yum || command_exists rpm; then
                        install_rpm && success=true
                    fi

                    if [[ "${success}" == "false" ]] && command_exists brew; then
                        install_homebrew && success=true
                    fi

                    if [[ "${success}" == "false" ]] && command_exists cargo; then
                        install_cargo && success=true
                    fi

                    if [[ "${success}" == "false" ]]; then
                        install_binary && success=true
                    fi
                    ;;

                arch|manjaro|endeavouros)
                    # Try: Cargo → Homebrew → Binary
                    if command_exists cargo; then
                        install_cargo && success=true
                    fi

                    if [[ "${success}" == "false" ]] && command_exists brew; then
                        install_homebrew && success=true
                    fi

                    if [[ "${success}" == "false" ]]; then
                        install_binary && success=true
                    fi
                    ;;

                alpine)
                    # Try: Cargo → Binary
                    if command_exists cargo; then
                        install_cargo && success=true
                    fi

                    if [[ "${success}" == "false" ]]; then
                        install_binary && success=true
                    fi
                    ;;

                *)
                    # Unknown distro - Try: Homebrew → Cargo → Binary
                    if command_exists brew; then
                        install_homebrew && success=true
                    fi

                    if [[ "${success}" == "false" ]] && command_exists cargo; then
                        install_cargo && success=true
                    fi

                    if [[ "${success}" == "false" ]]; then
                        install_binary && success=true
                    fi
                    ;;
            esac
            ;;

        freebsd|openbsd|netbsd)
            # BSD systems - Try: Cargo → Binary
            if command_exists cargo; then
                install_cargo && success=true
            fi

            if [[ "${success}" == "false" ]]; then
                install_binary && success=true
            fi
            ;;

        windows)
            # Try: Cargo → Binary
            if command_exists cargo; then
                install_cargo && success=true
            fi

            if [[ "${success}" == "false" ]]; then
                install_binary && success=true
            fi
            ;;

        *)
            # Unknown OS - try binary as last resort
            warn "Unknown operating system: ${os}"
            install_binary && success=true
            ;;
    esac

    if [[ "${success}" == "false" ]]; then
        die "All installation methods failed"
    fi
}

# Verify installation
verify_installation() {
    if command_exists "${BINARY_NAME}"; then
        local installed_version
        installed_version=$("${BINARY_NAME}" --version 2>/dev/null | head -1) || {
            warn "Binary found but version check failed"
            return
        }
        success "Rusty Commit installed successfully!"
        info "Version: ${installed_version}"
        info "Run '${BINARY_NAME} auth' to set up your AI provider"
    else
        die "Installation verification failed. ${BINARY_NAME} not found in PATH"
    fi
}

# Print help
print_help() {
    cat <<EOF
Rusty Commit Installation Script

Usage: $0 [OPTIONS]

Options:
  --help, -h              Show this help message
  --method METHOD         Force installation method (homebrew|deb|rpm|binary|cargo)
  --version VERSION       Install specific version
  --dir DIR              Installation directory for binary method
  --no-sudo              Don't use sudo
  --skip-verify          Skip checksum verification (not recommended)
  --accept-risks         Accept security risks and bypass warnings

Environment variables:
  INSTALL_METHOD         Same as --method
  INSTALL_VERSION        Same as --version
  INSTALL_DIR           Same as --dir
  NO_SUDO               Same as --no-sudo
  VERIFY_CHECKSUM       Set to 'false' to skip checksum verification
  VERIFY_SIGNATURE      Set to 'false' to skip GPG signature verification
  ACCEPT_RISKS          Set to 'true' to bypass security warnings

Examples:
  # Standard installation
  curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh | bash

  # Install specific version
  curl -fsSL ... | bash -s -- --version v1.0.2

  # Force binary installation to custom directory
  curl -fsSL ... | bash -s -- --method binary --dir ~/.local/bin

  # Install without sudo
  curl -fsSL ... | NO_SUDO=1 bash

EOF
}

# Main execution
main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --help|-h)
                print_help
                exit 0
                ;;
            --method)
                INSTALL_METHOD="$2"
                shift 2
                ;;
            --version)
                INSTALL_VERSION="$2"
                shift 2
                ;;
            --dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            --no-sudo)
                NO_SUDO=1
                shift
                ;;
            --skip-verify)
                VERIFY_CHECKSUM=false
                shift
                ;;
            --accept-risks)
                ACCEPT_RISKS=true
                shift
                ;;
            *)
                warn "Unknown option: $1"
                shift
                ;;
        esac
    done

    echo "╔══════════════════════════════════════╗"
    echo "║   Rusty Commit Installation Script   ║"
    echo "╚══════════════════════════════════════╝"
    echo

    # Perform security checks
    security_check

    # Check prerequisites
    if ! command_exists curl; then
        die "curl is required but not installed"
    fi

    # Create temp directory
    TEMP_DIR=$(mktemp -d -t rusty-commit-install.XXXXXX) || {
        die "Failed to create temporary directory"
    }

    # Run installation
    install

    # Verify
    verify_installation

    echo
    echo "🎉 Installation complete! Happy committing! 🦀"
}

# Only run main if not being sourced
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
