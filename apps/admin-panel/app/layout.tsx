import type { Metadata } from "next"
import { Inter_Tight } from "next/font/google"

// eslint-disable-next-line import/no-unassigned-import
import "./globals.css"
import { getServerSession } from "next-auth"

import { authOptions } from "./api/auth/[...nextauth]/options"
import { AuthSessionProvider } from "./session-provider"

import { ApolloClient } from "@/lib/apollo-client"

import { redirect } from "next/navigation"

import { SideBar } from "@/components/sidebar"
import { Toaster } from "@/components/primitive/toast"
import { RealtimePriceUpdates } from "@/components/realtime-price"

export const metadata: Metadata = {
  description: "lava Bank Admin Panel",
}

const inter = Inter_Tight({ subsets: ["latin"], display: "auto" })

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  const session = await getServerSession(authOptions)
  if (!session) {
    redirect("/api/auth/signin")
  }

  return (
    <html lang="en">
      <body className={inter.className}>
        <AuthSessionProvider session={session}>
          <ApolloClient>
            <Toaster />
            <RealtimePriceUpdates />
            <main className="flex flex-col md:flex-row min-h-screen w-full">
              <SideBar />
              <div className="flex-1 p-6 h-screen overflow-y-auto">{children}</div>
            </main>
          </ApolloClient>
        </AuthSessionProvider>
      </body>
    </html>
  )
}
