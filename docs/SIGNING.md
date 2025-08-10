# Package Signing Setup Guide

This guide explains how to set up GPG signing for Rusty Commit releases.

## 1. Generate GPG Key (One-time Setup)

### Option A: Using GPG directly

```bash
# Generate a new GPG key
gpg --full-generate-key

# Choose:
# - RSA and RSA (default)
# - 4096 bits
# - Key expires in 2 years (recommended)
# - Real name: Rusty Commit Release Bot
# - Email: releases@rustycommit.example.com
# - Comment: Rusty Commit Release Signing Key
```

### Option B: Using a batch file (automated)

Create `gpg-batch.txt`:
```
%echo Generating GPG key for Rusty Commit
Key-Type: RSA
Key-Length: 4096
Subkey-Type: RSA
Subkey-Length: 4096
Name-Real: Rusty Commit Release Bot
Name-Email: releases@rustycommit.example.com
Name-Comment: Rusty Commit Release Signing Key
Expire-Date: 2y
%no-protection
%commit
%echo done
```

Then run:
```bash
gpg --batch --generate-key gpg-batch.txt
```

## 2. Export Keys

```bash
# Find your key ID
gpg --list-secret-keys --keyid-format=long
# Look for: sec   rsa4096/XXXXXXXXXXXXXXXX

# Export private key (KEEP THIS SECRET!)
gpg --armor --export-secret-keys XXXXXXXXXXXXXXXX > rusty-commit-signing-key.asc

# Export public key (share this publicly)
gpg --armor --export XXXXXXXXXXXXXXXX > rusty-commit-public-key.asc

# Export ownertrust (optional, for automation)
gpg --export-ownertrust > rusty-commit-ownertrust.txt
```

## 3. Add to GitHub Secrets

Go to your repository Settings → Secrets and variables → Actions, and add:

1. **GPG_PRIVATE_KEY**
   ```bash
   # Copy the entire content including headers
   cat rusty-commit-signing-key.asc | base64
   ```

2. **GPG_PASSPHRASE** (if you set one)
   - The passphrase for your GPG key
   - Leave empty if no passphrase

3. **GPG_KEY_ID**
   - The key ID (e.g., `XXXXXXXXXXXXXXXX`)

## 4. Publish Public Key

### Add to repository
Create `.github/rusty-commit-public-key.asc` and commit it.

### Upload to keyservers
```bash
# Upload to multiple keyservers for redundancy
gpg --keyserver hkps://keys.openpgp.org --send-keys XXXXXXXXXXXXXXXX
gpg --keyserver hkps://keyserver.ubuntu.com --send-keys XXXXXXXXXXXXXXXX
gpg --keyserver hkps://pgp.mit.edu --send-keys XXXXXXXXXXXXXXXX
```

### Add to GitHub profile
1. Go to https://github.com/settings/keys
2. Click "New GPG key"
3. Paste the public key content

## 5. Configure Workflows

The workflows are already configured to use these secrets. They will:
1. Import the GPG key
2. Sign the SHA256SUMS.txt file
3. Sign individual packages (.deb, .rpm)
4. Upload signatures with releases

## 6. Verify Signatures (for users)

Users can verify signatures:

```bash
# Import public key
curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/.github/rusty-commit-public-key.asc | gpg --import

# Or from keyserver
gpg --keyserver hkps://keys.openpgp.org --recv-keys XXXXXXXXXXXXXXXX

# Verify checksum file
gpg --verify SHA256SUMS.txt.asc SHA256SUMS.txt

# Verify package signature (example for .deb)
gpg --verify rusty-commit_1.0.2_amd64.deb.asc rusty-commit_1.0.2_amd64.deb
```

## 7. Package-Specific Signing

### Debian Packages (.deb)
```bash
# Sign with debsigs
debsigs --sign=origin rusty-commit_1.0.2_amd64.deb

# Or with dpkg-sig
dpkg-sig --sign builder rusty-commit_1.0.2_amd64.deb
```

### RPM Packages (.rpm)
```bash
# Add to ~/.rpmmacros
echo "%_signature gpg" >> ~/.rpmmacros
echo "%_gpg_name XXXXXXXXXXXXXXXX" >> ~/.rpmmacros

# Sign the package
rpm --addsign rusty-commit-1.0.2-1.x86_64.rpm
```

## Security Best Practices

1. **Key Storage**
   - Never commit private keys to repository
   - Use GitHub Secrets for CI/CD
   - Rotate keys every 2 years

2. **Key Protection**
   - Use a strong passphrase
   - Store backup in secure location
   - Consider using hardware security key for master key

3. **Verification**
   - Always verify signatures after generation
   - Test the full verification flow
   - Document key fingerprint in multiple places

4. **Revocation Certificate**
   ```bash
   gpg --gen-revoke XXXXXXXXXXXXXXXX > rusty-commit-revoke.asc
   # Store this securely offline!
   ```

## Troubleshooting

### "No secret key" error
- Ensure GPG_PRIVATE_KEY is properly base64 encoded
- Check that the key hasn't expired

### "Bad signature" error
- Verify you're using the correct public key
- Check file hasn't been modified after signing
- Ensure proper line endings (LF not CRLF)

### GitHub Actions issues
- Check secret names match exactly
- Verify base64 encoding is correct
- Look at workflow logs for import errors