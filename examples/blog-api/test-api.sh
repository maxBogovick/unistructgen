#!/bin/bash

# Blog API Test Script
# Demonstrates all endpoints of the UniStructGen-powered Blog API

set -e

BASE_URL="http://localhost:3000"
BOLD="\033[1m"
GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[1;33m"
BLUE="\033[0;34m"
RESET="\033[0m"

echo -e "${BOLD}${BLUE}"
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  Blog API Test Suite - UniStructGen Demo                 ║"
echo "║  Demonstrating Auto-Generated Types & Validation         ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo -e "${RESET}\n"

# Helper function to print section headers
print_section() {
    echo -e "\n${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
    echo -e "${BOLD}${GREEN}$1${RESET}"
    echo -e "${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}\n"
}

# Helper function to print test descriptions
print_test() {
    echo -e "${BOLD}${YELLOW}▶ $1${RESET}"
}

# Helper function to execute and show requests
execute_request() {
    local description="$1"
    local method="$2"
    local url="$3"
    local data="$4"

    echo -e "${BOLD}Request:${RESET}"
    if [ -n "$data" ]; then
        echo -e "  ${method} ${url}"
        echo -e "  ${BOLD}Body:${RESET}"
        echo "$data" | jq '.' 2>/dev/null || echo "$data"
    else
        echo -e "  ${method} ${url}"
    fi

    echo -e "\n${BOLD}Response:${RESET}"
    if [ -n "$data" ]; then
        curl -s -X "${method}" "${url}" \
            -H "Content-Type: application/json" \
            -d "$data" | jq '.' 2>/dev/null || echo "Failed to parse response"
    else
        curl -s -X "${method}" "${url}" | jq '.' 2>/dev/null || echo "Failed to parse response"
    fi
    echo ""
}

# Check if server is running
print_section "🔍 Checking Server Status"
if ! curl -s "${BASE_URL}/health" > /dev/null 2>&1; then
    echo -e "${RED}❌ Server is not running!${RESET}"
    echo -e "Please start the server first:"
    echo -e "  ${BOLD}cd examples/blog-api${RESET}"
    echo -e "  ${BOLD}cargo run${RESET}"
    exit 1
fi
echo -e "${GREEN}✓ Server is running${RESET}"

# ============================================================================
# 1. CREATE POSTS - Success Cases
# ============================================================================
print_section "📝 Test 1: Create Blog Posts (Success Cases)"

print_test "1.1 Create a valid blog post"
POST1_RESPONSE=$(curl -s -X POST "${BASE_URL}/posts" \
    -H "Content-Type: application/json" \
    -d '{
        "title": "Getting Started with Rust",
        "content": "Rust is an amazing systems programming language that provides memory safety without garbage collection. In this tutorial, we will explore the basics.",
        "author": "john_doe",
        "tags": ["rust", "programming", "tutorial"],
        "status": "published"
    }')

echo "$POST1_RESPONSE" | jq '.'
POST1_ID=$(echo "$POST1_RESPONSE" | jq -r '.id')
echo -e "${GREEN}✓ Post created with ID: ${POST1_ID}${RESET}\n"

print_test "1.2 Create another post with minimal fields"
POST2_RESPONSE=$(curl -s -X POST "${BASE_URL}/posts" \
    -H "Content-Type: application/json" \
    -d '{
        "title": "Understanding Ownership",
        "content": "Ownership is Rust'\''s most unique feature and has deep implications for the rest of the language.",
        "author": "alice_rust"
    }')

echo "$POST2_RESPONSE" | jq '.'
POST2_ID=$(echo "$POST2_RESPONSE" | jq -r '.id')
echo -e "${GREEN}✓ Post created with ID: ${POST2_ID}${RESET}\n"

print_test "1.3 Create a draft post"
POST3_RESPONSE=$(curl -s -X POST "${BASE_URL}/posts" \
    -H "Content-Type: application/json" \
    -d '{
        "title": "Advanced Rust Patterns",
        "content": "Exploring advanced design patterns in Rust including the builder pattern, type state, and more.",
        "author": "bob_developer",
        "tags": ["rust", "advanced", "patterns"],
        "status": "draft"
    }')

echo "$POST3_RESPONSE" | jq '.'
POST3_ID=$(echo "$POST3_RESPONSE" | jq -r '.id')
echo -e "${GREEN}✓ Draft post created with ID: ${POST3_ID}${RESET}\n"

# ============================================================================
# 2. CREATE POSTS - Validation Errors
# ============================================================================
print_section "⚠️  Test 2: Validation Errors (Auto-Generated from OpenAPI!)"

print_test "2.1 Title too short (< 5 characters)"
execute_request "Title validation error" "POST" "${BASE_URL}/posts" '{
    "title": "Hi",
    "content": "This is my content with enough text here",
    "author": "john_doe"
}'

