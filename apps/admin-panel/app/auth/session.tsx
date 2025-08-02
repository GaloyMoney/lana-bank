"use client"

import { useEffect, useState } from "react"
import { usePathname, useRouter } from "next/navigation"
import dynamic from "next/dynamic"

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

const AuthGuard: React.FC<Props> = ({ children }) => {
  const router = useRouter()
  const pathName = usePathname()
  const { stopAppLoadingAnimation } = useAppLoading()
  const [authenticated, setAuthenticated] = useState<boolean | null>(null)

  useEffect(() => {
    const checkAuth = async () => {
      try {
        const isAuthenticated = await initKeycloak()
        if (isAuthenticated) {
          setAuthenticated(true)
          if (pathName === "/" || pathName.startsWith("/auth")) {
            router.push("/dashboard")
          }
        } else {
          setAuthenticated(false)
          if (!pathName.startsWith("/auth")) {
            router.push("/auth/login")
          }
        }
      } catch (error) {
        setAuthenticated(false)
        if (!pathName.startsWith("/auth")) router.push("/auth/login")
      } finally {
        stopAppLoadingAnimation()
      }
    }

    checkAuth()
  }, [pathName, router, stopAppLoadingAnimation])

  const client = makeClient({ coreAdminGqlUrl: "/graphql" })

  return (
    <BreadcrumbProvider>
      <ApolloProvider client={client}>
        <Toast />
        {authenticated ? (
          <AppLayout>{children}</AppLayout>
        ) : (
          <main className="h-screen w-full flex flex-col">{children}</main>
        )}
      </ApolloProvider>
    </BreadcrumbProvider>
  )
}

export const Authenticated = dynamic(() => Promise.resolve(AuthGuard), {
  ssr: false,
})

export const useLogout = () => {
  const router = useRouter()

  return {
    logout: async () => {
      await logout()
      router.push("/")
    },
  }
}
