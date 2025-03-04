import { NextRequest, NextResponse } from "next/server"

import { toSession } from "./lib/kratos/public/to-session"

const basePath = process.env.NEXT_PUBLIC_BASE_PATH || ""

export async function middleware(request: NextRequest): Promise<NextResponse | void> {
  const path = request.nextUrl.pathname
  const isAuthRoute = path.startsWith("/auth")

  try {
    const cookies = request.cookies
      .getAll()
      .reduce((acc, cookie) => `${acc}${cookie.name}=${cookie.value}; `, "")

    const session = await toSession({ cookie: cookies })

    if (
      !(session instanceof Error) &&
      session.authenticator_assurance_level &&
      parseInt(session.authenticator_assurance_level[3]) >= 2 &&
      session?.active
    ) {
      if (isAuthRoute) {
        return NextResponse.redirect(new URL(basePath || "/", request.url))
      }
      return NextResponse.next()
    }

    if (
      !(session instanceof Error) &&
      session.authenticator_assurance_level &&
      parseInt(session.authenticator_assurance_level[3]) < 2 &&
      session?.active
    ) {
      return NextResponse.next()
    }

    if (!isAuthRoute) {
      return NextResponse.redirect(new URL(`${basePath}/auth`, request.url))
    }
  } catch (error) {
    if (!isAuthRoute) {
      return NextResponse.redirect(new URL(`${basePath}/auth`, request.url))
    }
  }

  return NextResponse.next()
}

export const config = {
  matcher: ["/", "/((?!api|_next/static|_next/image|favicon.ico).*)"],
}
