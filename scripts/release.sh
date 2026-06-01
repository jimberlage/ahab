#!/usr/bin/env bash
set -euo pipefail

# Release script for ahab
# This script creates a git tag which triggers the GitHub Actions release workflow.
#
# Usage: ./scripts/release.sh <version>
# Example: ./scripts/release.sh v1.0.1

if [ $# -eq 0 ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 v1.0.1"
    exit 1
fi

VERSION="$1"
TAG="v${VERSION}"

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "Error: You have uncommitted changes. Please commit or stash them first."
    exit 1
fi

# Check if tag already exists
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Error: Tag $TAG already exists."
    exit 1
fi

echo "==> Creating release $TAG"
echo ""
echo "This will:"
echo "  1. Create and push git tag $TAG"
echo "  2. Trigger GitHub Actions to build all platforms"
echo "  3. Create a GitHub release with the binaries"
echo ""
read -p "Continue? [y/N] " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
fi

# Create and push tag
git tag -a "$TAG" -m "Release $TAG"
git push origin "$TAG"

echo ""
echo "==> Tag $TAG pushed!"
echo ""
echo "GitHub Actions is now building the release."
echo "Watch progress at: https://github.com/jimberlage/ahab/actions"
echo ""
echo "Once complete, update Formula/ahab.rb with the SHA256 values from the release notes."