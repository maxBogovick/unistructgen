#!/bin/bash

# Example usage of Schema Registry CLI

set -e

echo "=== Schema Registry Demo ==="
echo

# 1. Create a team
echo "1. Creating team..."
schema-registry team create backend --description "Backend team"
echo

# 2. Upload schema
echo "2. Uploading Pet Store schema..."
schema-registry upload \
  --name petstore \
  --version 1.0.0 \
  --file examples/petstore.yaml \
  --team backend \
  --description "Pet Store API v1"
echo

# 3. List schemas
echo "3. Listing all schemas..."
schema-registry list
echo

# 4. Get schema details
echo "4. Getting schema details..."
schema-registry get petstore
echo

# 5. Upload new version
echo "5. Uploading new version (simulating change)..."
schema-registry upload \
  --name petstore \
  --version 2.0.0 \
  --file examples/petstore.yaml \
  --team backend
echo

# 6. List versions
echo "6. Listing versions..."
schema-registry versions petstore
echo

# 7. Compare versions
echo "7. Comparing versions..."
schema-registry diff petstore --from 1.0.0 --to 2.0.0
echo

# 8. Generate code
echo "8. Generating Rust code..."
schema-registry generate petstore \
  --version 1.0.0 \
  --targets rust \
  --output ./generated
echo

# 9. Show statistics
echo "9. Showing statistics..."
schema-registry stats
echo

echo "=== Demo complete! ==="
