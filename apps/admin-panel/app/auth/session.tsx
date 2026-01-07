"use client"

import { useEffect, useState, useMemo } from "react"
import { useRouter } from "next/navigation"

import { ApolloProvider } from "@apollo/client"

import { AppLayout } from "../app-layout"
import { BreadcrumbProvider } from "../breadcrumb-provider"
import { useAppLoading } from "../app-loading"

import { initKeycloak, logout } from "./keycloak"
import IdleSessionGuard from "./idle-session-guard"

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
    let isMounted = true

    if (typeof window !== "undefined" && !initialized) {
      initKeycloak()
        .then((auth) => {
          if (isMounted) {
            setAuthenticated(auth)
            setInitialized(true)
            stopAppLoadingAnimation()
          }
        })
        .catch((err) => {
          if (isMounted) {
            console.error("Failed to initialize Keycloak", err)
            setInitialized(true)
          }
        })
    }

    return () => {
      isMounted = false
    }
  }, [initialized, stopAppLoadingAnimation])

  const client = useMemo(() => {
    if (initialized && authenticated) {
      return makeClient({
        coreAdminGqlUrl: "/graphql",
        coreAdminSseUrl: "/graphql/stream",
      })
    }
    return null
  }, [initialized, authenticated])

  if (!initialized || !authenticated || !client) {
    return null
  }

  return (
    <BreadcrumbProvider>
      <ApolloProvider client={client}>
        <Toast />
        <AppLayout>
          <IdleSessionGuard />
          {children}
        </AppLayout>
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
