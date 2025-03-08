import { gql } from "@apollo/client"

import {
  SumsubPermalinkCreateDocument,
  SumsubPermalinkCreateMutation,
  SumsubPermalinkCreateMutationVariables,
} from "../generated"

import { executeMutation } from "."

gql`
  mutation sumsubPermalinkCreate {
    sumsubPermalinkCreate
  }
`

export const sumsubPermalinkCreate = async () => {
  return executeMutation<
    SumsubPermalinkCreateMutation,
    SumsubPermalinkCreateMutationVariables
  >({
    document: SumsubPermalinkCreateDocument,
    variables: {},
  })
}
