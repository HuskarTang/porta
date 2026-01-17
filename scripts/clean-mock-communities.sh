#!/bin/bash
# Clean mock communities from Porta database files
# This script removes the test/mock communities that were previously seeded

set -e

DB_FILES=(
    "./server/porta.db"
    "./server/porta2.db"
    "./backend/data/porta.db"
    "./porta.db"
)

MOCK_IDS=("dev-community" "data-team" "test-env")

echo "Cleaning mock communities from database files..."

for db in "${DB_FILES[@]}"; do
    if [ -f "$db" ]; then
        echo "Processing: $db"
        for id in "${MOCK_IDS[@]}"; do
            sqlite3 "$db" "DELETE FROM communities WHERE id = '$id';" 2>/dev/null || true
        done
        count=$(sqlite3 "$db" "SELECT COUNT(*) FROM communities WHERE id IN ('dev-community', 'data-team', 'test-env');" 2>/dev/null || echo "0")
        if [ "$count" = "0" ]; then
            echo "  ✅ Mock communities removed from $db"
        else
            echo "  ⚠️  Warning: $count mock communities still exist in $db"
        fi
    fi
done

echo "Done!"
