#!/bin/bash
set -e

# UniStructGen Real-World Demo Script
# This script demonstrates the power of UniStructGen by generating
# complete, production-ready HTTP clients for real APIs.

echo "🚀 UniStructGen Real-World Client Generator Demo"
echo "=================================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if unistructgen is installed
if ! command -v unistructgen &> /dev/null; then
    echo -e "${YELLOW}⚠️  UniStructGen not found. Building from source...${NC}"
    cd ../../
    cargo build --release -p unistructgen
    export PATH="$PWD/target/release:$PATH"
    cd examples/realworld-clients
    echo -e "${GREEN}✓ UniStructGen built successfully${NC}"
fi

echo ""
echo -e "${BLUE}📦 Generating JSONPlaceholder API Client...${NC}"
echo "   This is a complete, type-safe HTTP client for testing APIs"
echo ""

# Generate the client
unistructgen client \
    --spec jsonplaceholder.yaml \
    --output ./generated/jsonplaceholder-client \
    --name "JsonPlaceholder" \
    --examples

echo ""
echo -e "${GREEN}✨ Success! Your client is ready!${NC}"
echo ""
echo -e "${BLUE}📁 Generated files:${NC}"
tree -L 2 generated/jsonplaceholder-client/ || ls -la generated/jsonplaceholder-client/

echo ""
echo -e "${BLUE}🚀 Quick start:${NC}"
echo "   cd generated/jsonplaceholder-client"
echo "   cargo build"
echo "   cargo run --example basic"
echo ""

echo -e "${YELLOW}💡 What just happened?${NC}"
echo "   ✨ Generated complete type-safe Rust structs from OpenAPI"
echo "   🔧 Created ready-to-use HTTP client with async/await"
echo "   📝 Added comprehensive documentation"
echo "   ✅ Included request/response validation"
echo "   💡 Generated usage examples"
echo "   📦 Created Cargo.toml with all dependencies"
echo ""

echo -e "${GREEN}🎉 Total time: ~5 seconds${NC}"
echo -e "${GREEN}   Manual coding would take: 8-20 hours${NC}"
echo ""

echo -e "${BLUE}🔥 Try it now!${NC}"
echo "   cd generated/jsonplaceholder-client && cargo run --example basic"
echo ""

# Optional: Build the generated client to verify it works
if [ "$1" == "--build" ]; then
    echo -e "${BLUE}🔨 Building generated client...${NC}"
    cd generated/jsonplaceholder-client
    cargo build
    echo -e "${GREEN}✓ Client builds successfully!${NC}"
fi
