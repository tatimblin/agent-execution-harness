#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 [major|minor|patch|VERSION]"
    echo "  major: 1.0.0 -> 2.0.0"
    echo "  minor: 1.0.0 -> 1.1.0"
    echo "  patch: 1.0.0 -> 1.0.1"
    echo "  VERSION: specific version like 1.2.3"
    exit 1
}

if [ $# -ne 1 ]; then
    usage
fi

# Get current version from Cargo.toml
current_version=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "${YELLOW}Current version: $current_version${NC}"

# Calculate new version
case $1 in
    major)
        IFS='.' read -r major minor patch <<< "$current_version"
        new_version="$((major + 1)).0.0"
        ;;
    minor)
        IFS='.' read -r major minor patch <<< "$current_version"
        new_version="$major.$((minor + 1)).0"
        ;;
    patch)
        IFS='.' read -r major minor patch <<< "$current_version"
        new_version="$major.$minor.$((patch + 1))"
        ;;
    *)
        if [[ $1 =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
            new_version=$1
        else
            echo -e "${RED}Invalid version format${NC}"
            usage
        fi
        ;;
esac

echo -e "${YELLOW}New version: $new_version${NC}"
read -p "Continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
fi

# Update Cargo.toml version
echo -e "${YELLOW}Updating Cargo.toml...${NC}"
sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
rm Cargo.toml.bak

# Update Homebrew formula version
echo -e "${YELLOW}Updating Homebrew formula...${NC}"
sed -i.bak "s/version \".*\"/version \"$new_version\"/" agent-execution-harness.rb
rm agent-execution-harness.rb.bak

# Commit changes
echo -e "${YELLOW}Committing version bump...${NC}"
git add Cargo.toml Cargo.lock agent-execution-harness.rb
git commit -m "Bump version to $new_version"

# Create and push tag
echo -e "${YELLOW}Creating and pushing tag...${NC}"
git tag "v$new_version"
git push && git push --tags

echo -e "${YELLOW}GitHub Actions is now running...${NC}"
echo "This will automatically:"
echo "1. Build binaries for all platforms"
echo "2. Create GitHub release with binaries"
echo "3. Publish to crates.io"
echo "4. Update Homebrew formula with correct SHA256 hashes"
echo "5. Push updated formula to homebrew-tap repository"
echo
echo "Check progress: https://github.com/tatimblin/agent-execution-harness/actions"
echo
echo -e "${GREEN}✓ Fully automated release $new_version started!${NC}"
echo "No manual steps required - everything is automated! 🎉"