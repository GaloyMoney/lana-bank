import { gql } from "@apollo/client"

gql`
  query Customers($first: Int!, $after: String) {
    customers(first: $first, after: $after) {
      nodes {
        customerId
        email
        balance {
          checking {
            settled
            pending
          }
        }
      }
      pageInfo {
        endCursor
        startCursor
        hasNextPage
        hasPreviousPage
      }
    }
  }
`
