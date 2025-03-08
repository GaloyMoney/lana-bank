# Lana Bank MCP (Model Context Protocol) Server

This is a Model Context Protocol (MCP) server for Lana Bank that allows Claude to interact with the bank's admin functionality.

## Setup with Claude for Desktop

1. Make sure you have Claude for Desktop installed and updated to the latest version.

2. Open your Claude for Desktop configuration file at `~/Library/Application Support/Claude/claude_desktop_config.json`.

3. Add the Lana MCP server configuration:

```json
{
  "mcpServers": {
    "lana": {
      "command": "node",
      "args": [
        "/ABSOLUTE/PATH/TO/lana-bank/apps/mcp/build/index.js",
        "your_admin_secret_here"
      ]
    }
  }
}
```

Replace `/ABSOLUTE/PATH/TO/` with the absolute path to your lana-bank repository, and `your_admin_secret_here` with your actual admin secret.

## Authentication

The server uses an admin secret for authentication, which is passed as the first command-line argument. This secret is used to authenticate GraphQL API requests to the Lana Bank admin API.

## Available Tools

This MCP server provides the following tools:

- Credit Facilities Tool - List credit facilities
- Credit Facility Details Tool - Get details for a specific credit facility
- Customer Credit Facilities Tool - List credit facilities for a specific customer
- Customer Details Tool - Get details for a specific customer

## Development

### Building the Server

```bash
cd /path/to/lana-bank
npm run build
```

### Testing Locally

You can test the server directly using:

```bash
node apps/mcp/build/index.js your_admin_secret_here
```

If the secret is not provided, the server will exit with an error message.

### Debugging

If you encounter issues with the MCP server in Claude for Desktop, check the logs at:

```bash
tail -n 20 -f ~/Library/Logs/Claude/mcp*.log
```
