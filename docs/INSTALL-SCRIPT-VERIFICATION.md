# Install Script Verification Guide

This guide shows how to verify the install script before running it, providing an extra layer of security.

## Quick Start (Secure One-liner)

Instead of directly piping to bash, you can verify first:

### Method 1: Verify with Cosign (Recommended)
```bash
# Download and verify install script with Cosign
curl -fsSL https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/install.sh -o install.sh
curl -fsSL https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/install.sh.cosign.bundle -o install.sh.cosign.bundle

# Verify signature (requires cosign)
cosign verify-blob \
  --bundle install.sh.cosign.bundle \
  --certificate-identity-regexp "https://github.com/hongkongkiwi/rusty-commit/.github/workflows/release.yml@.*" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  install.sh

# If verification succeeds, run the script
bash install.sh
```

### Method 2: Verify with GPG
```bash
# Download install script and GPG signature
curl -fsSL https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/install.sh -o install.sh
curl -fsSL https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/install.sh.asc -o install.sh.asc

# Import GPG public key
gpg --keyserver hkps://keys.openpgp.org --recv-keys 0EC2DFF577818B86BA38DA3F164E3F90E425B2AD

# Verify signature
gpg --verify install.sh.asc install.sh

# If verification succeeds, run the script
bash install.sh
```

### Method 3: Quick GPG Verification One-liner
```bash
# Download, verify, and run in one command
curl -fsSL https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/install.sh -o install.sh && \
curl -fsSL https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/install.sh.asc -o install.sh.asc && \
gpg --keyserver hkps://keys.openpgp.org --recv-keys 0EC2DFF577818B86BA38DA3F164E3F90E425B2AD && \
gpg --verify install.sh.asc install.sh && \
bash install.sh
```

## Why Verify the Install Script?

### Security Benefits
1. **Prevents script tampering** - Ensures the script hasn't been modified maliciously
2. **Verifies authenticity** - Confirms it comes from the official Rusty Commit project
3. **Supply chain protection** - Guards against compromised distribution channels
4. **Best security practice** - Following the principle of "trust but verify"

### What the Signatures Guarantee
- **Cosign signature**: Proves the script was signed by our GitHub Actions workflow
- **GPG signature**: Provides traditional cryptographic verification
- **Both methods**: Ensure the script is exactly what we published

## For Security-Conscious Users

If you want maximum security, you can also:

1. **Review the script contents** before running:
   ```bash
   curl -fsSL https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/install.sh | less
   ```

2. **Check the script source** in our repository:
   ```bash
   # View the latest version on GitHub
   curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh | less
   ```

3. **Manual installation** instead of script:
   - Use package managers directly: `brew install`, `apt install`, etc.
   - Download binaries manually and verify signatures
   - Build from source: `cargo install rusty-commit`

## Troubleshooting

### "cosign: command not found"
Install Cosign first:
```bash
# macOS
brew install cosign

# Linux
curl -O -L "https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64"
sudo mv cosign-linux-amd64 /usr/local/bin/cosign
chmod +x /usr/local/bin/cosign
```

### "gpg: command not found"
Install GPG first:
```bash
# macOS
brew install gnupg

# Ubuntu/Debian
sudo apt install gnupg

# Fedora/RHEL
sudo dnf install gnupg2
```

### Signature verification fails
1. **Check internet connection** - Key servers may be temporarily unavailable
2. **Try different keyserver**:
   ```bash
   gpg --keyserver hkp://pgpkeys.eu --recv-keys 0EC2DFF577818B86BA38DA3F164E3F90E425B2AD
   ```
3. **Use repository public key** as fallback:
   ```bash
   curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/.github/rusty-commit-public-key.asc | gpg --import
   ```

## Traditional One-liner (If You Trust Us)

If you trust the project and want the simplest installation:
```bash
curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh | bash
```

*Note: This fetches directly from the repository and doesn't verify signatures, but it's still secure over HTTPS.*

## Summary

- **Most secure**: Download, verify with Cosign/GPG, then run
- **Balanced**: Use verification one-liners above
- **Quick & easy**: Traditional curl-to-bash (still safe with HTTPS)
- **Maximum control**: Manual installation methods

Choose the method that matches your security requirements! 🔒
