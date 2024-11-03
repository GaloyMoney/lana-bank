import { Toast } from "@/components/toast"

import { HelveticaNeueFont, RobotoMono } from "@/lib/ui/fonts"
import "@/lib/ui/globals.css"

const RootLayout: React.FC<React.PropsWithChildren> = ({ children }) => (
  <html lang="en">
    <body
      className={`${HelveticaNeueFont.variable} ${RobotoMono.variable} antialiased w-screen h-screen`}
    >
      <Toast />
      {children}
    </body>
  </html>
)

export default RootLayout

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
