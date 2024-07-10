import { gql } from "@apollo/client"

import { GetChartOfAccountsDocument, GetChartOfAccountsQuery } from "../generated"

import { executeQuery } from "."

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

export const chartOfAccountsQuery = async () => {
  return executeQuery<GetChartOfAccountsQuery>({
    document: GetChartOfAccountsDocument,
    variables: {},
  })
}
