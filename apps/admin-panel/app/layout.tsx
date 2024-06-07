import type { Metadata } from "next";
import { Inter_Tight } from "next/font/google";
import "./globals.css";
import { ApolloWrapper } from "@/lib/core-admin-client/ssr-wrapper";

export const metadata: Metadata = {
  title: "Create Next App",
  description: "Generated by create next app",
};

const inter = Inter_Tight({ subsets: ["latin"], display: "auto" });

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <ApolloWrapper>{children}</ApolloWrapper>
      </body>
    </html>
  );
}
