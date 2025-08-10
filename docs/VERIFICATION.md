# Package Verification Guide

This guide explains how to manually verify the authenticity of Rusty Commit packages and binaries.

## Signature Methods

Rusty Commit uses multiple signing methods following Rust ecosystem best practices:

1. **Cosign/Sigstore** (Primary for binaries) - Keyless signing with OIDC identity
2. **GPG Signatures** (Traditional) - Detached signatures for all artifacts
3. **GitHub Attestations** - Provenance attestations for build artifacts
4. **Native Package Signing** - OS-specific signing for .deb and .rpm packages

## Automatic Verification

The install script automatically verifies (in order of preference):
1. Cosign/Sigstore signatures (if cosign is installed)
2. GPG signatures (fallback if cosign unavailable)
3. SHA256 checksums
4. Native package signatures (.deb and .rpm)

To skip verification (not recommended):
```bash
VERIFY_SIGNATURE=false VERIFY_CHECKSUM=false curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh | bash
```

## Manual Verification

### Method 1: Cosign/Sigstore (Recommended for Binaries)

#### Install Cosign
```bash
# macOS
brew install cosign

# Linux (binary)
curl -O -L "https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64"
sudo mv cosign-linux-amd64 /usr/local/bin/cosign
chmod +x /usr/local/bin/cosign

# Or using go
go install github.com/sigstore/cosign/v2/cmd/cosign@latest
```

#### Verify with Cosign Bundle
```bash
# Download binary and its Cosign bundle
wget https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/rustycommit-linux-x86_64.tar.gz
wget https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/rustycommit-linux-x86_64.tar.gz.cosign.bundle

# Verify using the bundle (recommended)
cosign verify-blob \
  --bundle rustycommit-linux-x86_64.tar.gz.cosign.bundle \
  --certificate-identity-regexp "https://github.com/hongkongkiwi/rusty-commit/.github/workflows/release.yml@.*" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  rustycommit-linux-x86_64.tar.gz
```

#### Verify GitHub Attestations
```bash
# Using GitHub CLI
gh attestation verify rustycommit-linux-x86_64.tar.gz \
  --repo hongkongkiwi/rusty-commit
```

### Method 2: GPG Signatures (Traditional)

#### 1. Import the GPG Public Key

```bash
# From GitHub repository
curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/.github/rusty-commit-public-key.asc | gpg --import

# Or from keyserver
gpg --keyserver hkps://keys.openpgp.org --recv-keys 0EC2DFF577818B86BA38DA3F164E3F90E425B2AD
# Alternative keyserver:
# gpg --keyserver hkp://pgpkeys.eu --recv-keys 0EC2DFF577818B86BA38DA3F164E3F90E425B2AD
```

#### 2. Verify SHA256 Checksums

```bash
# Download checksums and signature
wget https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/SHA256SUMS.txt
wget https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/SHA256SUMS.txt.asc

# Verify checksum file signature
gpg --verify SHA256SUMS.txt.asc SHA256SUMS.txt

# Verify your download
sha256sum -c SHA256SUMS.txt 2>/dev/null | grep "your-file-name"
```

### 3. Verify Package Signatures

#### For .deb packages:
```bash
# Download package and signature
wget https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/rusty-commit_1.0.2_amd64.deb
wget https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/rusty-commit_1.0.2_amd64.deb.asc

# Verify with GPG
gpg --verify rusty-commit_1.0.2_amd64.deb.asc rusty-commit_1.0.2_amd64.deb

# Or verify embedded signature (if available)
dpkg-sig --verify rusty-commit_1.0.2_amd64.deb
```

#### For .rpm packages:
```bash
# Download package and signature
wget https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/rusty-commit-1.0.2-1.x86_64.rpm
wget https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/rusty-commit-1.0.2-1.x86_64.rpm.asc

# Verify with GPG
gpg --verify rusty-commit-1.0.2-1.x86_64.rpm.asc rusty-commit-1.0.2-1.x86_64.rpm

# Or verify embedded signature (if available)
rpm --checksig rusty-commit-1.0.2-1.x86_64.rpm
```

#### For binary archives:
```bash
# Download archive and signature
wget https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/rustycommit-linux-x86_64.tar.gz
wget https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/rustycommit-linux-x86_64.tar.gz.asc

# Verify signature
gpg --verify rustycommit-linux-x86_64.tar.gz.asc rustycommit-linux-x86_64.tar.gz
```

## Understanding Signature Output

### Cosign Output

#### Good signature:
```
Verified OK
Certificate subject: https://github.com/hongkongkiwi/rusty-commit/.github/workflows/release.yml@refs/tags/v1.0.2
Certificate issuer: https://token.actions.githubusercontent.com
```

This confirms:
- The binary was signed by the official GitHub Actions workflow
- The signature is cryptographically valid
- The signing identity matches the expected repository

#### Failed verification:
```
Error: verifying blob: failed to verify signature
```
**DO NOT USE** - The file may have been tampered with or the signature is invalid.

### GPG Output

#### Good signature:
```
gpg: Signature made [date]
gpg: using RSA key [KEY_ID]
gpg: Good signature from "Rusty Commit Release Bot <releases@rustycommit.example.com>"
```

#### Warning about trust:
```
gpg: WARNING: This key is not certified with a trusted signature!
```
This is normal if you haven't marked the key as trusted. The signature is still valid.

#### Bad signature:
```
gpg: BAD signature from [...]
```
**DO NOT USE** - The file has been modified or corrupted.

## Trust Levels

To mark the Rusty Commit signing key as trusted:

```bash
# Edit key trust
gpg --edit-key [KEY_ID]
gpg> trust
# Select 3 (marginally trusted) or 4 (fully trusted)
gpg> quit
```

## Platform-Specific Verification

### macOS (if code-signed)
```bash
codesign --verify --verbose rco
```

### Windows (if code-signed)
```powershell
Get-AuthenticodeSignature .\rco.exe
```

## Security Benefits

### Why Multiple Signature Methods?

1. **Cosign/Sigstore** - Modern keyless signing
   - No long-lived keys to manage or compromise
   - Transparent logging via Rekor
   - Identity-based trust (GitHub Actions OIDC)
   - Industry standard for container and binary signing

2. **GPG Signatures** - Traditional cryptographic signing
   - Widely supported and understood
   - Works offline without external dependencies
   - Backward compatibility with existing tools

3. **GitHub Attestations** - Supply chain security
   - Proves artifacts were built by GitHub Actions
   - Provides build provenance
   - SLSA Level 3 compliance

## Security Notes

1. **Always verify signatures** when downloading packages manually
2. **Prefer Cosign verification** for binaries (follows Rust ecosystem best practices)
3. **Check the certificate identity** matches the official repository
4. **Use HTTPS only** for downloads
5. **Report suspicious signatures** via GitHub issues

## Troubleshooting

### "No public key" error
- Ensure you've imported the public key
- Check if the key ID matches

### "Can't check signature: No public key"
```bash
# Import the key first
curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/.github/rusty-commit-public-key.asc | gpg --import
```

### Checksum mismatch
- Re-download the file (might be corrupted)
- Ensure you're checking the right file version
- Check for CRLF/LF line ending issues

## Contact

For security concerns, please email security@[domain] or open a GitHub issue.