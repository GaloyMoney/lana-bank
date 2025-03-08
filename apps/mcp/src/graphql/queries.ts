import { createRequire } from "module";
const require = createRequire(import.meta.url);
const { gql } = require("@apollo/client");

export const EXAMPLE_QUERY = gql`
  query ExampleQuery {
    __typename
  }
`;
