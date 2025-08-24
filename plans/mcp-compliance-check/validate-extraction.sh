#!/bin/bash
# Quick validation script for MCP extraction progress

set -e

echo "🔍 MCP Extraction Validation"
echo "============================"

# Check if MCP crate exists
if [ -d "../../shadowcat/crates/mcp" ]; then
    echo "✅ MCP crate directory exists"
    cd ../../shadowcat/crates/mcp
else
    echo "❌ MCP crate not found at shadowcat/crates/mcp"
    echo "   Run B.0 task first"
    exit 1
fi

# Check if it compiles
echo -n "🔨 Checking compilation... "
if cargo check --quiet 2>/dev/null; then
    echo "✅"
else
    echo "❌"
    echo "   Fix compilation errors before continuing"
    exit 1
fi

# Check which modules exist
echo ""
echo "📦 Extracted Modules:"
for module in types messages constants version builder parser client server; do
    if [ -f "src/$module.rs" ]; then
        echo "  ✅ $module.rs"
    else
        echo "  ⏳ $module.rs (not yet)"
    fi
done

# Run tests if they exist
echo ""
echo "🧪 Running tests..."
if cargo test --quiet 2>/dev/null; then
    TEST_COUNT=$(cargo test 2>&1 | grep -E "^test result:" | grep -oE "[0-9]+ passed" | grep -oE "[0-9]+")
    echo "✅ $TEST_COUNT tests passing"
else
    echo "⚠️  Tests failing or not found"
fi

# Check milestone tests
echo ""
echo "🎯 Milestone Status:"
for i in 1 2 3 4 5; do
    if cargo test milestone_$i --quiet 2>/dev/null; then
        echo "  ✅ Milestone $i"
    else
        echo "  ⏳ Milestone $i"
    fi
done

# Check if fixtures parse
echo ""
echo "📋 Fixture Validation:"
if [ -d "../../../plans/mcp-compliance-check/fixtures" ]; then
    FIXTURE_COUNT=$(ls ../../../plans/mcp-compliance-check/fixtures/*.json 2>/dev/null | wc -l)
    echo "  Found $FIXTURE_COUNT fixtures"
    
    # Try to parse with jq
    for fixture in ../../../plans/mcp-compliance-check/fixtures/*.json; do
        if jq empty "$fixture" 2>/dev/null; then
            echo "  ✅ $(basename $fixture)"
        else
            echo "  ❌ $(basename $fixture) - Invalid JSON"
        fi
    done
else
    echo "  ⚠️  No fixtures found"
fi

echo ""
echo "📊 Summary:"
echo "========="

# Count progress
DONE_COUNT=$(ls src/*.rs 2>/dev/null | wc -l)
echo "  Modules extracted: $DONE_COUNT"
echo "  Tests passing: ${TEST_COUNT:-0}"
echo "  Next task: Check tracker for what's next"

echo ""
echo "💡 Quick next steps:"
echo "  1. cd ../../shadowcat/crates/mcp"
echo "  2. cargo test --test milestones"
echo "  3. Check plans/mcp-compliance-check/mcp-compliance-check-tracker.md"