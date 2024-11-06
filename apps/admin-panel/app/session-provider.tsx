"use client"

import { Session } from "next-auth"
import { SessionProvider } from "next-auth/react"

export function AuthSessionProvider({
  children,
  session,
}: Readonly<{
  children: React.ReactNode
  session: Session
}>) {
  const isDevelopment = process.env.NODE_ENV === "development"
  const sessionProviderProps = {
    session,
    ...(isDevelopment && { basePath: "/admin-panel/api/auth" }),
  }

  return <SessionProvider {...sessionProviderProps}>{children}</SessionProvider>
}
