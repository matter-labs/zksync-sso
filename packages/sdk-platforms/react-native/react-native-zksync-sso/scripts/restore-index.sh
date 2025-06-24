#!/bin/bash

INDEX_PATH="src/index.tsx"

echo "Restoring index.tsx using git..."

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "⚠️  Not in a git repository, skipping git restore"
    exit 0
fi

# Check if the file exists
if [ ! -f "$INDEX_PATH" ]; then
    echo "📁 index.tsx does not exist, attempting to restore from git..."
    # Try to restore the file from git
    if git checkout HEAD -- "$INDEX_PATH" 2>/dev/null; then
        echo "✅ index.tsx restored from git successfully!"
        exit 0
    else
        echo "⚠️  index.tsx not found in git repository, skipping restore"
        exit 0
    fi
fi

# File exists, check if it has changes
if git diff --quiet HEAD -- "$INDEX_PATH" 2>/dev/null; then
    echo "✅ index.tsx is already up to date with git"
    exit 0
fi

# File has changes, restore it from git
echo "🔄 index.tsx has changes, restoring from git..."
if git checkout HEAD -- "$INDEX_PATH"; then
    echo "✅ index.tsx restored from git successfully!"
else
    echo "❌ Error restoring index.tsx from git"
    exit 1
fi 