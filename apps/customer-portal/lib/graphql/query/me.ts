import { gql } from "@apollo/client"

import { MeDocument, MeQuery, MeQueryVariables } from "../generated"

import { executeQuery } from "."

gql`
  query Me {
    me {
      customerId
      email
      applicantId
      status
      level
      balance {
        checking {
          settled
          pending
        }
      }
    }
  }
`

export const meQuery = async () => {
  return executeQuery<MeQuery, MeQueryVariables>({
    document: MeDocument,
    variables: {},
  })
}
