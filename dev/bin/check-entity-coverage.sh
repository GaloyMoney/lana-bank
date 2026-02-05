#!/usr/bin/env bash
set -euo pipefail

MIN_COVERAGE=50

echo "Running coverage analysis for entity files (minimum: ${MIN_COVERAGE}%)..."
echo ""

COVERAGE_JSON=$(SQLX_OFFLINE=true cargo llvm-cov nextest --json --locked -E 'kind(lib)')

FAILED=0
TOTAL=0

echo "=== Entity Coverage Report ==="
echo ""

while IFS=$'\t' read -r filename count covered percent; do
    TOTAL=$((TOTAL + 1))

    # Compare integer part of percent against threshold
    int_percent=${percent%%.*}
    if [[ "$int_percent" -ge "$MIN_COVERAGE" ]]; then
        echo "PASS: $filename (${covered}/${count} lines, ${percent}%)"
    else
        echo "FAIL: $filename (${covered}/${count} lines, ${percent}%)"
        FAILED=$((FAILED + 1))
    fi
done < <(echo "$COVERAGE_JSON" | jq -r '
    .data[0].files[]
    | select(.filename | test("core/.*entity\\.rs$"))
    | [.filename, (.summary.lines.count | tostring), (.summary.lines.covered | tostring), (.summary.lines.percent | tostring)]
    | @tsv
')

echo ""
echo "Total: $TOTAL entity files checked"
echo "Passed: $((TOTAL - FAILED))"
echo "Failed: $FAILED"

if [[ "$TOTAL" -eq 0 ]]; then
    echo ""
    echo "ERROR: No entity files found in coverage output"
    exit 1
fi

if [[ "$FAILED" -gt 0 ]]; then
    echo ""
    echo "ERROR: $FAILED entity file(s) have less than ${MIN_COVERAGE}% line coverage"
    exit 1
fi

echo ""
echo "All entity files meet ${MIN_COVERAGE}% line coverage!"
