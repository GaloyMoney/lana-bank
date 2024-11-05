// eslint-disable-next-line import/no-unassigned-import
import "@/lib/ui/globals.css"
import type { Metadata } from "next"
import { redirect } from "next/navigation"

import { headers } from "next/headers"
import { getServerSession } from "next-auth"

import { AuthSessionProvider } from "./session-provider"
import { authOptions } from "./api/auth/[...nextauth]/options"

import { HelveticaNeueFont, RobotoMono } from "@/lib/ui/fonts"
import { Toast } from "@/components/toast"
import GQLClient from "@/lib/graphql/client"

export const metadata: Metadata = {
  title: "Lana Bank | Admin Panel",
  icons: [
    {
      rel: "icon",
      url: "/favicon.ico",
      type: "image/x-icon",
    },
  ],
}

const PUBLIC_PAGES = ["/auth/login", "/auth/error", "/auth/verify"]

const RootLayout: React.FC<React.PropsWithChildren> = async ({ children }) => {
  const headerList = headers()
  const currentPath = headerList.get("x-current-path") || "/"

  const session = await getServerSession(authOptions)
  if (!session && !PUBLIC_PAGES.includes(currentPath)) redirect("/auth/login")
  if (session && PUBLIC_PAGES.includes(currentPath)) redirect("/")
  if (session && ["/", "/app"].includes(currentPath)) redirect("/app/dashboard")

  return (
    <html lang="en">
      <body
        className={`${HelveticaNeueFont.variable} ${RobotoMono.variable} antialiased w-screen h-screen select-none`}
      >
        <AuthSessionProvider session={session}>
          <GQLClient>
            <Toast />
            {children}
          </GQLClient>
        </AuthSessionProvider>
      </body>
    </html>
  )
}

export default RootLayout
