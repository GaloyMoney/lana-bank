const { Pool } = require('pg');
const fs = require('fs').promises; // Use promise-based fs

// --- Configuration ---
const LANA_DOT_FILE = 'output/lana.dot';
const CALA_DOT_FILE = 'output/cala.dot';

// --- Database Connection ---
const connectionString = process.env.PG_CON;
if (!connectionString) {
    console.error('Error: PG_CON environment variable is not set.');
    console.error('Please set PG_CON to your PostgreSQL connection string.');
    process.exit(1);
}

const pool = new Pool({ connectionString });

// --- DOT Generation Functions ---

// Helper to escape strings for DOT labels
function escapeDot(str) {
    if (typeof str !== 'string') {
        return str; // Return non-strings as is
    }
    return str.replace(/"/g, '\\"').replace(/\n/g, '\\n');
}


async function generateLanaDot(client) {
    console.log('Generating Lana DOT file...');
    try {
        // Adjust query if needed based on actual schema and desired graph structure
        const query = `SELECT event, event_type FROM core_chart_events ORDER BY id;`; // Assuming an 'id' or similar for consistent ordering
        const result = await client.query(query);
        console.log(`Fetched ${result.rows.length} rows for Lana DAG.`);

        let dotString = 'digraph Lana {\n';
        dotString += '  rankdir=LR;\n'; // Or TB, RL, BT
        dotString += '  node [shape=box, style=filled, fillcolor=lightgrey];\n\n';

        // Create nodes - Needs a clear way to define edges based on the data
        result.rows.forEach((row, index) => {
            // Create a safer, more unique ID if 'event' isn't guaranteed unique or contains problematic chars
            const nodeId = `event_${index}_${escapeDot(String(row.event)).replace(/\W+/g, '_')}`;
            const nodeLabel = `${escapeDot(row.event)}\\n(Type: ${escapeDot(row.event_type)})`;
            dotString += `  "${nodeId}" [label="${nodeLabel}"];\n`;
        });

        // TODO: Define edge logic for Lana graph based on data relationships
        // Example: If rows represent sequential events, add edges between them
        // for (let i = 0; i < result.rows.length - 1; i++) {
        //     const sourceId = `event_${i}_${escapeDot(String(result.rows[i].event)).replace(/\W+/g, '_')}`;
        //     const targetId = `event_${i + 1}_${escapeDot(String(result.rows[i+1].event)).replace(/\W+/g, '_')}`;
        //     dotString += `  "${sourceId}" -> "${targetId}";\n`;
        // }

        dotString += '}\n';

        await fs.writeFile(LANA_DOT_FILE, dotString);
        console.log(`Successfully wrote Lana graph to ${LANA_DOT_FILE}`);

    } catch (err) {
        console.error('Error generating Lana DOT:', err);
        throw err; // Re-throw to handle in main block
    }
}


async function generateCalaDot(client) {
    console.log('Generating Cala DOT file...');
    try {
        // Fetch Accounts
        const accountsQuery = `
            SELECT id::text, COALESCE(name, 'Unnamed Account') as name, COALESCE(code, '') as code, external_id::text, 'account' as type
            FROM cala_accounts`;
        const accountsResult = await client.query(accountsQuery);

        // Fetch Account Sets
        const setsQuery = `
            SELECT id::text, COALESCE(name, 'Unnamed Set') as name, external_id::text, 'account_set' as type
            FROM cala_account_sets`;
        const setsResult = await client.query(setsQuery);

        // Fetch Account -> Set Memberships
        const memberAccQuery = `
            SELECT account_set_id::text as source, member_account_id::text as target
            FROM cala_account_set_member_accounts WHERE transitive = false`;
        const memberAccResult = await client.query(memberAccQuery);

        // Fetch Set -> Set Memberships
        const memberSetQuery = `
            SELECT account_set_id::text as source, member_account_set_id::text as target
            FROM cala_account_set_member_account_sets`;
        const memberSetResult = await client.query(memberSetQuery);

        let dotString = 'digraph Cala {\n';
        dotString += '  rankdir=LR;\n';
        dotString += '  node [shape=record, style=filled];\n\n';

        // Add account nodes
        dotString += '  // Account Nodes\n';
        accountsResult.rows.forEach(r => {
            const labelParts = [escapeDot(r.name)];
            if (r.code) labelParts.push(`Code: ${escapeDot(r.code)}`);
            // labelParts.push(`ID: ${escapeDot(r.id)}`); // Often redundant if node ID is the UUID
            const label = `{${labelParts.join(' | ')}}`;
            dotString += `  "${escapeDot(r.id)}" [label="${label}", fillcolor=lightblue];\n`;
        });
        dotString += '\n';

        // Add set nodes
        dotString += '  // Set Nodes\n';
        setsResult.rows.forEach(r => {
            const labelParts = [escapeDot(r.name)];
            // labelParts.push(`ID: ${escapeDot(r.id)}`); // Often redundant
            const label = `{${labelParts.join(' | ')}}`;
            dotString += `  "${escapeDot(r.id)}" [label="${label}", shape=folder, fillcolor=lightcoral];\n`; // Different shape/color for sets
        });
        dotString += '\n';

        const nodes = [...accountsResult.rows, ...setsResult.rows];
        const nodeIds = new Set(nodes.map(n => n.id));
        const edges = [...memberAccResult.rows, ...memberSetResult.rows];

        // Filter out edges pointing to/from non-existent nodes
        const validEdges = edges.filter(e => nodeIds.has(e.source) && nodeIds.has(e.target));
        const invalidEdgeCount = edges.length - validEdges.length;
        if (invalidEdgeCount > 0) {
            console.warn(`Filtered out ${invalidEdgeCount} Cala edges with missing nodes.`);
        }

        console.log(`Fetched ${nodes.length} nodes and ${validEdges.length} edges for Cala DAG.`);


        // Add edges
        dotString += '  // Edges (Memberships)\n';
        validEdges.forEach(edge => {
            dotString += `  "${escapeDot(edge.source)}" -> "${escapeDot(edge.target)}";\n`;
        });

        dotString += '}\n';

        await fs.writeFile(CALA_DOT_FILE, dotString);
        console.log(`Successfully wrote Cala graph to ${CALA_DOT_FILE}`);

    } catch (err) {
        console.error('Error generating Cala DOT:', err);
        throw err; // Re-throw to handle in main block
    }
}


// --- Main Execution ---
(async () => {
    let client;
    try {
        console.log('Connecting to database...');
        client = await pool.connect();
        console.log('Database connection successful.');

        await generateLanaDot(client);
        await generateCalaDot(client);

        console.log('DOT file generation complete.');

    } catch (err) {
        console.error('An error occurred during DOT generation:', err);
        process.exitCode = 1; // Indicate failure
    } finally {
        if (client) {
            console.log('Releasing database client.');
            client.release();
        }
        console.log('Closing database pool.');
        await pool.end(); // Close the pool
        console.log('Database pool closed.');
    }
})(); 