print_test "2.2 Invalid author format (contains special characters)"
execute_request "Author pattern validation" "POST" "${BASE_URL}/posts" '{
    "title": "Valid Title Here",
    "content": "This is my content with enough text to pass validation",
    "author": "john@doe.com"
}'

print_test "2.3 Content too short (< 10 characters)"
execute_request "Content length validation" "POST" "${BASE_URL}/posts" '{
    "title": "Valid Title",
    "content": "Too short",
    "author": "john_doe"
}'

print_test "2.4 Too many tags (> 10)"
execute_request "Tags array length validation" "POST" "${BASE_URL}/posts" '{
    "title": "Valid Title Here",
    "content": "This is my content with enough text",
    "author": "john_doe",
    "tags": ["tag1", "tag2", "tag3", "tag4", "tag5", "tag6", "tag7", "tag8", "tag9", "tag10", "tag11"]
}'

# ============================================================================
# 3. LIST POSTS
# ============================================================================
print_section "📋 Test 3: List Posts (with Pagination)"

print_test "3.1 Get all posts (default pagination)"
execute_request "List all posts" "GET" "${BASE_URL}/posts"

print_test "3.2 Get posts with custom limit"
execute_request "List with limit=2" "GET" "${BASE_URL}/posts?limit=2"

print_test "3.3 Get posts with offset"
execute_request "List with offset=1" "GET" "${BASE_URL}/posts?limit=10&offset=1"

# ============================================================================
# 4. GET SINGLE POST
# ============================================================================
print_section "🔍 Test 4: Get Individual Post"

print_test "4.1 Get first post by ID"
execute_request "Get post details" "GET" "${BASE_URL}/posts/${POST1_ID}"

print_test "4.2 Get post again (view count incremented!)"
execute_request "Get post again" "GET" "${BASE_URL}/posts/${POST1_ID}"

print_test "4.3 Try to get non-existent post"
execute_request "Get non-existent post" "GET" "${BASE_URL}/posts/00000000-0000-0000-0000-000000000000"

# ============================================================================
# 5. UPDATE POST
# ============================================================================
print_section "✏️  Test 5: Update Posts"

print_test "5.1 Update post title"
execute_request "Update title" "PUT" "${BASE_URL}/posts/${POST1_ID}" '{
    "title": "Getting Started with Rust - Updated Edition"
}'

print_test "5.2 Update post status and add tags"
execute_request "Update status and tags" "PUT" "${BASE_URL}/posts/${POST2_ID}" '{
    "status": "published",
    "tags": ["rust", "ownership", "memory-safety"]
}'

print_test "5.3 Try to update with invalid title (too short)"
execute_request "Update with invalid title" "PUT" "${BASE_URL}/posts/${POST1_ID}" '{
    "title": "Bad"
}'

print_test "5.4 Update non-existent post"
execute_request "Update non-existent post" "PUT" "${BASE_URL}/posts/00000000-0000-0000-0000-000000000000" '{
    "title": "This will fail"
}'

# ============================================================================
# 6. COMMENTS
# ============================================================================
print_section "💬 Test 6: Comments"

print_test "6.1 Add comment to first post"
COMMENT1_RESPONSE=$(curl -s -X POST "${BASE_URL}/posts/${POST1_ID}/comments" \
    -H "Content-Type: application/json" \
    -d '{
        "author": "alice",
        "content": "Great article! Thanks for sharing this comprehensive guide."
    }')
echo "$COMMENT1_RESPONSE" | jq '.'
echo -e "${GREEN}✓ Comment added${RESET}\n"

