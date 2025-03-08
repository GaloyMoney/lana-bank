import { createRequire } from "module";
const require = createRequire(import.meta.url);
const { ApolloClient, InMemoryCache, HttpLink } = require("@apollo/client");

// Create and export the Apollo Client instance directly
export const graphqlClient = new ApolloClient({
  cache: new InMemoryCache(),
  link: new HttpLink({
    uri: "http://localhost:4455/admin/graphql",
  }),
});
