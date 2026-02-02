#!/usr/bin/env bash
#
# Benchmark script to compare lld vs mold linker performance
# Run this from the workspace root after entering the nix shell
#

set -euo pipefail

WORKSPACE_ROOT="${WORKSPACE_ROOT:-$(pwd)}"
CARGO_CONFIG="$WORKSPACE_ROOT/.cargo/config.toml"
CARGO_CONFIG_BACKUP="$WORKSPACE_ROOT/.cargo/config.toml.backup"
RESULTS_FILE="$WORKSPACE_ROOT/linker-benchmark-results.txt"

# Number of benchmark runs
NUM_RUNS="${NUM_RUNS:-3}"

# Package to benchmark (use a smaller crate for quicker results, or full workspace)
BENCHMARK_TARGET="${BENCHMARK_TARGET:--p lana-cli}"

echo "========================================="
echo "Linker Benchmark: lld vs mold"
echo "========================================="
echo ""
echo "Configuration:"
echo "  Workspace: $WORKSPACE_ROOT"
echo "  Target: $BENCHMARK_TARGET"
echo "  Runs per linker: $NUM_RUNS"
echo ""

# Check if required tools are available
check_tools() {
    echo "Checking required tools..."
    
    if ! command -v clang &>/dev/null; then
        echo "ERROR: clang not found. Please enter nix develop shell first."
        exit 1
    fi
    
    if ! command -v lld &>/dev/null; then
        echo "ERROR: lld not found. Please enter nix develop shell first."
        exit 1
    fi
    
    if ! command -v mold &>/dev/null; then
        echo "ERROR: mold not found. Please enter nix develop shell first."
        exit 1
    fi
    
    echo "  clang: $(command -v clang)"
    echo "  lld: $(command -v lld)"
    echo "  mold: $(command -v mold)"
    echo ""
}

# Backup original config
backup_config() {
    if [[ -f "$CARGO_CONFIG" ]]; then
        cp "$CARGO_CONFIG" "$CARGO_CONFIG_BACKUP"
        echo "Backed up original cargo config"
    fi
}

# Restore original config
restore_config() {
    if [[ -f "$CARGO_CONFIG_BACKUP" ]]; then
        mv "$CARGO_CONFIG_BACKUP" "$CARGO_CONFIG"
        echo "Restored original cargo config"
    fi
}

