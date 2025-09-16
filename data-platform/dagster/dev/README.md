# Dagster Pipeline Development Tools

This folder contains development tools for working with the Dagster pipeline.

## Files

- `dagster-lana-pipeline-trigger.gql` - GraphQL mutation to trigger the lana_pipeline_job
- `dagster-lana-pipeline-status.gql` - GraphQL query to check the status of the most recent run
- `dagster-refresh-location.gql` - GraphQL mutation to refresh the code location
- `monitor-pipeline.sh` - Bash script to trigger and monitor the pipeline execution
- `refresh-location.sh` - Bash script to refresh the Dagster code location

## Usage

### Monitor Pipeline Script

The `monitor-pipeline.sh` script provides a complete pipeline monitoring experience:

```bash
./monitor-pipeline.sh
```

**What it does:**
1. ✅ Checks if Dagster server is running on localhost:3000
2. 🎯 Triggers the `lana_pipeline_job`
3. 👀 Monitors the execution in real-time
4. 🎉 Shows success message when completed
5. ❌ Shows error message if failed

**Features:**
- Color-coded output for better readability
- Real-time status updates (QUEUED → STARTING → STARTED → SUCCESS/FAILURE)
- Automatic timeout after 5 minutes
- Detailed error reporting
- Progress indicators

### Refresh Code Location Script

The `refresh-location.sh` script reloads the Dagster code location to pick up any code changes:

```bash
./refresh-location.sh
```

**What it does:**
1. 🐳 Checks if Docker is available and the container exists
2. 🛑 Stops and removes the `lana_pipelines` container
3. 🔨 Rebuilds the container with your latest code changes
4. ⏳ Waits for the container to be ready and Dagster server to respond
5. 🔄 Triggers a reload of the "Lana Pipelines" code location
6. 🎉 Shows success message when container is rebuilt and code is reloaded
7. ❌ Shows error message if any step fails

**Use this when:**
- You've made changes to your pipeline code
- You've updated dependencies or configuration
- You want to ensure Dagster is using the latest code
- You need to restart the code location container

### Manual GraphQL Usage

You can also use the GraphQL files directly:

```bash
# Trigger the job
curl -X POST http://localhost:3000/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "'"$(cat dagster-lana-pipeline-trigger.gql | tr '\n' ' ' | sed 's/"/\\"/g')"'"}'

# Check status
curl -X POST http://localhost:3000/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "'"$(cat dagster-lana-pipeline-status.gql | tr '\n' ' ' | sed 's/"/\\"/g')"'"}'

# Refresh code location
curl -X POST http://localhost:3000/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "'"$(cat dagster-refresh-location.gql | tr '\n' ' ' | sed 's/"/\\"/g')"'"}'
```

## Prerequisites

- Dagster server must be running on `http://localhost:3000`
- The `lana_pipeline_job` must be available in the Dagster workspace
- `curl` and `jq` (optional, for JSON parsing) must be installed
- `docker` and `docker compose` must be installed and accessible for the refresh script
- The `lana_pipelines` container must be running
