import { gql } from "@apollo/client"

gql`
  query ChartOfAccountAccountSet($id: UUID!) {
    chartOfAccountsAccountSet(accountSetId: $id) {
      id
      name
      subAccounts(first: 10) {
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
`
