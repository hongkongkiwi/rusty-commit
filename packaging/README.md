# Rusty Commit Packaging

This directory contains packaging configurations for various operating systems and package managers.

## Automated Package Creation

When a GitHub release is published, the following packages are automatically created and attached to the release:

### Linux Packages

#### Debian/Ubuntu (.deb)
- **Architectures**: amd64, arm64, armhf
- **Installation**: `sudo dpkg -i rusty-commit_VERSION_ARCH.deb`
- **Uninstallation**: `sudo apt remove rusty-commit`

#### RPM-based Systems (.rpm)
- **Architectures**: x86_64, aarch64
- **Distributions**: Fedora, RHEL, CentOS, openSUSE
- **Installation**: `sudo rpm -i rusty-commit-VERSION.ARCH.rpm`
- **Uninstallation**: `sudo rpm -e rusty-commit`

#### Alpine Linux (.apk)
- **Architectures**: x86_64 (musl)
- **Installation**: `sudo apk add --allow-untrusted rusty-commit-VERSION.apk`
- **Uninstallation**: `sudo apk del rusty-commit`

#### Snap Package (.snap)
- **Universal package** for all Linux distributions
- **Installation**: `sudo snap install --classic rusty-commit_VERSION.snap`
- **Uninstallation**: `sudo snap remove rusty-commit`

### Platform-Agnostic Archives

#### Tarball (.tar.gz)
- **Platforms**: Linux (x86_64, aarch64, armv7, musl), macOS (x86_64, aarch64)
- **Installation**: Extract and run `./install.sh`
- **Manual Installation**: Copy `rco` to `/usr/local/bin/`

#### Windows ZIP (.zip)
- **Architecture**: x86_64
- **Installation**: Extract and run `install.bat` or manually add to PATH

## Manual Package Building

### Prerequisites

```bash
# Debian/Ubuntu
sudo apt-get install dpkg-dev debhelper

# RPM-based
sudo yum install rpm-build rpmlint

# Alpine
sudo apk add alpine-sdk

# Snap
sudo snap install snapcraft --classic
```

### Building Packages Locally

#### Build .deb Package
```bash
VERSION=1.0.1
ARCH=amd64  # or arm64, armhf

# Create package structure
mkdir -p rusty-commit_${VERSION}_${ARCH}/{DEBIAN,usr/bin}
cp target/release/rco rusty-commit_${VERSION}_${ARCH}/usr/bin/
cp packaging/debian/control rusty-commit_${VERSION}_${ARCH}/DEBIAN/
sed -i "s/VERSION/${VERSION}/g" rusty-commit_${VERSION}_${ARCH}/DEBIAN/control
sed -i "s/ARCH/${ARCH}/g" rusty-commit_${VERSION}_${ARCH}/DEBIAN/control

# Build package
dpkg-deb --build rusty-commit_${VERSION}_${ARCH}
```

#### Build .rpm Package
```bash
VERSION=1.0.1
cp packaging/rpm/rusty-commit.spec ~/rpmbuild/SPECS/
sed -i "s/VERSION/${VERSION}/g" ~/rpmbuild/SPECS/rusty-commit.spec
cp target/release/rco ~/rpmbuild/SOURCES/

rpmbuild -bb ~/rpmbuild/SPECS/rusty-commit.spec
```

#### Build .apk Package
```bash
VERSION=1.0.1
cp packaging/alpine/APKBUILD .
sed -i "s/VERSION/${VERSION}/g" APKBUILD

abuild -r
```

## Package Contents

All packages include:
- The `rco` binary
- README documentation
- License file (MIT)
- Man page (when available)

## Dependencies

All packages declare a dependency on `git` since rusty-commit is a git commit message generator.

## Signing Packages

For production releases, packages should be signed:

- **.deb**: Use `dpkg-sig` or `debsigs`
- **.rpm**: Use `rpmsign` or `rpm --addsign`
- **.apk**: Use `abuild-sign`
- **.snap**: Automatically signed when published to Snap Store

## Distribution Channels

After creation, packages can be distributed via:

1. **GitHub Releases**: Automatically attached to releases
2. **Package Repositories**: 
   - APT repository for .deb
   - YUM/DNF repository for .rpm
   - Alpine package repository for .apk
3. **Snap Store**: For snap packages
4. **Homebrew**: Via tap or homebrew-core

## Testing Packages

Always test packages before release:

```bash
# Test installation
sudo dpkg -i package.deb  # or rpm -i, apk add, etc.

# Verify installation
which rco
rco --version

# Test functionality
rco auth
rco commit

# Test uninstallation
sudo apt remove rusty-commit  # or rpm -e, apk del, etc.
```

## Support

For packaging issues, please open an issue at:
https://github.com/hongkongkiwi/rusty-commit/issues