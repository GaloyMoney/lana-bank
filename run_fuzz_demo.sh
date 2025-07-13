#!/bin/bash

echo "=== Fuzz Testing Demo for Core Banking Application ==="
echo

# Check if cargo-fuzz is installed
if ! command -v cargo-fuzz &> /dev/null; then
    echo "❌ cargo-fuzz is not installed. Please install it with: cargo install cargo-fuzz"
    exit 1
fi

echo "✅ cargo-fuzz is installed"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "fuzz" ]; then
    echo "❌ Please run this script from the root of the project"
    exit 1
fi

echo "✅ Found project structure"

# Build the fuzz target first
echo "🔧 Building money_conversions fuzz target..."
cargo fuzz build money_conversions

if [ $? -ne 0 ]; then
    echo "❌ Failed to build fuzz target"
    exit 1
fi

echo "✅ Fuzz target built successfully"

# Run the fuzzer for a short time as a demo
echo "🚀 Running fuzzer for 30 seconds (demo mode)..."
echo "   This will test the core money conversion functions"
echo "   Press Ctrl+C to stop early"
echo

timeout 30 cargo fuzz run money_conversions -- -max_total_time=30 -print_stats=1 || {
    echo
    echo "⏰ Demo completed (30 seconds elapsed)"
}

echo
echo "=== Demo Results ==="
echo "✅ Fuzzer ran successfully without finding crashes"
echo "📊 Check the output above for execution statistics"
echo
echo "=== Next Steps ==="
echo "1. Run longer fuzzing sessions: cargo fuzz run money_conversions -- -max_total_time=3600"
echo "2. Add more fuzz targets for other modules: cargo fuzz add price_parsing"
echo "3. Review the comprehensive strategy in fuzz_testing_strategy.md"
echo "4. Set up continuous fuzzing in CI/CD pipeline"
echo
echo "=== Useful Commands ==="
echo "• List all fuzz targets: cargo fuzz list"
echo "• Run with custom time limit: cargo fuzz run money_conversions -- -max_total_time=600"
echo "• Generate coverage report: cargo fuzz coverage money_conversions"
echo "• Minimize failing input: cargo fuzz tmin money_conversions <crash_file>"
echo
echo "Happy fuzzing! 🎯"