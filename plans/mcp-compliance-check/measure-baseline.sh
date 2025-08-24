#!/bin/bash
# Measure shadowcat baseline performance before extraction

echo "📊 Shadowcat MCP Performance Baseline"
echo "===================================="
echo "Date: $(date)"
echo ""

# Navigate to main shadowcat (not worktree)
cd /Users/kevin/src/tapwire/shadowcat || exit 1

# Measure compilation time
echo "⏱️  Compilation Performance:"
echo -n "  Debug build: "
time -p cargo build --quiet 2>&1 | grep real | awk '{print $2 "s"}'

echo -n "  Release build: "
time -p cargo build --release --quiet 2>&1 | grep real | awk '{print $2 "s"}'

# Measure test time
echo ""
echo "🧪 Test Performance:"
echo -n "  MCP tests: "
time -p cargo test mcp:: --quiet 2>&1 | grep real | awk '{print $2 "s"}'

# Count lines of code
echo ""
echo "📏 Code Size Metrics:"
echo "  MCP module files:"
for file in src/mcp/*.rs; do
    if [ -f "$file" ]; then
        lines=$(wc -l < "$file")
        size=$(ls -lh "$file" | awk '{print $5}')
        echo "    $(basename $file): $lines lines, $size"
    fi
done

# Total MCP module size
total_lines=$(find src/mcp -name "*.rs" -exec cat {} \; | wc -l)
echo "  Total: $total_lines lines"

# Check for benchmarks
echo ""
echo "🏃 Benchmarks:"
if cargo bench --list 2>/dev/null | grep -q mcp; then
    cargo bench --bench mcp 2>/dev/null | tail -5
else
    echo "  No MCP benchmarks found"
fi

# Memory usage estimate
echo ""
echo "💾 Memory Usage (estimate):"
if command -v /usr/bin/time >/dev/null 2>&1; then
    /usr/bin/time -l cargo test mcp::types::tests --quiet 2>&1 | grep "maximum resident" | head -1
else
    echo "  Unable to measure (install GNU time)"
fi

# Save results
echo ""
echo "💾 Saving baseline to: baseline-$(date +%Y%m%d).txt"
echo "================================" > baseline-$(date +%Y%m%d).txt
echo "Shadowcat MCP Baseline" >> baseline-$(date +%Y%m%d).txt
echo "Date: $(date)" >> baseline-$(date +%Y%m%d).txt
echo "================================" >> baseline-$(date +%Y%m%d).txt
echo "" >> baseline-$(date +%Y%m%d).txt
echo "Files extracted:" >> baseline-$(date +%Y%m%d).txt
ls -la src/mcp/*.rs >> baseline-$(date +%Y%m%d).txt
echo "" >> baseline-$(date +%Y%m%d).txt
echo "Total lines: $total_lines" >> baseline-$(date +%Y%m%d).txt

echo ""
echo "✅ Baseline measurement complete!"
echo "   Use this to detect regressions after extraction"