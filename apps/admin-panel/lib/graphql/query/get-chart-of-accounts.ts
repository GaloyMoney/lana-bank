import { gql } from "@apollo/client"

gql`
  query GetChartOfAccounts {
    chartOfAccounts {
      name
      categories {
        id
        name
        accounts {
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