# Set linker configuration
set_linker() {
    local linker="$1"
    
    if [[ "$linker" == "lld" ]]; then
        cat > "$CARGO_CONFIG" << 'EOF'
# Use lld linker for faster linking on all platforms
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.aarch64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
EOF
    elif [[ "$linker" == "mold" ]]; then
        cat > "$CARGO_CONFIG" << 'EOF'
# Use mold linker for faster linking on all platforms
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[target.aarch64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
EOF
    else
        echo "Unknown linker: $linker"
        exit 1
    fi
    
    echo "Set linker to: $linker"
}

# Clean build artifacts to force relink
clean_build() {
    echo "Cleaning build artifacts..."
    # Remove target directory binaries to force relinking
    # We keep the deps to avoid recompiling everything
    rm -rf target/debug/lana-cli target/debug/deps/lana_cli* 2>/dev/null || true
    rm -rf target/debug/incremental/lana_cli* 2>/dev/null || true
    rm -rf target/release/lana-cli target/release/deps/lana_cli* 2>/dev/null || true
    rm -rf target/release/incremental/lana_cli* 2>/dev/null || true
}

# Full clean for initial setup
full_clean() {
    echo "Full clean of target directory..."
    cargo clean 2>/dev/null || true
}

# Run a single benchmark
run_benchmark() {
    local linker="$1"
    local run_num="$2"
    
    echo ""
    echo "--- $linker benchmark run $run_num ---"
    
    # Clean artifacts to force relink
    clean_build
    
    # Time the build
    local start_time=$(date +%s.%N)
    
    SQLX_OFFLINE=true cargo build $BENCHMARK_TARGET --features mock-custodian,sumsub-testing 2>&1 | tail -5
    
    local end_time=$(date +%s.%N)
    local elapsed=$(echo "$end_time - $start_time" | bc)
    
    echo "  Time: ${elapsed}s"
    echo "$elapsed"
}

# Calculate average
calc_average() {
    local -a times=("$@")
    local sum=0
    for t in "${times[@]}"; do
        sum=$(echo "$sum + $t" | bc)
    done
    echo "scale=3; $sum / ${#times[@]}" | bc
}

# Main benchmark function
run_benchmarks() {
    local -a lld_times=()
    local -a mold_times=()
    
    # First, do a warm-up build to compile all dependencies
    echo "========================================="
    echo "Phase 1: Initial warm-up build (compiling dependencies)"
    echo "========================================="
    set_linker "lld"
    SQLX_OFFLINE=true cargo build $BENCHMARK_TARGET --features mock-custodian,sumsub-testing 2>&1 | tail -10
    
    echo ""
    echo "========================================="
    echo "Phase 2: Benchmarking lld linker"
    echo "========================================="
    set_linker "lld"
    
    for i in $(seq 1 $NUM_RUNS); do
        local time=$(run_benchmark "lld" "$i")
        lld_times+=("$time")
    done
    
    echo ""
    echo "========================================="
    echo "Phase 3: Benchmarking mold linker"
    echo "========================================="
    set_linker "mold"
    
    for i in $(seq 1 $NUM_RUNS); do
        local time=$(run_benchmark "mold" "$i")
        mold_times+=("$time")
    done
    
    # Calculate results
    local lld_avg=$(calc_average "${lld_times[@]}")
    local mold_avg=$(calc_average "${mold_times[@]}")
    local speedup=$(echo "scale=2; $lld_avg / $mold_avg" | bc)
    local diff=$(echo "scale=3; $lld_avg - $mold_avg" | bc)
    
    echo ""
    echo "========================================="
    echo "BENCHMARK RESULTS"
    echo "========================================="
    echo ""
    echo "lld times: ${lld_times[*]}"
    echo "mold times: ${mold_times[*]}"
    echo ""
    echo "lld average:  ${lld_avg}s"
    echo "mold average: ${mold_avg}s"
    echo ""
    echo "Difference: ${diff}s"
    echo "Speedup: ${speedup}x"
    echo ""
    
    if (( $(echo "$mold_avg < $lld_avg" | bc -l) )); then
        echo "RESULT: mold is faster by ${diff}s (${speedup}x speedup)"
    else
        echo "RESULT: lld is faster"
    fi
    
    # Save results to file
    {
        echo "========================================="
        echo "Linker Benchmark Results"
        echo "Date: $(date)"
        echo "========================================="
        echo ""
        echo "Configuration:"
        echo "  Target: $BENCHMARK_TARGET"
        echo "  Runs: $NUM_RUNS"
        echo ""
        echo "lld times: ${lld_times[*]}"
        echo "mold times: ${mold_times[*]}"
        echo ""
        echo "lld average:  ${lld_avg}s"
        echo "mold average: ${mold_avg}s"
        echo ""
        echo "Difference: ${diff}s"
        echo "Speedup: ${speedup}x"
        echo ""
        if (( $(echo "$mold_avg < $lld_avg" | bc -l) )); then
            echo "RESULT: mold is faster by ${diff}s (${speedup}x speedup)"
        else
            echo "RESULT: lld is faster"
        fi
    } > "$RESULTS_FILE"
    
    echo ""
    echo "Results saved to: $RESULTS_FILE"
}

# Cleanup on exit
cleanup() {
    echo ""
    echo "Cleaning up..."
    restore_config
}

trap cleanup EXIT

# Main execution
main() {
    check_tools
    backup_config
    run_benchmarks
}

main "$@"
