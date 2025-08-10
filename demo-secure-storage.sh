#!/bin/bash

echo "🔐 Rusty Commit Secure Storage Demo"
echo "=================================="
echo

# Build with secure storage
echo "📦 Building with secure storage support..."
cargo build --release --features secure-storage --quiet

echo "✅ Build complete!"
echo

# Show status
echo "📊 Checking secure storage status..."
./target/release/rco config status
echo

# Demo setting an API key
echo "🔑 Demo: Setting API key (will be stored securely if available)..."
echo "   Run: rco config set RCO_API_KEY=sk-your-key-here"
echo

echo "📝 Notes:"
echo "  - On macOS: Keys stored in Keychain"
echo "  - On Linux: Keys stored in GNOME Keyring/KWallet"
echo "  - On Windows: Keys stored in Credential Manager"
echo "  - Fallback: Keys stored in ~/.config/rustycommit/ if keychain unavailable"
