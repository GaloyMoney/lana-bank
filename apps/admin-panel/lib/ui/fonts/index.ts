import localFont from "next/font/local"

export const HelveticaNeueFont = localFont({
  src: [
    {
      path: "./HelveticaNeue-Light.otf",
      weight: "300",
      style: "normal",
    },
    {
      path: "./HelveticaNeue-Medium.otf",
      weight: "500",
      style: "normal",
    },
    {
      path: "./HelveticaNeue-Bold.otf",
      weight: "700",
      style: "normal",
    },
  ],
  variable: "--font-helvetica-neue",
})

import { Roboto_Mono as GoogleFont_RobotoMono } from "next/font/google"

export const RobotoMono = GoogleFont_RobotoMono({
  weight: "300",
  subsets: ["latin"],
  variable: "--font-roboto-mono",
})
