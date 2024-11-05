import { Toast } from "@/components/toast"

import { HelveticaNeueFont, RobotoMono } from "@/lib/ui/fonts"
import "@/lib/ui/globals.css"

import { AuthSessionProvider } from "./session-provider"
import { getServerSession } from "next-auth"

import { Metadata } from "next/types"
import { redirect } from "next/navigation"
import { authOptions } from "./api/auth/[...nextauth]/options"

export const metadata: Metadata = {
  title: "Lana Bank | Admin Panel",
  icons: [
    {
      rel: "icon",
      url: "/favicon.ico",
    },
  ],
}

const RootLayout = async ({
  children,
}: Readonly<{
  children: React.ReactNode
}>) => {
  const session = await getServerSession(authOptions)
  if (!session) {
    redirect("/api/auth/signin")
  }

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
