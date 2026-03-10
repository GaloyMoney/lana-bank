import type { Metadata } from "next"
import { InterTight } from "@lana/web/fonts"

import "./globals.css"
import { PublicEnvScript } from "next-runtime-env"

import { ThemeProvider } from "next-themes"

import { Toaster } from "@lana/web/ui/toast"

import NavBar from "@/components/nav-bar"
import { SessionProvider } from "@/components/auth/session-provider"
import { getSessionWithData } from "@/lib/auth/get-session-with-data"

export const metadata: Metadata = {
  title: "Lana Bank",
  description: "Where the lana keeps flowing",
}

export const dynamic = "force-dynamic"

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  const { session, meData } = await getSessionWithData()

  return (
    <html lang="en">
      {process.env.NODE_ENV === "development" ||
      process.env.RUNNING_IN_CI === "true" ? null : (
        <head>
          <PublicEnvScript />
        </head>
      )}
      <body className={`${InterTight.className} mb-8`}>
        <SessionProvider session={session}>
          <ThemeProvider
            attribute="class"
            defaultTheme="light"
            enableSystem
            disableTransitionOnChange
          >
            {meData ? <NavBar meQueryData={meData} /> : null}
            {children}
            <Toaster />
          </ThemeProvider>
        </SessionProvider>
      </body>
    </html>
  )
}
