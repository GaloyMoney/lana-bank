import { gql } from "@apollo/client"

gql`
  query ChartOfAccountCategoryAccountSet($id: UUID!) {
    chartOfAccountsCategoryAccountSet(accountSetId: $id) {
      id
      name
      hasSubAccounts
      subAccounts {
        ... on AccountDetails {
          id
          name
        }
        ... on AccountSetDetails {
          id
          name
        }
      }
    }
  }
`
