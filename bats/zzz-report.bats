#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
  login_superadmin
  login_lanacli
}

teardown_file() {
  stop_server
}

find_report_run() {
  local cli_output
  cli_output=$("$LANACLI" --json report list --first 1)
  run_id=$(echo "$cli_output" | jq -r '.[0].reportRunId')
  [[ "$run_id" != "null" && -n "$run_id" ]] || return 1
}

wait_for_report_run_complete() {
  local run_id=$1
  local cli_output
  cli_output=$("$LANACLI" --json report find --id "$run_id")
  state=$(echo "$cli_output" | jq -r '.state')
  [[ "$state" == "SUCCESS" || "$state" == "FAILED" ]] || return 1
}

@test "report: trigger run, wait for completion, and download files" {
  if [[ "${DAGSTER:-}" != "true" ]]; then
    skip "Skipping report test because DAGSTER is not enabled"
  fi
  if [[ -z "${SA_CREDS_BASE64:-}" ]]; then
    skip "Skipping report test because SA_CREDS_BASE64 is not defined"
  fi

  # Trigger a report run
  local cli_output
  cli_output=$("$LANACLI" --json report trigger)
  echo "triggerReportRun response: $cli_output"

  # Poll for the report run to appear (async creation, may take a while)
  retry 60 5 find_report_run
  cli_output=$("$LANACLI" --json report list --first 1)
  echo "reportRuns response: $cli_output"
  run_id=$(echo "$cli_output" | jq -r '.[0].reportRunId')
  [[ "$run_id" != "null" && -n "$run_id" ]] || exit 1

  # Wait for the report run to reach a terminal state (up to ~8 minutes)
  retry 240 2 wait_for_report_run_complete "$run_id"

  # Fetch completed run details
  cli_output=$("$LANACLI" --json report find --id "$run_id")
  echo "Completed run: $cli_output"

  # Extract reports and their files, then generate download links
  reports_json=$(echo "$cli_output" | jq '.reports')
  reports_length=$(echo "$reports_json" | jq 'length')
  [[ "$reports_length" -gt 0 ]] || exit 1

  for i in $(seq 0 $((reports_length - 1))); do
    report_id=$(echo "$reports_json" | jq -r ".[$i].reportId")
    files_json=$(echo "$reports_json" | jq ".[$i].files")
    files_length=$(echo "$files_json" | jq 'length')
    [[ "$files_length" -gt 0 ]] || exit 1

    for j in $(seq 0 $((files_length - 1))); do
      extension=$(echo "$files_json" | jq -r ".[$j].extension")

      local link_output
      link_output=$("$LANACLI" --json report download-link --report-id "$report_id" --extension "$extension")
      echo "Download link response: $link_output"

      url=$(echo "$link_output" | jq -r '.url')
      [[ "$url" != "null" && -n "$url" ]] || exit 1

      # Handle both local file:// URLs and HTTP URLs
      if [[ "$url" == file://* ]]; then
        local_path="${url#file://}"
        [[ -f "$local_path" ]] || exit 1
        file_size=$(wc -c < "$local_path")
        [[ "$file_size" -gt 0 ]] || { echo "Local report file is empty: $local_path"; exit 1; }
        echo "Local file verified (${file_size} bytes): $local_path"
      else
        # When running with GCS, assert the URL is a real GCS signed URL
        if [[ -n "${DOCS_BUCKET_NAME:-}" ]]; then
          [[ "$url" == https://storage.googleapis.com/* ]] || {
            echo "Expected GCS signed URL (storage.googleapis.com) but got: $url"
            exit 1
          }
          echo "Confirmed GCS signed URL for report=${report_id} extension=${extension}"
        fi

        # Download the file and verify it is non-empty
        tmp_file=$(mktemp)
        http_code=$(curl -s -o "$tmp_file" -w "%{http_code}" "$url")
        [[ "$http_code" == "200" ]] || { echo "HTTP ${http_code} downloading report from URL: $url"; rm -f "$tmp_file"; exit 1; }
        file_size=$(wc -c < "$tmp_file")
        [[ "$file_size" -gt 0 ]] || { echo "Downloaded report file is empty (report=${report_id} extension=${extension})"; rm -f "$tmp_file"; exit 1; }
        echo "HTTP download verified (${file_size} bytes, HTTP ${http_code}): report=${report_id} extension=${extension}"
        rm -f "$tmp_file"
      fi
    done
  done
}
