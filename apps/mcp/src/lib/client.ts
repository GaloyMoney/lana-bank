import { createRequire } from "module";
const require = createRequire(import.meta.url);
const apolloClient = require("@apollo/client");
const { ApolloClient, InMemoryCache, HttpLink } = apolloClient;

// Create and export the Apollo Client instance directly
export const graphqlClient = new ApolloClient({
  cache: new InMemoryCache(),
  link: new HttpLink({
    uri: "http://localhost:4455/admin/graphql",
  }),
});
