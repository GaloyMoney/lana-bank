import EmailProvider from "next-auth/providers/email"
import { NextAuthOptions } from "next-auth"
import axios from "axios"

import { customPostgresAdapter } from "@/lib/auth/db/auth-adapter"
import { pool } from "@/lib/auth/db"
import { basePath, env } from "@/env"

async function checkUserEmail(email: string): Promise<boolean> {
  try {
    const response = await axios.post(env.CHECK_USER_ALLOWED_CALLBACK_URL, {
      email: email,
      transient_payload: {},
    })

    console.log("User check response:", response.status)
    return response.status === 200
  } catch (error) {
    console.error("Error checking user:", error)
    return false
  }
}

export const authOptions: NextAuthOptions = {
  providers: [
    EmailProvider({
      server: env.EMAIL_SERVER,
      from: env.EMAIL_FROM,
    }),
  ],
  session: {
    strategy: "jwt",
  },
  callbacks: {
    async redirect() {
      return `${basePath}/dashboard`
    },
    async signIn({ account }) {
      const email = account?.providerAccountId
      if (account?.provider === "email" && email) {
        return checkUserEmail(email)
      }
      return false
    },
    async session({ session, token }) {
      if (session.user && token.email) {
        session.user.name = token.email.split("@")[0]
        session.user.email = token.email
      }
      return session
    },
  },
  adapter: customPostgresAdapter(pool),
  secret: env.NEXTAUTH_SECRET,
  pages: {
    signIn: `${basePath}/auth/login`,
    error: `${basePath}/auth/error`,
    verifyRequest: `${basePath}/auth/verify`,
  },
}
