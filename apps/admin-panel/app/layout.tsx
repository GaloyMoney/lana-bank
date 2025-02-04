"use client"

import { useMemo } from "react"
import { Inter_Tight } from "next/font/google"

import { ApolloProvider } from "@apollo/client"

import { BreadcrumbProvider } from "./breadcrumb-provider"
import { Authenticated } from "./auth/session"

import { makeClient } from "@/lib/apollo-client/client"
import { Toast } from "@/components/toast"
import { env } from "@/env"

// eslint-disable-next-line import/no-unassigned-import
import "./globals.css"

const inter = Inter_Tight({
  subsets: ["latin"],
  variable: "--font-inter",
})

const RootLayout: React.FC<React.PropsWithChildren> = ({ children }) => {
  const appVersion = env.NEXT_PUBLIC_APP_VERSION
  const client = useMemo(() => {
    return makeClient({
      coreAdminGqlUrl: appVersion.endsWith("dev") ? "/admin/graphql" : "/graphql",
    })
  }, [appVersion])

  return (
    <html lang="en">
      <body className={`${inter.className} antialiased bg-background`}>
        <BreadcrumbProvider>
          <ApolloProvider client={client}>
            <Toast />
            <Authenticated>{children}</Authenticated>
          </ApolloProvider>
        </BreadcrumbProvider>
      </body>
    </html>
  )
}

export default RootLayout
