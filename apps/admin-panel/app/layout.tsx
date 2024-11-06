import type { Metadata } from "next"
import { Inter_Tight } from "next/font/google"

import { getServerSession } from "next-auth"
import { redirect } from "next/navigation"
import { headers } from "next/headers"

import { authOptions } from "./api/auth/[...nextauth]/options"
import { AuthSessionProvider } from "./session-provider"

import CreateButton from "./create"
import { SearchInput } from "./search"
import NavBar from "./navbar"

import { Toast } from "@/components/new/toast"
import { RealtimePriceUpdates } from "@/components/realtime-price"
import ApolloServerWrapper from "@/lib/apollo-client/server-wrapper"

import { HelveticaNeueFont, RobotoMono } from "@/lib/ui/fonts"

// eslint-disable-next-line import/no-unassigned-import
import "@/lib/ui/globals.css"

export const metadata: Metadata = {
  title: "Lana Bank | Admin Panel",
  icons: [
    {
      rel: "icon",
      url: "/favicon.ico",
    },
  ],
}

const inter = Inter_Tight({ subsets: ["latin"], display: "auto" })

const PUBLIC_PAGES = ["/auth/login", "/auth/error", "/auth/verify"]

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  const headerList = await headers()
  const currentPath = headerList.get("x-current-path") || "/"

  const session = await getServerSession(authOptions)
  if (!session && !PUBLIC_PAGES.includes(currentPath)) redirect("/auth/login")
  if (session && PUBLIC_PAGES.includes(currentPath)) redirect("/")
  if (session && ["/", "/app"].includes(currentPath)) redirect("/dashboard")

  return (
    <html lang="en">
      <body
        className={`${HelveticaNeueFont.variable} ${RobotoMono.variable} ${inter.className} antialiased w-screen h-screen select-none`}
      >
        <AuthSessionProvider session={session}>
          <ApolloServerWrapper>
            <Toast />
            <RealtimePriceUpdates />
            {PUBLIC_PAGES.includes(currentPath) ? (
              children
            ) : (
              <AppLayout>{children}</AppLayout>
            )}
          </ApolloServerWrapper>
        </AuthSessionProvider>
      </body>
    </html>
  )
}

const AppLayout = ({ children }: Readonly<{ children: React.ReactNode }>) => (
  <div className="bg-soft h-full w-full flex flex-col md:flex-row">
    <NavBar />
    <div className="flex-1 pt-[72px] md:pt-2 p-2 max-h-screen overflow-hidden">
      <div className="p-2 border rounded-md flex flex-col w-full h-full">
        <div className="md:flex gap-2 hidden pb-2">
          <SearchInput />
          <CreateButton />
        </div>
        <main className="h-full overflow-y-auto no-scrollbar">{children}</main>
      </div>
    </div>
  </div>
)
