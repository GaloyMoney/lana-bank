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
  return (
    <SessionProvider basePath="/admin-panel/api/auth" session={session}>
      {children}
    </SessionProvider>
  )
}
