import type { Metadata } from "next"
import { Inter_Tight } from "next/font/google"

// eslint-disable-next-line import/no-unassigned-import
import "./globals.css"
import { PublicEnvScript } from "next-runtime-env"

import { ThemeProvider } from "next-themes"

import { Toaster } from "@lana/web/ui/toast"

import NavBar from "@/components/nav-bar"
import { meQuery } from "@/lib/graphql/query/me"

export const metadata: Metadata = {
  title: "Lana Bank",
  description: "Where the lana keeps flowing",
}
const inter = Inter_Tight({ subsets: ["latin"], display: "auto" })

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  const session = await meQuery()
  return (
    <html lang="en">
      {process.env.NODE_ENV === "development" ||
      process.env.RUNNING_IN_CI === "true" ? null : (
        <head>
          <PublicEnvScript />
        </head>
      )}
      <body className={`${inter.className} mb-8`}>
        <ThemeProvider
          attribute="class"
          defaultTheme="light"
          enableSystem
          disableTransitionOnChange
        >
          {session instanceof Error ? null : <NavBar meQueryData={session} />}
          {children}
          <Toaster />
        </ThemeProvider>
      </body>
    </html>
  )
}
