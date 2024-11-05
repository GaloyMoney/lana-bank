export async function register() {
  if (process.env.NEXT_RUNTIME === "nodejs") {
    const { pool } = await import("@/lib/auth/db")
    const createTableQuery = `
    CREATE TABLE IF NOT EXISTS verification_token (
      identifier TEXT NOT NULL,
      expires TIMESTAMPTZ NOT NULL,
      token TEXT NOT NULL,
      PRIMARY KEY (identifier, token)
    );
  `
    try {
      const client = await pool.connect()
      try {
        await client.query(createTableQuery)
        console.log("Table created successfully or already exists")
      } finally {
        client.release()
      }
    } catch (err) {
      console.error("Error creating table", err)
    } finally {
      pool.end()
    }
  }
}
