const express = require('express');
const { Pool } = require('pg');
const path = require('path');

const app = express();
const port = 3030; // Use a different port

// --- Database Connection ---
const connectionString = process.env.PG_CON;
if (!connectionString) {
    console.error('Error: PG_CON environment variable is not set.');
    process.exit(1);
}
const pool = new Pool({ connectionString });

(async () => {
    let client;
    try {
        client = await pool.connect();
        console.log(`Cala DAG Server: Database connection test successful on port ${port}!`);
    } catch (err) {
        console.error('Cala DAG Server: Database connection test failed:', err);
    } finally {
        if (client) client.release();
    }
})();

// --- Middleware ---
app.use(express.json());

// --- Route for Root Path ---
app.get('/', (req, res) => {
    console.log('Serving cala-dag-index.html for root path');
    res.sendFile(path.join(__dirname, 'cala-dag-index.html'));
});

// --- API Endpoint ---
app.get('/api/cala-graph-data', async (req, res) => {
    console.log('Received request for /api/cala-graph-data');
    let client;
    try {
        client = await pool.connect();

        // Fetch Accounts (assuming 'cala_accounts' table exists with name/code)
        // Adjust column names if necessary based on your actual schema
        const accountsQuery = `
            SELECT
                id::text,
                COALESCE(name, 'Unnamed Account') as name,
                COALESCE(code, '') as code,
                external_id::text,
                'account' as type
            FROM cala_accounts`;
        const accountsResult = await client.query(accountsQuery);
        const accountNodes = accountsResult.rows.map(r => ({
            id: r.id,
            label: `${r.name} ${r.code ? '(' + r.code + ')' : ''}`.trim(),
            type: r.type,
            external_id: r.external_id
        }));

        // Fetch Account Sets (assuming 'cala_account_sets' table exists with name)
        // Adjust column names if necessary
        const setsQuery = `
            SELECT
                id::text,
                COALESCE(name, 'Unnamed Set') as name,
                external_id::text,
                'account_set' as type
            FROM cala_account_sets`;
        const setsResult = await client.query(setsQuery);
        const setNodes = setsResult.rows.map(r => ({
            id: r.id,
            label: r.name,
            type: r.type,
            external_id: r.external_id
        }));

        // Fetch Account -> Set Memberships
        const memberAccQuery = `
            SELECT
                account_set_id::text as source,
                member_account_id::text as target
            FROM cala_account_set_member_accounts
            WHERE transitive = false`; // Only direct memberships for DAG
        const memberAccResult = await client.query(memberAccQuery);

        // Fetch Set -> Set Memberships
        const memberSetQuery = `
            SELECT
                account_set_id::text as source,
                member_account_set_id::text as target
            FROM cala_account_set_member_account_sets`; // Assuming no transitive flag needed here, or adjust if present
        const memberSetResult = await client.query(memberSetQuery);

        const nodes = [...accountNodes, ...setNodes];
        const edges = [...memberAccResult.rows, ...memberSetResult.rows];

        // Basic validation: Ensure all edge sources/targets exist as nodes
        const nodeIds = new Set(nodes.map(n => n.id));
        const validEdges = edges.filter(e => nodeIds.has(e.source) && nodeIds.has(e.target));
        const invalidEdgeCount = edges.length - validEdges.length;
        if (invalidEdgeCount > 0) {
            console.warn(`Filtered out ${invalidEdgeCount} edges with missing nodes.`);
        }


        console.log(`Successfully fetched ${nodes.length} nodes and ${validEdges.length} edges from Cala tables.`);
        res.json({ nodes, edges: validEdges });

    } catch (err) {
        console.error('Error executing Cala graph data query:', err);
        res.status(500).json({ error: 'Failed to fetch Cala graph data' });
    } finally {
        if (client) client.release();
    }
});

// --- Serve Static Files (for JS, CSS, etc.) ---
// Serve files directly from the 'dev/dag' directory
app.use(express.static(path.join(__dirname, '.')));

// --- Fallback for SPA (Optional, uncomment if needed) ---
/*
app.get('*', (req, res) => {
  res.sendFile(path.join(__dirname, 'cala-dag-index.html')); // Serve the new HTML file
});
*/

// --- Start Server ---
app.listen(port, () => {
    console.log(`Cala DAG server running at http://localhost:${port}`);
    console.log(`Serving static files from: ${path.join(__dirname, '.')}`);
    console.log('Connecting to database using PG_CON...');
});

// --- Graceful Shutdown ---
process.on('SIGINT', async () => {
    console.log('\nShutting down Cala DAG server...');
    await pool.end();
    console.log('Database pool closed.');
    process.exit(0);
}); 