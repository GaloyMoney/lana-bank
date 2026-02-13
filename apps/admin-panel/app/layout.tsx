import "./globals.css"
import "simplebar-react/dist/simplebar.min.css"

import type { Metadata } from "next"
import { NextIntlClientProvider } from "next-intl"
import { getLocale, getMessages } from "next-intl/server"
import { Inter_Tight } from "next/font/google"

import AppLoading from "./app-loading"
import { Authenticated } from "./auth/session"

export const metadata: Metadata = {
  title: "Lana Bank",
  description:
    "Unlock the power of Bitcoin-backed lending with Lana Bank – fast, secure, and seamless",
}

const inter = Inter_Tight({
  subsets: ["latin"],
  variable: "--font-inter",
})

const RootLayout: React.FC<React.PropsWithChildren> = async ({ children }) => {
  const locale = await getLocale()
  const messages = await getMessages()

  return (
    <html lang={locale} suppressHydrationWarning>
      <body className={`${inter.className} antialiased bg-background`}>
        <NextIntlClientProvider messages={messages}>
          <AppLoading>
            <Authenticated>{children}</Authenticated>
          </AppLoading>
        </NextIntlClientProvider>
      </body>
    </html>
  )
}

export default RootLayout
