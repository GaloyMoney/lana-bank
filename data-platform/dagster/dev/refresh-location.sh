#!/bin/bash

# Dagster Code Location Refresh Script
# Reloads the "Lana Pipelines" code location to pick up code changes

set -e

# Configuration
DAGSTER_URL="http://localhost:3000/graphql"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REFRESH_GQL="$SCRIPT_DIR/dagster-refresh-location.gql"
DOCKER_COMPOSE_DIR="$(dirname "$SCRIPT_DIR")"
CONTAINER_NAME="lana_pipelines"

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
    
    if [ -n "$variables" ]; then
        curl -s -X POST "$DAGSTER_URL" \
            -H "Content-Type: application/json" \
            -d "{\"query\": \"$(cat "$query_file" | tr '\n' ' ' | sed 's/"/\\"/g')\", \"variables\": $variables}"
    else
        curl -s -X POST "$DAGSTER_URL" \
            -H "Content-Type: application/json" \
            -d "{\"query\": \"$(cat "$query_file" | tr '\n' ' ' | sed 's/"/\\"/g')\"}"
    fi
}

# Function to check if JSON contains error
has_error() {
    local json="$1"
    echo "$json" | grep -q '"errors"'
}

# Function to extract JSON value
extract_json_value() {
    local json="$1"
    local key="$2"
    echo "$json" | grep -o "\"$key\":\"[^\"]*\"" | cut -d'"' -f4
}

echo -e "${BLUE}🔄 Starting Dagster Code Location Refresh${NC}"
echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}This will restart the code location container and reload the code${NC}"

# Check if GraphQL file exists
if [ ! -f "$REFRESH_GQL" ]; then
    echo -e "${RED}❌ Error: GraphQL file not found: $REFRESH_GQL${NC}"
    exit 1
fi

# Check if Docker is available
echo -e "${YELLOW}🐳 Checking Docker availability...${NC}"
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Error: Docker is not installed or not in PATH${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Docker is available${NC}"

# Check if the container exists
echo -e "${YELLOW}🔍 Checking if $CONTAINER_NAME container exists...${NC}"
if ! docker ps -a --format "table {{.Names}}" | grep -q "^$CONTAINER_NAME$"; then
    echo -e "${RED}❌ Error: Container '$CONTAINER_NAME' not found${NC}"
    echo -e "${RED}   Make sure the Dagster docker-compose is running${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Container '$CONTAINER_NAME' found${NC}"

# Stop and remove the code location container
echo -e "${YELLOW}🛑 Stopping $CONTAINER_NAME container...${NC}"
docker stop "$CONTAINER_NAME" > /dev/null 2>&1 || true

echo -e "${YELLOW}🗑️  Removing $CONTAINER_NAME container...${NC}"
docker rm "$CONTAINER_NAME" > /dev/null 2>&1 || true

# Rebuild and start the container
echo -e "${YELLOW}🔨 Rebuilding $CONTAINER_NAME container with latest code...${NC}"
cd "$DOCKER_COMPOSE_DIR"
if ! docker compose up -d --build "$CONTAINER_NAME" > /dev/null 2>&1; then
    echo -e "${RED}❌ Error: Failed to rebuild and start container '$CONTAINER_NAME'${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Container '$CONTAINER_NAME' rebuilt and started successfully${NC}"

# Wait for container to be ready
echo -e "${YELLOW}⏳ Waiting for container to be ready...${NC}"
sleep 15

# Check if Dagster server is running
echo -e "${YELLOW}🔍 Checking Dagster server connection...${NC}"
max_attempts=12  # 2 minutes with 10-second intervals
attempt=0
while [ $attempt -lt $max_attempts ]; do
    attempt=$((attempt + 1))
    if curl -s "$DAGSTER_URL" > /dev/null; then
        echo -e "${GREEN}✅ Dagster server is running${NC}"
        break
    fi
    if [ $attempt -eq $max_attempts ]; then
        echo -e "${RED}❌ Error: Cannot connect to Dagster server at $DAGSTER_URL${NC}"
        echo -e "${RED}   Server may still be starting up. Try again in a few minutes.${NC}"
        exit 1
    fi
    echo -e "${YELLOW}⏳ Waiting for Dagster server... (attempt $attempt/$max_attempts)${NC}"
    sleep 10
done

# Refresh the code location
echo -e "${YELLOW}🔄 Refreshing 'Lana Pipelines' code location...${NC}"
refresh_response=$(make_graphql_request "$REFRESH_GQL")

if has_error "$refresh_response"; then
    echo -e "${RED}❌ Error refreshing code location:${NC}"
    echo "$refresh_response" | jq '.errors' 2>/dev/null || echo "$refresh_response"
    exit 1
fi

# Check the response
if echo "$refresh_response" | grep -q "WorkspaceLocationEntry"; then
    echo -e "${GREEN}✅ Code location refreshed successfully!${NC}"
    echo -e "${GREEN}📦 Location: Lana Pipelines${NC}"
    echo -e "${GREEN}🐳 Container: $CONTAINER_NAME rebuilt${NC}"
    echo -e "${GREEN}=========================================${NC}"
    echo -e "${GREEN}🎉 Your code changes have been loaded! 🎉${NC}"
    echo -e "${GREEN}✨ The 'Lana Pipelines' container has been rebuilt${NC}"
    echo -e "${GREEN}🔄 Code location has been refreshed${NC}"
    echo -e "${GREEN}🚀 You can now run your updated pipeline${NC}"
    echo -e "${GREEN}=========================================${NC}"
elif echo "$refresh_response" | grep -q "PythonError"; then
    echo -e "${RED}❌ Python error during refresh:${NC}"
    echo "$refresh_response" | jq '.data.reloadRepositoryLocation.message' 2>/dev/null || echo "$refresh_response"
    exit 1
elif echo "$refresh_response" | grep -q "RepositoryLocationNotFound"; then
    echo -e "${RED}❌ Error: Repository location 'Lana Pipelines' not found${NC}"
    echo -e "${RED}   Make sure the location name is correct${NC}"
    exit 1
else
    echo -e "${YELLOW}⚠️  Unexpected response:${NC}"
    echo "$refresh_response"
fi
