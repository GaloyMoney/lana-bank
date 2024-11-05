import {
  Adapter,
  AdapterSession,
  AdapterUser,
  VerificationToken,
} from "next-auth/adapters"
import type { Pool } from "pg"

export function customPostgresAdapter(client: Pool): Adapter {
  return {
    async createVerificationToken(
      verificationToken: VerificationToken,
    ): Promise<VerificationToken> {
      const { identifier, expires, token } = verificationToken
      const sql = `
        INSERT INTO verification_token ( identifier, expires, token ) 
        VALUES ($1, $2, $3)
        `
      await client.query(sql, [identifier, expires, token])
      return verificationToken
    },
    async useVerificationToken({
      identifier,
      token,
    }: {
      identifier: string
      token: string
    }): Promise<VerificationToken> {
      const sql = `
        delete from verification_token
        where identifier = $1 and token = $2
        RETURNING identifier, expires, token 
        `
      const result = await client.query(sql, [identifier, token])
      return result.rowCount !== 0 ? result.rows[0] : null
    },
    // need these functions to satisfy the interface, and to resolve next-auth errors.
    //  these are not used anywhere.
    async createUser(user: { email: string; emailVerified: Date }): Promise<AdapterUser> {
      return { id: user.email, ...user }
    },
    async getUser() {
      return null
    },
    async getUserByEmail() {
      return null
    },
    async getUserByAccount() {
      return null
    },
    async updateUser(user): Promise<AdapterUser> {
      return user as AdapterUser
    },
    async linkAccount() {
      return null
    },
    async getSessionAndUser() {
      return null
    },
    async updateSession() {
      return null
    },
    async deleteSession() {
      return null
    },
    async deleteUser() {
      return null
    },
    async createSession({
      sessionToken,
      userId,
      expires,
    }: {
      sessionToken: string
      userId: string
      expires: Date
    }): Promise<AdapterSession> {
      return { sessionToken, userId, expires }
    },
    async unlinkAccount(): Promise<void> {
      return
    },
  }
}
