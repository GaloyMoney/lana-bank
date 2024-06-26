import { NextRequest, NextResponse } from "next/server"

import { verifyToken } from "./lib/auth/jwks"

export async function middleware(request: NextRequest): Promise<NextResponse | void> {
  const token = request.headers.get("authorization")

  /* Shouldn't happen unless requests are not coming through oathkeeper */
  if (!token) throw new Error("Authorization header not found")

  const decodedToken = await verifyToken(token.split(" ")[1])

  if (decodedToken.sub === "anonymous") {
    return NextResponse.redirect(new URL("/auth", request.url))
  }

  return NextResponse.next()
}
