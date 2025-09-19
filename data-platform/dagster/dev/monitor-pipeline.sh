#!/bin/bash

# Dagster Pipeline Monitor Script
# Triggers the lana_to_dw_job and monitors its execution

set -e

# Configuration
DAGSTER_URL="http://localhost:3000/graphql"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TRIGGER_GQL="$SCRIPT_DIR/dagster-lana-pipeline-trigger.gql"
STATUS_GQL="$SCRIPT_DIR/dagster-lana-pipeline-status.gql"

# Pipeline configuration - easily changeable
PIPELINE_JOB_NAME="lana_to_dw_job"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to make GraphQL requests
make_graphql_request() {
    local query_file="$1"
    local variables="$2"
    
    # Replace PIPELINE_JOB_NAME placeholder in the query
    local query_content=$(cat "$query_file" | sed "s/PIPELINE_JOB_NAME_PLACEHOLDER/$PIPELINE_JOB_NAME/g")
    
    if [ -n "$variables" ]; then
        curl -s -X POST "$DAGSTER_URL" \
            -H "Content-Type: application/json" \
            -d "{\"query\": \"$(echo "$query_content" | tr '\n' ' ' | sed 's/"/\\"/g')\", \"variables\": $variables}"
    else
        curl -s -X POST "$DAGSTER_URL" \
            -H "Content-Type: application/json" \
            -d "{\"query\": \"$(echo "$query_content" | tr '\n' ' ' | sed 's/"/\\"/g')\"}"
    fi
}

# Function to extract JSON value
extract_json_value() {
    local json="$1"
    local key="$2"
    echo "$json" | grep -o "\"$key\":\"[^\"]*\"" | cut -d'"' -f4
}

# Function to check if JSON contains error
has_error() {
    local json="$1"
    echo "$json" | grep -q '"errors"'
}

echo -e "${BLUE}🚀 Starting Dagster Pipeline Monitor${NC}"
echo -e "${BLUE}=====================================${NC}"

# Check if GraphQL files exist
if [ ! -f "$TRIGGER_GQL" ] || [ ! -f "$STATUS_GQL" ]; then
    echo -e "${RED}❌ Error: GraphQL files not found in $SCRIPT_DIR${NC}"
    exit 1
fi

# Check if Dagster server is running
echo -e "${YELLOW}🔍 Checking Dagster server connection...${NC}"
if ! curl -s "$DAGSTER_URL" > /dev/null; then
    echo -e "${RED}❌ Error: Cannot connect to Dagster server at $DAGSTER_URL${NC}"
    echo -e "${RED}   Make sure the Dagster server is running on port 3000${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Dagster server is running${NC}"

# Trigger the pipeline
echo -e "${YELLOW}🎯 Triggering lana_to_dw_job...${NC}"
trigger_response=$(make_graphql_request "$TRIGGER_GQL")

if has_error "$trigger_response"; then
    echo -e "${RED}❌ Error triggering pipeline:${NC}"
    echo "$trigger_response" | jq '.errors' 2>/dev/null || echo "$trigger_response"
    exit 1
fi

# Extract run ID from trigger response
run_id=$(extract_json_value "$trigger_response" "runId")
if [ -z "$run_id" ]; then
    echo -e "${RED}❌ Error: Could not extract run ID from trigger response${NC}"
    echo "$trigger_response"
    exit 1
fi

echo -e "${GREEN}✅ Pipeline triggered successfully!${NC}"
echo -e "${BLUE}📋 Run ID: $run_id${NC}"

# Monitor the pipeline
echo -e "${YELLOW}👀 Monitoring pipeline execution...${NC}"
echo -e "${BLUE}=====================================${NC}"

max_attempts=60  # 5 minutes with 5-second intervals
attempt=0
last_status=""

while [ $attempt -lt $max_attempts ]; do
    attempt=$((attempt + 1))
    
    # Get current status
    status_response=$(make_graphql_request "$STATUS_GQL")
    
    if has_error "$status_response"; then
        echo -e "${RED}❌ Error getting status:${NC}"
        echo "$status_response" | jq '.errors' 2>/dev/null || echo "$status_response"
        exit 1
    fi
    
    # Extract status from response
    current_status=$(extract_json_value "$status_response" "status")
    
    if [ -z "$current_status" ]; then
        echo -e "${RED}❌ Error: Could not extract status from response${NC}"
        echo "$status_response"
        exit 1
    fi
    
    # Show status change
    if [ "$current_status" != "$last_status" ]; then
        case "$current_status" in
            "QUEUED")
                echo -e "${YELLOW}⏳ Status: QUEUED (waiting to start)${NC}"
                ;;
            "STARTED")
                echo -e "${BLUE}🏃 Status: STARTED (running)${NC}"
                ;;
            "SUCCESS")
                echo -e "${GREEN}🎉 Status: SUCCESS (completed successfully)${NC}"
                echo -e "${GREEN}=====================================${NC}"
                echo -e "${GREEN}🎊 Pipeline completed successfully! 🎊${NC}"
                echo -e "${GREEN}✅ Your $PIPELINE_JOB_NAME has finished successfully${NC}"
                echo -e "${GREEN}📊 Data has been processed and loaded to BigQuery${NC}"
                echo -e "${GREEN}=====================================${NC}"
                exit 0
                ;;
            "FAILURE")
                echo -e "${RED}💥 Status: FAILURE (job failed)${NC}"
                echo -e "${RED}=====================================${NC}"
                echo -e "${RED}❌ Pipeline execution failed! ❌${NC}"
                echo -e "${RED}🔍 Check the Dagster UI for detailed error logs${NC}"
                echo -e "${RED}📋 Run ID: $run_id${NC}"
                echo -e "${RED}=====================================${NC}"
                exit 1
                ;;
            "CANCELED")
                echo -e "${YELLOW}🛑 Status: CANCELED (job was canceled)${NC}"
                echo -e "${YELLOW}=====================================${NC}"
                echo -e "${YELLOW}⚠️  Pipeline execution was canceled${NC}"
                echo -e "${YELLOW}📋 Run ID: $run_id${NC}"
                echo -e "${YELLOW}=====================================${NC}"
                exit 1
                ;;
            *)
                echo -e "${BLUE}📊 Status: $current_status${NC}"
                ;;
        esac
        last_status="$current_status"
    else
        # Show progress dots
        printf "."
    fi
    
    # Wait before next check
    sleep 5
done

# Timeout reached
echo -e "\n${RED}⏰ Timeout reached after $((max_attempts * 5)) seconds${NC}"
echo -e "${RED}❌ Pipeline monitoring timed out${NC}"
echo -e "${YELLOW}📋 Run ID: $run_id${NC}"
echo -e "${YELLOW}🔍 Check the Dagster UI for current status${NC}"
exit 1
