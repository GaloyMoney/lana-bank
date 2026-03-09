import { ApolloError } from "@apollo/client"

export const isAuthorizationError = (error: ApolloError | undefined): boolean => {
  if (!error) return false

  return error.graphQLErrors.some(
    (e) =>
      e.extensions?.code === "FORBIDDEN" ||
      e.message.includes("AuthorizationError") ||
      e.message.includes("PermissionDenied"),
  )
}
