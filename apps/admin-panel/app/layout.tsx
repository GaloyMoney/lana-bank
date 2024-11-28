import type { Metadata } from "next"
import { Inter_Tight } from "next/font/google"
import { getServerSession } from "next-auth"
import { redirect } from "next/navigation"
import { headers } from "next/headers"

import { authOptions } from "./api/auth/[...nextauth]/options"
import { AuthSessionProvider } from "./session-provider"

import CreateButton, { CreateContextProvider } from "./create"

import { RealtimePriceUpdates } from "@/components/realtime-price"
import ApolloServerWrapper from "@/lib/apollo-client/server-wrapper"
import { Toast } from "@/components/toast"
import { SidebarProvider, SidebarInset } from "@/ui/sidebar"
import { AppSidebar } from "@/components/app-sidebar"

// eslint-disable-next-line import/no-unassigned-import
import "./globals.css"

export const metadata: Metadata = {
  title: "Lana Bank | Admin Panel",
}

const inter = Inter_Tight({
  subsets: ["latin"],
  variable: "--font-inter",
})

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
      <body className={`${inter.className} antialiased select-none bg-background`}>
        <AuthSessionProvider session={session}>
          <ApolloServerWrapper>
            <SidebarProvider>
              <Toast />
              <AppSidebar />
              <SidebarInset className="min-h-screen md:peer-data-[variant=inset]:shadow-none border">
                {PUBLIC_PAGES.includes(currentPath) ? (
                  children
                ) : (
                  <AppLayout>{children}</AppLayout>
                )}
              </SidebarInset>
            </SidebarProvider>
          </ApolloServerWrapper>
        </AuthSessionProvider>
      </body>
    </html>
  )
}

const AppLayout = ({ children }: Readonly<{ children: React.ReactNode }>) => {
  return (
    <CreateContextProvider>
      <div className="container mx-auto p-2">
        <div className="max-w-7xl w-full mx-auto">
          <header className="flex justify-between items-center mb-2">
            <div className="font-semibold text-sm p-2 bg-secondary rounded-md">
              Welcome to Lana Bank
            </div>
            <CreateButton />
          </header>

          <RealtimePriceUpdates />
          <main>{children}</main>
        </div>
      </div>
    </CreateContextProvider>
  )
}
