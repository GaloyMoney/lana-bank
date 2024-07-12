import type { Metadata } from "next"
import { Inter_Tight } from "next/font/google"

// eslint-disable-next-line import/no-unassigned-import
import "./globals.css"
import { PublicEnvScript } from "next-runtime-env"

import { Toaster } from "@/components/primitive/toast"
import NavBar from "@/components/nav-bar"

export const metadata: Metadata = {
  title: "Lava Bank",
  description: "Where the lava keeps flowing",
}
const inter = Inter_Tight({ subsets: ["latin"], display: "auto" })

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="en">
      <head>
        <PublicEnvScript />
      </head>
      <body className={inter.className}>
        <NavBar />
        {children}
        <Toaster />
      </body>
    </html>
  )
}
