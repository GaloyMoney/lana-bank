import { helveticaNeueFont } from "@/lib/ui/fonts"
import "@/lib/ui/globals.css"

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="en">
      <body className={`${helveticaNeueFont.variable} antialiased`}>{children}</body>
    </html>
  )
}

import type { Metadata } from "next"

export const metadata: Metadata = {
  title: "Lana Bank | Admin Panel",
  icons: [
    {
      rel: "icon",
      url: "/favicon.ico",
    },
  ],
}
