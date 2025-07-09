import { NextRequest, NextResponse } from "next/server"

export async function GET(request: NextRequest) {
  try {
    // Basic request validation
    const userAgent = request.headers.get("user-agent")
    if (!userAgent) {
      return NextResponse.json(
        { error: "Invalid request" },
        { status: 400 }
      )
    }

    // Check if the request method is allowed
    if (request.method !== "GET") {
      return NextResponse.json(
        { error: "Method not allowed" },
        { status: 405, headers: { Allow: "GET" } }
      )
    }

    // Return minimal health information without exposing system details
    return NextResponse.json(
      {
        status: "healthy",
        timestamp: new Date().toISOString(),
      },
      {
        status: 200,
        headers: {
          "Cache-Control": "no-cache, no-store, must-revalidate",
          "Pragma": "no-cache",
          "Expires": "0",
        },
      }
    )
  } catch (error) {
    // Log error without exposing details to client
    console.error("Health check error:", error)
    
    return NextResponse.json(
      { error: "Internal server error" },
      { status: 500 }
    )
  }
}

// Explicitly handle other HTTP methods
export async function POST() {
  return NextResponse.json(
    { error: "Method not allowed" },
    { status: 405, headers: { Allow: "GET" } }
  )
}

export async function PUT() {
  return NextResponse.json(
    { error: "Method not allowed" },
    { status: 405, headers: { Allow: "GET" } }
  )
}

export async function DELETE() {
  return NextResponse.json(
    { error: "Method not allowed" },
    { status: 405, headers: { Allow: "GET" } }
  )
}
