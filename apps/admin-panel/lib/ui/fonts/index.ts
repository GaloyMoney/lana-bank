import localFont from "next/font/local";

export const helveticaNeueFont = localFont({
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
});