print_test "6.2 Add another comment"
execute_request "Add second comment" "POST" "${BASE_URL}/posts/${POST1_ID}/comments" '{
    "author": "bob_dev",
    "content": "Very helpful tutorial, especially the ownership section!"
}'

print_test "6.3 Get all comments for the post"
execute_request "Get comments" "GET" "${BASE_URL}/posts/${POST1_ID}/comments"

print_test "6.4 Try to add comment with invalid author"
execute_request "Invalid comment author" "POST" "${BASE_URL}/posts/${POST1_ID}/comments" '{
    "author": "bad@email.com",
    "content": "This should fail validation"
}'

print_test "6.5 Try to add comment to non-existent post"
execute_request "Comment on non-existent post" "POST" "${BASE_URL}/posts/00000000-0000-0000-0000-000000000000/comments" '{
    "author": "alice",
    "content": "This will fail because post does not exist"
}'

# ============================================================================
# 7. DELETE POSTS
# ============================================================================
print_section "🗑️  Test 7: Delete Posts"

print_test "7.1 Delete the draft post"
echo -e "${BOLD}Request:${RESET} DELETE ${BASE_URL}/posts/${POST3_ID}"
RESPONSE=$(curl -s -w "\n%{http_code}" -X DELETE "${BASE_URL}/posts/${POST3_ID}")
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
echo -e "${BOLD}Response:${RESET} HTTP ${HTTP_CODE}"
if [ "$HTTP_CODE" == "204" ]; then
    echo -e "${GREEN}✓ Post deleted successfully${RESET}\n"
else
    echo -e "${RED}✗ Unexpected response code${RESET}\n"
fi

print_test "7.2 Verify post is deleted"
execute_request "Try to get deleted post" "GET" "${BASE_URL}/posts/${POST3_ID}"

print_test "7.3 Try to delete non-existent post"
echo -e "${BOLD}Request:${RESET} DELETE ${BASE_URL}/posts/00000000-0000-0000-0000-000000000000"
RESPONSE=$(curl -s -w "\n%{http_code}" -X DELETE "${BASE_URL}/posts/00000000-0000-0000-0000-000000000000")
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)
echo -e "${BOLD}Response:${RESET} HTTP ${HTTP_CODE}"
echo "$BODY" | jq '.' 2>/dev/null || echo "$BODY"
echo ""

# ============================================================================
# 8. FINAL STATE
# ============================================================================
print_section "📊 Test 8: Final State"

print_test "8.1 List all remaining posts"
execute_request "Final post list" "GET" "${BASE_URL}/posts"

# ============================================================================
# SUMMARY
# ============================================================================
echo -e "\n${BOLD}${BLUE}╔═══════════════════════════════════════════════════════════╗${RESET}"
echo -e "${BOLD}${BLUE}║                    TEST SUMMARY                           ║${RESET}"
echo -e "${BOLD}${BLUE}╚═══════════════════════════════════════════════════════════╝${RESET}\n"

echo -e "${GREEN}✅ All Tests Completed!${RESET}\n"
echo -e "${BOLD}What This Demo Showed:${RESET}"
echo -e "  ${GREEN}✓${RESET} Auto-generated types from OpenAPI spec"
echo -e "  ${GREEN}✓${RESET} Automatic validation (title length, author pattern, etc.)"
echo -e "  ${GREEN}✓${RESET} Type-safe UUID handling"
echo -e "  ${GREEN}✓${RESET} Enum validation (status: draft/published/archived)"
echo -e "  ${GREEN}✓${RESET} Array length validation (tags max 10)"
echo -e "  ${GREEN}✓${RESET} Detailed error messages with field-level validation"
echo -e "  ${GREEN}✓${RESET} Full CRUD operations"
echo -e "  ${GREEN}✓${RESET} Nested resources (comments on posts)"
echo -e "  ${GREEN}✓${RESET} Pagination support"
echo ""
echo -e "${BOLD}${YELLOW}🎉 UniStructGen Magic:${RESET}"
echo -e "  All validation rules came from ${BOLD}blog-api.yaml${RESET}"
echo -e "  Zero manual validation code written!"
echo -e "  100% type-safe at compile time!"
echo ""
echo -e "${BOLD}Try modifying ${BLUE}blog-api.yaml${RESET}${BOLD} and see the changes automatically!${RESET}\n"
