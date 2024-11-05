import { Toast } from "@/components/toast"

import { HelveticaNeueFont, RobotoMono } from "@/lib/ui/fonts"
import "@/lib/ui/globals.css"

import { getServerSession } from "next-auth"

import { Metadata } from "next/types"
import { redirect } from "next/navigation"

import { headers } from "next/headers"

import { authOptions } from "./api/auth/[...nextauth]/options"
import { AuthSessionProvider } from "./session-provider"

export const metadata: Metadata = {
  title: "Lana Bank | Admin Panel",
  icons: [
    {
      rel: "icon",
      url: "/favicon.ico",
    },
  ],
}

const PUBLIC_PAGES = ["/auth/login", "/auth/error", "/auth/verify"]

const RootLayout = async ({
  children,
}: Readonly<{
  children: React.ReactNode
}>) => {
  const headerList = await headers()
  const currentPath = headerList.get("x-current-path") || "/"

  const session = await getServerSession(authOptions)
  if (!session && !PUBLIC_PAGES.includes(currentPath)) redirect("/auth/login")
  if (session && PUBLIC_PAGES.includes(currentPath)) redirect("/")

  return (
    <html lang="en">
      <AuthSessionProvider session={session}>
        <body
          className={`${HelveticaNeueFont.variable} ${RobotoMono.variable} antialiased w-screen h-screen select-none`}
        >
          <Toast />
          {children}
        </body>
      </AuthSessionProvider>
    </html>
  )
}

export default RootLayout
