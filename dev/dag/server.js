const express = require('express');
const { Pool } = require('pg');
const path = require('path');

const app = express();
const port = 3000; // Or another port if 8080 is used elsewhere

// Check for PG_CON environment variable
const connectionString = process.env.PG_CON;
if (!connectionString) {
    console.error('Error: PG_CON environment variable is not set.');
    console.error('Please set PG_CON to your PostgreSQL connection string.');
    process.exit(1); // Exit if the connection string is missing
}

// Create a PostgreSQL connection pool
const pool = new Pool({
    connectionString: connectionString,
});

// Test the database connection immediately
(async () => {
    let client;
    try {
        client = await pool.connect();
        console.log('Database connection test successful!');
    } catch (err) {
        console.error('Database connection test failed:', err);
        // Optionally exit if connection fails on startup
        // process.exit(1);
    } finally {
        if (client) {
            client.release(); // Release the client back to the pool
        }
    }
})();

// Middleware to parse JSON request bodies
app.use(express.json());

// API endpoint to fetch graph data
app.get('/api/graph-data', async (req, res) => {
    console.log('Received request for /api/graph-data');
    try {
        // Query the database - adjust schema/table name if needed
        const query = `SELECT event, event_type FROM core_chart_events`;


        // use external id of the cala account set
        // cala account set has lana code

        // external id from cala account set : <chart_id>.<code>



        // mapping also present in chart.
        // chart entity
        // node added event has cala.id?

        const result = await pool.query(query);
        console.log(`Successfully fetched ${result.rows.length} rows from the database.`);
        res.json(result.rows);
    } catch (err) {
        console.error('Error executing database query:', err);
        res.status(500).json({ error: 'Failed to fetch graph data from database' });
    }
});

// Serve static files (index.html, dag-graph.js, node_modules, etc.)
// This should come AFTER specific API routes but BEFORE the catch-all
app.use(express.static(path.join(__dirname, '.')));

// Fallback: Serve index.html for any route not matched by API or static files
// This MUST be the last route handler
/*
app.get('*', (req, res) => {
  console.log(`Serving index.html for unmatched route: ${req.path}`);
  res.sendFile(path.join(__dirname, 'index.html'));
});
*/

// Start the server
app.listen(port, () => {
    console.log(`DAG server running at http://localhost:${port}`);
    console.log(`Serving static files from: ${path.join(__dirname, '.')}`);
    console.log('Connecting to database using PG_CON...');
});

// Optional: Graceful shutdown
process.on('SIGINT', async () => {
    console.log('\nShutting down server...');
    await pool.end();
    console.log('Database pool closed.');
    process.exit(0);
}); 