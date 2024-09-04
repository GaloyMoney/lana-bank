"use client"

import React from "react"
import { unstable_noStore as noStore } from "next/cache"

import { ApolloNextAppProvider } from "@apollo/experimental-nextjs-app-support"

import { makeClient } from "./client"

const ApolloClient: React.FC<React.PropsWithChildren> = ({ children }) => {
  noStore()

  return <ApolloNextAppProvider makeClient={makeClient}>{children}</ApolloNextAppProvider>
}

export { ApolloClient }
