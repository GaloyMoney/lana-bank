import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";
import { graphqlClient } from "./lib/client.js";
import { registerAllTools } from "./chat/tools.js";

// Create server instance
const server = new McpServer({
  name: "lana-admin",
  version: "1.0.0",
});

// Register all tools
registerAllTools(server);

async function main() {
  // Validate required admin secret from command-line arguments
  // The secret is expected to be the first argument after the script path (index 2)
  const adminSecret = process.argv[2];

  if (!adminSecret) {
    console.error(
      "Error: ADMIN_SECRET is not provided as a command-line argument."
    );
    console.error(
      "Please add it to your Claude Desktop configuration file as the first argument."
    );
    process.exit(1);
  }

  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("Lana Admin MCP Server running on stdio");
}

main().catch((error) => {
  console.error("Fatal error in main():", error);
  process.exit(1);
});
