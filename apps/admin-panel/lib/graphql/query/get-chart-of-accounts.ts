import { gql } from "@apollo/client"

gql`
  query GetChartOfAccounts {
    chartOfAccounts {
      name
      categories {
        id
        name
        accounts {
          __typename
          ... on AccountDetails {
            id
            name
          }
          ... on AccountSetDetails {
            id
            name
            hasSubAccounts
          }
        }
      }
    }
  }
`
