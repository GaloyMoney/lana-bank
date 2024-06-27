import { NextRequest, NextResponse } from "next/server"

import { verifyToken } from "./lib/auth/jwks"

const privateRoutes = [/^\/$/]

export async function middleware(request: NextRequest): Promise<NextResponse | void> {
  const token = request.headers.get("authorization")

  /* Next two lines shouldn't throw errors unless requests are not coming through oathkeeper or JWKS_URL is invalid */
  if (!token) throw new Error("Authorization header not found")
  const decodedToken = await verifyToken(token.split(" ")[1])

  const isPrivateRoute = privateRoutes.some((regex) =>
    regex.test(request.nextUrl.pathname),
  )

  if (isPrivateRoute && decodedToken.sub === "anonymous") {
    return NextResponse.redirect(new URL("/auth", request.url))
  } else if (!isPrivateRoute && decodedToken.sub !== "anonymous") {
    return NextResponse.redirect(new URL("/", request.url))
  }

  return NextResponse.next()
}

export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api (API routes)
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     */
    "/((?!api|_next/static|_next/image|favicon.ico).*)",
  ],
}
