"use client"

import { useEffect, useState } from "react"
import { useRouter } from "next/navigation"

import { ApolloProvider } from "@apollo/client"

import { AppLayout } from "../app-layout"
import { BreadcrumbProvider } from "../breadcrumb-provider"
import { useAppLoading } from "../app-loading"

import { initKeycloak, logout } from "./keycloak"

import { Toast } from "@/components/toast"
import { makeClient } from "@/lib/apollo-client/client"

type Props = {
  children: React.ReactNode
}

export const Authenticated: React.FC<Props> = ({ children }) => {
  const [initialized, setInitialized] = useState(false)
  const [authenticated, setAuthenticated] = useState(false)
  const { stopAppLoadingAnimation } = useAppLoading()

  useEffect(() => {
    if (typeof window !== "undefined") {
      initKeycloak()
        .then((auth) => {
          setAuthenticated(auth)
          setInitialized(true)
          stopAppLoadingAnimation()
        })
        .catch((err) => console.error("Failed to initialize Keycloak", err))
    }
  }, [])

  if (!initialized || !authenticated) {
    return null
  }

  const client = makeClient({ coreAdminGqlUrl: "/graphql" })
  return (
    <BreadcrumbProvider>
      <ApolloProvider client={client}>
        <Toast />
        <AppLayout>{children}</AppLayout>
      </ApolloProvider>
    </BreadcrumbProvider>
  )
}

export const useLogout = () => {
  const router = useRouter()
  return {
    logout: async () => {
      await logout()
      router.push("/")
    },
  }
}
