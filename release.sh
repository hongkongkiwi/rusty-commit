#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to display usage
usage() {
    echo "Usage: $0 [patch|minor|major|x.y.z]"
    echo ""
    echo "Examples:"
    echo "  $0 patch           # Bump patch version (0.1.0 -> 0.1.1)"
    echo "  $0 minor           # Bump minor version (0.1.0 -> 0.2.0)"
    echo "  $0 major           # Bump major version (0.1.0 -> 1.0.0)"
    echo "  $0 1.2.3           # Set specific version"
    exit 1
}

# Check if argument provided
if [ $# -eq 0 ]; then
    usage
fi

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
echo -e "${YELLOW}Current version: $CURRENT_VERSION${NC}"

# Parse version components
IFS='.' read -r -a VERSION_PARTS <<< "$CURRENT_VERSION"
MAJOR="${VERSION_PARTS[0]}"
MINOR="${VERSION_PARTS[1]}"
PATCH="${VERSION_PARTS[2]}"

# Determine new version based on input
case "$1" in
    patch)
        NEW_VERSION="$MAJOR.$MINOR.$((PATCH + 1))"
        ;;
    minor)
        NEW_VERSION="$MAJOR.$((MINOR + 1)).0"
        ;;
    major)
        NEW_VERSION="$((MAJOR + 1)).0.0"
        ;;
    [0-9]*.[0-9]*.[0-9]*)
        NEW_VERSION="$1"
        ;;
    *)
        echo -e "${RED}Invalid version type: $1${NC}"
        usage
        ;;
esac

echo -e "${GREEN}New version: $NEW_VERSION${NC}"

# Check if we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo -e "${YELLOW}Warning: Not on main branch (current: $CURRENT_BRANCH)${NC}"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}Error: You have uncommitted changes${NC}"
    echo "Please commit or stash them first."
    exit 1
fi

# Pull latest changes
echo "Pulling latest changes..."
git pull origin main

# Run tests first (single-threaded to avoid env var race conditions)
echo "Running tests..."
if ! cargo test --all-features --quiet -- --test-threads=1; then
    echo -e "${RED}Tests failed! Aborting release.${NC}"
    exit 1
fi

echo "Running clippy..."
if ! cargo clippy --all-features -- -D warnings 2>/dev/null; then
    echo -e "${RED}Clippy found issues! Aborting release.${NC}"
    exit 1
fi

# Update version in Cargo.toml
echo "Updating Cargo.toml..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
else
    # Linux
    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
fi

# Update Cargo.lock
echo "Updating Cargo.lock..."
cargo update -p rusty-commit

# Commit version bump
echo "Committing version bump..."
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to $NEW_VERSION

Preparing for release v$NEW_VERSION"

# Create and push tag
TAG="v$NEW_VERSION"
echo "Creating tag $TAG..."
git tag -a "$TAG" -m "Release $TAG

$(git log --pretty=format:"- %s" "v$CURRENT_VERSION"..HEAD 2>/dev/null | grep -v "bump version")"

# Push changes
echo "Pushing changes and tag..."
git push origin main
git push origin "$TAG"

echo ""
echo -e "${GREEN}✅ Release $TAG initiated successfully!${NC}"
echo ""
echo "Next steps:"
echo "1. Check GitHub Actions: https://github.com/hongkongkiwi/rusty-commit/actions"
echo "2. Once CI passes, the release will be created automatically"
echo "3. Binary artifacts will be uploaded to the GitHub release"
echo "4. Package will be published to crates.io"
echo ""
echo "Release URL will be: https://github.com/hongkongkiwi/rusty-commit/releases/tag/$TAG"
