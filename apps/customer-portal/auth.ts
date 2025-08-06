import NextAuth from "next-auth"
import Keycloak from "next-auth/providers/keycloak"

import { env } from "./env"

export const { handlers, auth, signIn, signOut } = NextAuth({
  secret: env.AUTH_SECRET,
  providers: [
    Keycloak({
      clientId: env.NEXT_PUBLIC_KEYCLOAK_CLIENT_ID,
      clientSecret: env.AUTH_KEYCLOAK_SECRET,
      issuer: `${env.NEXT_PUBLIC_KEYCLOAK_URL}/realms/${env.NEXT_PUBLIC_KEYCLOAK_REALM}`,
    }),
  ],
  callbacks: {
    async jwt({ token, account }) {
      if (account) {
        token.accessToken = account.access_token
        token.refreshToken = account.refresh_token
        token.expiresAt = account.expires_at
      }
      return token
    },
    async session({ session, token }) {
      session.accessToken = token.accessToken as string
      return session
    },
    authorized({ auth }) {
      return !!auth
    },
  },
})
