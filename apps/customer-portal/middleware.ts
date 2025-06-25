import { NextRequest, NextResponse } from "next/server"

import { toSession } from "./lib/kratos/public/to-session"

export async function middleware(request: NextRequest): Promise<NextResponse | void> {
  const path = request.nextUrl.pathname
  const isAuthRoute = path.startsWith("/auth")

  try {
    const cookies = request.cookies
      .getAll()
      .reduce((acc, cookie) => `${acc}${cookie.name}=${cookie.value}; `, "")

    const session = await toSession({ cookie: cookies })

    if (!(session instanceof Error) && session?.active) {
      if (isAuthRoute) {
        return NextResponse.redirect(new URL("/", request.url))
      }
      return NextResponse.next()
    }

    if (!isAuthRoute) {
      return NextResponse.redirect(new URL("/auth", request.url))
    }
  } catch (error) {
    if (!isAuthRoute) {
      return NextResponse.redirect(new URL("/auth", request.url))
    }
  }

  return NextResponse.next()
}

export const config = {
  matcher: ["/", "/((?!api|_next/static|_next/image|favicon.ico).*)"],
}
