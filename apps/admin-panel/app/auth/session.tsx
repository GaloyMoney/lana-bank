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
  const [authError, setAuthError] = useState<Error | null>(null)
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
            setAuthError(err instanceof Error ? err : new Error("Authentication failed"))
            setInitialized(true)
            stopAppLoadingAnimation()
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

  if (!initialized) {
    return null
  }

  if (authError) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background px-6">
        <div className="max-w-lg space-y-3 rounded-lg border bg-card p-6 text-center shadow-sm">
          <h1 className="text-xl font-semibold">Authentication failed</h1>
          <p className="text-sm text-muted-foreground">
            The admin panel could not initialize the local Keycloak login flow.
          </p>
          <p className="text-sm text-muted-foreground">
            If you are using Brave, turn off Shields for <code>admin.localhost</code>,
            reload the page, and try again.
          </p>
        </div>
      </div>
    )
  }

  if (!authenticated || !client) {
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
