import { gql } from "@apollo/client"

gql`
  query ChartOfAccountsAccountSet($accountSetId: UUID!, $first: Int!, $after: String) {
    accountSet(accountSetId: $accountSetId) {
      id
      name
      subAccounts(first: $first, after: $after) {
        edges {
          cursor
          node {
            __typename
            ... on AccountDetails {
              __typename
              id
              name
            }
            ... on AccountSetDetails {
              __typename
              id
              name
              hasSubAccounts
            }
          }
        }
      }
    }
  }
`
