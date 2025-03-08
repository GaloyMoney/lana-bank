import { DocumentNode, TypedDocumentNode } from "@apollo/client/core"
import { OperationVariables } from "@apollo/client"

import { getClient } from "@/lib/apollo-client/rsc"

type MutationDocument<TData, TVariables> =
  | DocumentNode
  | TypedDocumentNode<TData, TVariables>

export async function executeMutation<
  TData,
  TVariables extends OperationVariables = OperationVariables,
>(options: {
  document: MutationDocument<TData, TVariables>
  variables: TVariables
}): Promise<TData | Error> {
  try {
    const response = await getClient().mutate<TData, TVariables>({
      mutation: options.document,
      variables: options.variables,
    })

    if (response.errors && response.errors.length > 0) {
      throw new Error(response.errors[0].message)
    }

    if (!response.data) {
      throw new Error("No data found")
    }

    return response.data
  } catch (error) {
    if (error instanceof Error) {
      return error
    }
    return new Error("An unknown error occurred")
  }
}
