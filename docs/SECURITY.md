# Security Policy

## Installation Security

### SHA256 Checksum Verification

All release artifacts include SHA256 checksums that are automatically verified during installation:

1. **Checksum Files**: Each release includes a `SHA256SUMS.txt` file containing checksums for all artifacts
2. **Automatic Verification**: The install script automatically downloads and verifies checksums
3. **Manual Verification**: You can manually verify downloads:
   ```bash
   # Download the checksum file
   curl -LO https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/SHA256SUMS.txt

   # Verify your download
   sha256sum -c SHA256SUMS.txt --ignore-missing
   ```

### GPG Signature Verification (Future)

We plan to implement GPG signing for enhanced security:

1. **Signed Checksums**: The `SHA256SUMS.txt.asc` file will contain GPG signature
2. **Package Signing**: Individual packages (.deb, .rpm) will be signed
3. **Verification**: Instructions will be provided for verifying signatures

### Installation Script Security

The installation script includes multiple security features:

1. **Root Protection**: Warns when running as root (override with `ACCEPT_RISKS=true`)
2. **Environment Checks**: Detects suspicious environment variables (LD_PRELOAD, etc.)
3. **Checksum Verification**: Verifies all downloads by default
4. **HTTPS Only**: All downloads use HTTPS with retry logic
5. **Temp Directory**: Uses secure temporary directories with automatic cleanup
6. **Error Handling**: Comprehensive error handling with rollback on failure

### Secure Credential Storage

Rusty Commit supports secure credential storage using system keychains:

- **macOS**: Keychain Access
- **Linux**: Secret Service API (GNOME Keyring, KWallet)
- **Windows**: Windows Credential Manager

API keys are never stored in plain text when secure storage is enabled.

## Reporting Security Vulnerabilities

If you discover a security vulnerability, please:

1. **DO NOT** open a public issue
2. Email security details to [your-security-email@example.com]
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

We will acknowledge receipt within 48 hours and provide updates on the fix.

## Security Best Practices

### For Users

1. **Always verify checksums** when downloading binaries manually
2. **Use secure storage** features for API keys (`--features secure-storage`)
3. **Keep software updated** to get security patches
4. **Review scripts** before piping to bash
5. **Use official sources** for installation

### For Contributors

1. **Never commit secrets** or API keys
2. **Use environment variables** for sensitive data
3. **Follow secure coding practices**
4. **Update dependencies** regularly
5. **Test security features** before releases

## Dependency Security

We use several measures to ensure dependency security:

1. **Cargo audit**: Regular security audits of dependencies
2. **Dependabot**: Automated dependency updates
3. **Minimal dependencies**: We minimize external dependencies
4. **Trusted crates**: Only use well-maintained, popular crates

## Build Security

Our CI/CD pipeline includes:

1. **Protected branches**: Main branch requires reviews
2. **Signed commits**: Encourage GPG-signed commits
3. **CI security checks**: Automated security scanning
4. **Release artifacts**: Built in clean CI environment
5. **Reproducible builds**: Working towards reproducible builds

## Future Security Enhancements

Planned security improvements:

1. **GPG signing** for all release artifacts
2. **Signed commits** enforcement
3. **Security audit** automation
4. **SLSA compliance** for supply chain security
5. **Reproducible builds** verification
6. **Code signing** for Windows/macOS binaries
