#!/bin/bash
# =============================================================================
# Porta Release Script
# =============================================================================
# This script helps prepare a GitHub release:
# - Bumps version in package.json, backend, server, and src-tauri manifests
# - Adds an entry to CHANGELOG.md
# - Creates a git tag
#
# Usage:
#   ./scripts/release.sh 0.1.0
# =============================================================================

set -e

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

VERSION="$1"
DATE="$(date +%Y-%m-%d)"

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  exit 1
fi

echo "Preparing release v$VERSION..."

update_version() {
  local file="$1"
  if [ ! -f "$file" ]; then
    echo "File not found: $file"
    exit 1
  fi
  if grep -q "\"version\": \"[^\"]*\"" "$file"; then
    sed -i.bak "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$file"
    rm -f "$file.bak"
  elif grep -q "^version = " "$file"; then
    sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" "$file"
    rm -f "$file.bak"
  fi
}

# Update versions
update_version "$ROOT_DIR/package.json"
update_version "$ROOT_DIR/backend/Cargo.toml"
update_version "$ROOT_DIR/server/Cargo.toml"
update_version "$ROOT_DIR/src-tauri/Cargo.toml"

# Update changelog
CHANGELOG="$ROOT_DIR/CHANGELOG.md"
if [ -f "$CHANGELOG" ]; then
  awk -v ver="$VERSION" -v date="$DATE" '
    BEGIN { added=0 }
    /^## \\[Unreleased\\]/ {
      print
      print ""
      print "## [" ver "] - " date
      added=1
      next
    }
    { print }
    END {
      if (!added) {
        print "## [" ver "] - " date
      }
    }
  ' "$CHANGELOG" > "$CHANGELOG.tmp"
  mv "$CHANGELOG.tmp" "$CHANGELOG"
fi

git add "$ROOT_DIR/package.json" \
        "$ROOT_DIR/backend/Cargo.toml" \
        "$ROOT_DIR/server/Cargo.toml" \
        "$ROOT_DIR/src-tauri/Cargo.toml" \
        "$ROOT_DIR/CHANGELOG.md"

git commit -m "chore(release): v$VERSION"
git tag "v$VERSION"

echo "Release v$VERSION prepared."
echo "Next steps:"
echo "  git push origin main --tags"
