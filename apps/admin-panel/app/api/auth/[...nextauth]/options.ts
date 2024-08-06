import EmailProvider from "next-auth/providers/email"
import CredentialsProvider from "next-auth/providers/credentials"
import { NextAuthOptions } from "next-auth"
import axios from "axios"

import { customPostgresAdapter } from "@/lib/auth/db/auth-adapter"
import { pool } from "@/lib/auth/db"
import { env } from "@/env"

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

    // For admin user
    CredentialsProvider({
      credentials: {
        username: { label: "Username", type: "text" },
        password: { label: "Password", type: "password" },
      },
      authorize: async (credentials) => {
        const adminUserName = env.ADMIN_CREDENTIALS.split(":")[0]
        const adminPassword = env.ADMIN_CREDENTIALS.split(":")[1]
        if (
          credentials?.username === adminUserName &&
          credentials?.password === adminPassword
        ) {
          return { id: "1", email: env.ADMIN_EMAIL }
        }
        return null
      },
    }),
  ],
  session: {
    strategy: "jwt",
  },
  callbacks: {
    async signIn({ account, user }) {
      const email = account?.providerAccountId
      if (account?.provider === "email" && email) {
        return checkUserEmail(email)
      } else if (account?.provider === "credentials" && user.email) {
        return checkUserEmail(user.email)
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
}
