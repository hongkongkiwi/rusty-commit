# Installation Guide for Rusty Commit

## Quick Install

### Using the install script (recommended)
```bash
curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh | bash
```

### Manual Installation

1. Download the appropriate archive for your platform from the [releases page](https://github.com/hongkongkiwi/rusty-commit/releases)
2. Extract the archive
3. Move the binary to your PATH
4. (Optional) Install shell completions and man page

## Platform-Specific Instructions

### macOS / Linux

```bash
# Download and extract (replace URL with your platform)
curl -L https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/rustycommit-<platform>.tar.gz | tar xz

# Move binary to PATH
sudo mv rco /usr/local/bin/

# Install shell completions (optional)
# For Bash:
sudo cp completions/rco.bash /etc/bash_completion.d/

# For Zsh:
sudo cp completions/_rco /usr/share/zsh/site-functions/

# For Fish:
cp completions/rco.fish ~/.config/fish/completions/

# Install man page (optional)
sudo cp man/rco.1 /usr/local/share/man/man1/
sudo mandb
```

### Windows

1. Download the Windows zip file from the releases page
2. Extract the archive
3. Add the directory containing `rco.exe` to your PATH
4. (Optional) Install PowerShell completions

## Using Cargo

```bash
# Basic installation
cargo install rusty-commit

# With secure storage feature
cargo install rusty-commit --features secure-storage
```

## Building from Source

```bash
# Clone the repository
git clone https://github.com/hongkongkiwi/rusty-commit.git
cd rusty-commit

# Build release version
cargo build --release

# Binary will be at target/release/rco
```

## Verifying Installation

```bash
# Check version
rco --version

# Run configuration wizard
rco config wizard

# Authenticate with a provider
rco auth login
```

## Shell Completions

Shell completions are included in the release archives in the `completions/` directory:
- `rco.bash` - Bash completions
- `_rco` - Zsh completions
- `rco.fish` - Fish completions

## Troubleshooting

If you encounter issues:
1. Ensure the binary has execute permissions: `chmod +x /usr/local/bin/rco`
2. Check that the binary is in your PATH: `which rco`
3. For secure storage issues, ensure your system keychain is accessible
4. Report issues at: https://github.com/hongkongkiwi/rusty-commit/issues