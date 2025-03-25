import { createRequire } from "module";
const require = createRequire(import.meta.url);
const apolloClient = require("@apollo/client");
const { ApolloClient, InMemoryCache, HttpLink } = apolloClient;

// Get admin secret from command line arguments
// The secret is expected to be the first argument after the script path (index 2)
const adminSecret = process.argv[2] || "";

// Create and export the Apollo Client instance directly
export const graphqlClient = new ApolloClient({
  cache: new InMemoryCache(),
  link: new HttpLink({
    uri: "http://localhost:4455/admin/graphql",
    headers: {
      Authorization: `Bearer ${adminSecret}`,
    },
  }),
});
