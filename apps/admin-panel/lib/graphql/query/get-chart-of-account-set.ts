import { gql } from "@apollo/client"

gql`
  query ChartOfAccountCategoryAccountSet($id: UUID!) {
    chartOfAccountsCategoryAccountSet(accountSetId: $id) {
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
