import { jwtVerify, createRemoteJWKSet } from "jose"

import { env } from "@/env"

const JWKS = createRemoteJWKSet(new URL(env.JWKS_URL))

export const verifyToken = async (token: string) => {
  const { payload } = await jwtVerify(token, JWKS, {
    algorithms: ["RS256"],
  })
  return payload
}
